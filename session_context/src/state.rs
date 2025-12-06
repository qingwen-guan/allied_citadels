use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use room_context::services::RoomService;
use user_context::services::UserService;

use crate::services::SessionService;

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

