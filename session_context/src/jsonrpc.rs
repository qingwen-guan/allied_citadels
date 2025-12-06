use serde::{Deserialize, Serialize};
use serde_json::json;

// JSON-RPC 2.0 version constant
pub const JSON_RPC_VERSION: &str = "2.0";

// JSON-RPC 2.0 standard error codes
pub const JSON_RPC_INVALID_REQUEST: i32 = -32600;
pub const JSON_RPC_METHOD_NOT_FOUND: i32 = -32601;
pub const JSON_RPC_INTERNAL_ERROR: i32 = -32603;

// Application-specific error codes
pub const ERR_INVALID_CREDENTIALS: i32 = -32001;
pub const ERR_USER_NOT_FOUND: i32 = -32002;
pub const ERR_SESSION_NOT_FOUND: i32 = -32006;
pub const ERR_SESSION_EXPIRED: i32 = -32007;
pub const ERR_INVALID_OPERATION: i32 = -32003;
pub const ERR_INVALID_MAX_PLAYERS: i32 = -32004;
pub const ERR_ROOM_NAME_EXISTS: i32 = -32005;

// JSON-RPC 2.0 structures
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
  pub jsonrpc: String,
  pub method: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub params: Option<serde_json::Value>,
  /// Request ID. This server only supports Requests (not Notifications).
  /// All method calls must include an `id` field and will receive a response.
  /// Operations that would typically be notifications should be called as Requests
  /// with an `id` and will return an acknowledgment response.
  /// Supports numeric, string, and UUID IDs per JSON-RPC 2.0 spec.
  pub id: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
  jsonrpc: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  result: Option<serde_json::Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  error: Option<JsonRpcError>,
  id: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
  code: i32,
  message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
  pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
    Self {
      jsonrpc: JSON_RPC_VERSION.to_string(),
      result: Some(result),
      error: None,
      id,
    }
  }

  pub fn error(id: serde_json::Value, code: i32, message: String, data: Option<serde_json::Value>) -> Self {
    Self {
      jsonrpc: JSON_RPC_VERSION.to_string(),
      result: None,
      error: Some(JsonRpcError { code, message, data }),
      id,
    }
  }
}

pub async fn handle_jsonrpc_request(
  text: &str, state: &crate::state::AppState,
) -> Result<Option<JsonRpcResponse>, Box<dyn std::error::Error + Send + Sync>> {
  let request: JsonRpcRequest = serde_json::from_str(text)?;

  // Validate id is a valid type per JSON-RPC 2.0 spec (number or string)
  // Null is valid per spec but represents a notification, which we don't support
  // If invalid, treat as notification (no response per spec)
  // This check MUST come before any error responses to ensure notifications never receive responses
  match &request.id {
    serde_json::Value::Number(_) | serde_json::Value::String(_) => {
      // Valid ID type - proceed with request
    },
    serde_json::Value::Null => {
      // Null ID is a notification - not supported, treat as notification (no response per JSON-RPC 2.0 spec)
      return Ok(None);
    },
    _ => {
      // Invalid ID type - treat as notification (no response per JSON-RPC 2.0 spec)
      return Ok(None);
    },
  }

  // Validate jsonrpc version (only after confirming ID is valid, so we can send error response)
  if request.jsonrpc != JSON_RPC_VERSION {
    return Ok(Some(JsonRpcResponse::error(
      request.id,
      JSON_RPC_INVALID_REQUEST,
      format!(
        "Invalid jsonrpc version: expected '{}', got '{}'",
        JSON_RPC_VERSION, request.jsonrpc
      ),
      None,
    )));
  }

  // All requests must have an id (id is required, so deserialization will fail if missing)
  execute_jsonrpc_method(&request.method, request.params.clone(), state, request.id).await
}

async fn execute_jsonrpc_method(
  method: &str, params: Option<serde_json::Value>, state: &crate::state::AppState, id: serde_json::Value,
) -> Result<Option<JsonRpcResponse>, Box<dyn std::error::Error + Send + Sync>> {
  match method {
    "ping" => Ok(Some(JsonRpcResponse::success(
      id,
      json!({ "pong": true, "timestamp": chrono::Utc::now().to_rfc3339() }),
    ))),
    "echo" => {
      let text = params
        .and_then(|p| p.get("text").and_then(|v| v.as_str().map(|s| s.to_string())))
        .unwrap_or_else(|| "No text provided".to_string());
      Ok(Some(JsonRpcResponse::success(
        id,
        json!({ "echo": text, "timestamp": chrono::Utc::now().to_rfc3339() }),
      )))
    },
    "login" => {
      let params = params.ok_or_else(|| "Missing params")?;
      let nickname = params
        .get("nickname")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing or invalid 'nickname' parameter")?;
      let password = params
        .get("password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing or invalid 'password' parameter")?;

      match state.user_service.login(nickname, password).await {
        Ok(login_response) => Ok(Some(JsonRpcResponse::success(
          id,
          json!({
            "session_id": login_response.session_id,
            "user_id": login_response.user_id,
          }),
        ))),
        Err(e) => {
          let (code, message) = match e {
            user_context::errors::UserError::InvalidCredentials => {
              (ERR_INVALID_CREDENTIALS, "Invalid credentials".to_string())
            },
            user_context::errors::UserError::NotFound => (ERR_USER_NOT_FOUND, "User not found".to_string()),
            user_context::errors::UserError::Database(err) => {
              (JSON_RPC_INTERNAL_ERROR, format!("Database error: {}", err))
            },
            user_context::errors::UserError::InvalidOperation(msg) => {
              (ERR_INVALID_OPERATION, format!("Invalid operation: {}", msg))
            },
            _ => (JSON_RPC_INTERNAL_ERROR, format!("Login failed: {}", e)),
          };
          Ok(Some(JsonRpcResponse::error(id, code, message, None)))
        },
      }
    },
    "room.list" => {
      // Parse optional offset and limit from params
      let offset = params
        .as_ref()
        .and_then(|p| p.get("offset"))
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);
      let limit = params
        .as_ref()
        .and_then(|p| p.get("limit"))
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);

      match state.room_service.list_active_rooms_detailed(offset, limit).await {
        Ok(rooms) => {
          // Convert RoomDetails to JSON
          let rooms_json: Vec<serde_json::Value> = rooms
            .into_iter()
            .map(|room| {
              json!({
                "id": room.id,
                "number": room.number,
                "name": room.name,
                "creator_id": room.creator_id,
                "creator_name": room.creator_name,
                "max_players": room.max_players,
                "seated_players": room.seated_players,
                "created_at": room.created_at.to_rfc3339(),
                "expires_at": room.expires_at.to_rfc3339(),
              })
            })
            .collect();
          Ok(Some(JsonRpcResponse::success(id, json!({ "rooms": rooms_json }))))
        },
        Err(e) => {
          let (code, message) = match e {
            room_context::errors::RoomError::Database(err) => {
              (JSON_RPC_INTERNAL_ERROR, format!("Database error: {}", err))
            },
            room_context::errors::RoomError::InvalidOperation(msg) => {
              (ERR_INVALID_OPERATION, format!("Invalid operation: {}", msg))
            },
            _ => (JSON_RPC_INTERNAL_ERROR, format!("Failed to list rooms: {}", e)),
          };
          Ok(Some(JsonRpcResponse::error(id, code, message, None)))
        },
      }
    },
    "room.create" => {
      let params = params.ok_or_else(|| "Missing params")?;
      let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing or invalid 'session_id' parameter")?;
      let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing or invalid 'name' parameter")?;
      let max_players = params
        .get("max_players")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "Missing or invalid 'max_players' parameter")? as usize;

      match state.session_service.create_room(session_id, name, max_players).await {
        Ok(room) => Ok(Some(JsonRpcResponse::success(
          id,
          json!({
            "id": room.id().to_string(),
            "number": room.number().value(),
            "name": room.name().as_str(),
            "creator_id": room.creator().to_string(),
            "max_players": room.max_players().value(),
            "created_at": room.created_at().to_rfc3339(),
            "expires_at": room.expires_at().to_rfc3339(),
          }),
        ))),
        Err(e) => {
          let (code, message) = match e {
            crate::services::SessionServiceError::SessionNotFound(_) => {
              (ERR_SESSION_NOT_FOUND, "Session not found".to_string())
            },
            crate::services::SessionServiceError::SessionExpired => {
              (ERR_SESSION_EXPIRED, "Session expired".to_string())
            },
            crate::services::SessionServiceError::InvalidOperation(msg) => {
              (ERR_INVALID_OPERATION, format!("Invalid operation: {}", msg))
            },
            crate::services::SessionServiceError::Database(err) => {
              (JSON_RPC_INTERNAL_ERROR, format!("Database error: {}", err))
            },
            crate::services::SessionServiceError::InvalidMaxPlayers => (
              ERR_INVALID_MAX_PLAYERS,
              "Invalid max players: must be 4 or 6".to_string(),
            ),
            crate::services::SessionServiceError::RoomNameExists => {
              (ERR_ROOM_NAME_EXISTS, "Room name already exists".to_string())
            },
          };
          Ok(Some(JsonRpcResponse::error(id, code, message, None)))
        },
      }
    },
    "welcome" => {
      // Acknowledgment for welcome message
      Ok(Some(JsonRpcResponse::success(id, json!({ "acknowledged": true }))))
    },
    _ => Ok(Some(JsonRpcResponse::error(
      id,
      JSON_RPC_METHOD_NOT_FOUND,
      format!("Method not found: {}", method),
      None,
    ))),
  }
}

