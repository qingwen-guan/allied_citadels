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

use crate::jsonrpc::{JSON_RPC_VERSION, JsonRpcId, handle_jsonrpc_request};
use crate::services::SessionService;
use crate::state::{AppState, ConnectionInfo};

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

  // Send welcome message as JSON-RPC request with server ID
  let welcome_id = JsonRpcId::new_server_id();
  let welcome = json!({
      "jsonrpc": JSON_RPC_VERSION,
      "method": "welcome",
      "params": {
          "connection_id": conn_id,
          "message": "Connected to Session Context Server"
      },
      "id": welcome_id.0
  });
  let welcome_text = serde_json::to_string(&welcome).unwrap();
  let _ = sender.send(Message::Text(welcome_text.into())).await;

  // Handle incoming messages
  while let Some(Ok(msg)) = receiver.next().await {
    match msg {
      Message::Text(text) => {
        match handle_jsonrpc_request(&text, &state).await {
          Some(response) => {
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
          None => {
            // Notification, no response needed
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
