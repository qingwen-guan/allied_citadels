use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::{Html, Json, Response};
use axum::routing::get;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

use room_context::services::RoomService;
use user_context::services::UserService;

use crate::services::SessionService;

// JSON-RPC 2.0 version constant
const JSON_RPC_VERSION: &str = "2.0";

// JSON-RPC 2.0 standard error codes
const JSON_RPC_INVALID_REQUEST: i32 = -32600;
const JSON_RPC_METHOD_NOT_FOUND: i32 = -32601;
const JSON_RPC_INTERNAL_ERROR: i32 = -32603;

// Application-specific error codes
const ERR_INVALID_CREDENTIALS: i32 = -32001;
const ERR_USER_NOT_FOUND: i32 = -32002;
const ERR_SESSION_NOT_FOUND: i32 = -32001;
const ERR_SESSION_EXPIRED: i32 = -32002;
const ERR_INVALID_OPERATION: i32 = -32003;
const ERR_INVALID_MAX_PLAYERS: i32 = -32004;
const ERR_ROOM_NAME_EXISTS: i32 = -32005;

// Shared application state
#[derive(Clone)]
pub struct AppState {
  pub user_service: Arc<UserService>,
  pub room_service: Arc<RoomService>,
  pub session_service: Arc<SessionService>,
  pub connections: Arc<Mutex<HashMap<String, ConnectionInfo>>>,
}

#[derive(Clone)]
pub struct ConnectionInfo {
  #[allow(dead_code)]
  pub id: String,
  #[allow(dead_code)]
  pub connected_at: chrono::DateTime<chrono::Utc>,
}

// JSON-RPC 2.0 structures
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcCall {
  pub jsonrpc: String,
  pub method: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub params: Option<serde_json::Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
  jsonrpc: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  result: Option<serde_json::Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  error: Option<JsonRpcError>,
  #[serde(skip_serializing_if = "Option::is_none")]
  id: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
  code: i32,
  message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
  pub fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
    Self {
      jsonrpc: JSON_RPC_VERSION.to_string(),
      result: Some(result),
      error: None,
      id,
    }
  }

  pub fn error(id: Option<serde_json::Value>, code: i32, message: String, data: Option<serde_json::Value>) -> Self {
    Self {
      jsonrpc: JSON_RPC_VERSION.to_string(),
      result: None,
      error: Some(JsonRpcError { code, message, data }),
      id,
    }
  }
}

// HTTP API structures

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
  status: String,
  timestamp: u64,
}

pub fn create_router(state: AppState) -> Router {
  Router::new()
        // HTTP routes
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/api/health", get(health_handler))
        // WebSocket route
        .route("/ws", get(websocket_handler))
        // Add CORS middleware
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()))
        .with_state(state)
}

// HTTP handlers
async fn root_handler() -> Html<&'static str> {
  Html(include_str!("static/index.html"))
}

async fn health_handler() -> Json<HealthResponse> {
  Json(HealthResponse {
    status: "ok".to_string(),
    timestamp: std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs(),
  })
}

// WebSocket handler
async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
  ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
  let (mut sender, mut receiver) = socket.split();
  let conn_id = uuid::Uuid::new_v4().to_string();

  // Add connection to state
  {
    let mut conns = state.connections.lock().await;
    conns.insert(
      conn_id.clone(),
      ConnectionInfo {
        id: conn_id.clone(),
        connected_at: chrono::Utc::now(),
      },
    );
    let count = conns.len();
    info!("New WebSocket connection: {} (total: {})", conn_id, count);
  }

  // Send welcome message as JSON-RPC notification
  let welcome = json!({
      "jsonrpc": JSON_RPC_VERSION,
      "method": "welcome",
      "params": {
          "connection_id": conn_id,
          "message": "Connected to Session Context Server"
      }
  });
  let welcome_text = serde_json::to_string(&welcome).unwrap();
  let _ = sender.send(Message::Text(welcome_text.into())).await;

  // Handle incoming messages
  while let Some(Ok(msg)) = receiver.next().await {
    match msg {
      Message::Text(text) => {
        match handle_jsonrpc_request(&text, &state).await {
          Ok(Some(response)) => {
            let response_text = serde_json::to_string(&response).unwrap_or_else(|e| {
              error!("Failed to serialize response: {}", e);
              String::new()
            });
            if !response_text.is_empty() {
              if sender.send(Message::Text(response_text.into())).await.is_err() {
                break;
              }
            }
          },
          Ok(None) => {
            // Notification, no response needed
          },
          Err(e) => {
            error!("Error handling JSON-RPC request: {}", e);
            // Try to send error response if we can parse the request ID
            if let Ok(req) = serde_json::from_str::<JsonRpcCall>(&text) {
              let id = req.id.clone();
              let error_response = JsonRpcResponse::error(
                id,
                JSON_RPC_INTERNAL_ERROR,
                "Internal error".to_string(),
                Some(json!({ "message": e.to_string() })),
              );
              let error_text = serde_json::to_string(&error_response).unwrap_or_default();
              if !error_text.is_empty() {
                let _ = sender.send(Message::Text(error_text.into())).await;
              }
            }
          },
        }
      },
      Message::Close(_) => {
        info!("WebSocket connection {} closed", conn_id);
        break;
      },
      _ => {},
    }
  }

  // Remove connection from state
  {
    let mut conns = state.connections.lock().await;
    conns.remove(&conn_id);
    info!("WebSocket connection {} removed (total: {})", conn_id, conns.len());
  }
}

async fn handle_jsonrpc_request(
  text: &str, state: &AppState,
) -> Result<Option<JsonRpcResponse>, Box<dyn std::error::Error + Send + Sync>> {
  let request: JsonRpcCall = serde_json::from_str(text)?;

  // Validate jsonrpc version
  if request.jsonrpc != JSON_RPC_VERSION {
    return Ok(Some(JsonRpcResponse::error(
      request.id.clone(),
      JSON_RPC_INVALID_REQUEST,
      format!(
        "Invalid jsonrpc version: expected '{}', got '{}'",
        JSON_RPC_VERSION, request.jsonrpc
      ),
      None,
    )));
  }

  // If id is None, it's a notification (no response needed)
  if request.id.is_none() {
    return handle_jsonrpc_method(&request.method, request.params.clone(), state, None).await;
  }

  handle_jsonrpc_method(&request.method, request.params.clone(), state, request.id.clone()).await
}

async fn handle_jsonrpc_method(
  method: &str, params: Option<serde_json::Value>, state: &AppState, id: Option<serde_json::Value>,
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
      // This is a notification, no response
      Ok(None)
    },
    _ => Ok(Some(JsonRpcResponse::error(
      id,
      JSON_RPC_METHOD_NOT_FOUND,
      format!("Method not found: {}", method),
      None,
    ))),
  }
}

pub async fn start_server(
  addr: &str, user_service: UserService, room_service: RoomService,
) -> Result<(), Box<dyn std::error::Error>> {
  use common_context::database::create_postgres_pool;
  use room_context::infra::{PostgresMessageRepository, PostgresRoomRepository};
  use room_context::managers::RoomManager;
  use user_context::domain::SessionManager;
  use user_context::infra::{PostgresSessionRepository, PostgresUserRepository};

  use crate::config::SessionConfig;

  // Create managers directly (not from services)
  let session_config = SessionConfig::load()?;
  let db_pool = create_postgres_pool(&session_config.db).await?;

  let session_repository = Box::new(PostgresSessionRepository::new(db_pool.clone()));
  let session_manager = Arc::new(SessionManager::new(
    session_repository,
    session_config.session_duration_hours,
  ));

  let room_repository = Box::new(PostgresRoomRepository::new(db_pool.clone()));
  let user_repository = Box::new(PostgresUserRepository::new(db_pool.clone()));
  let message_repository = Box::new(PostgresMessageRepository::new(db_pool));
  let room_manager = Arc::new(RoomManager::new(room_repository, user_repository, message_repository));

  let user_service = Arc::new(user_service);
  let room_service = Arc::new(room_service);
  let session_service = Arc::new(SessionService::new(session_manager, room_manager));
  let state = AppState {
    user_service,
    room_service,
    session_service,
    connections: Arc::new(Mutex::new(HashMap::new())),
  };

  let app = create_router(state);

  let listener = tokio::net::TcpListener::bind(addr).await?;

  // Replace 0.0.0.0 with localhost for display purposes
  let display_addr = addr.replace("0.0.0.0", "localhost");

  println!("🚀 Session Context server running on http://{}", display_addr);
  println!("📡 WebSocket available at ws://{}/ws", display_addr);
  println!("🌐 Open http://{} in your browser to test", display_addr);
  info!("🚀 Session Context server running on http://{}", addr);
  info!("📡 WebSocket available at ws://{}/ws", addr);

  axum::serve(listener, app).await?;

  Ok(())
}
