use std::sync::Arc;

use room_context::domain::entities::Room;
use room_context::managers::RoomManager;
use room_context::domain::valueobjects::{MaxPlayers, RoomName};
use room_context::errors::RoomError;
use user_context::domain::SessionManager;
use user_context::domain::valueobjects::SessionId;
use user_context::errors::UserError;

/// Service for session-based operations
/// This service layer provides session-aware operations that combine
/// user authentication (via sessions) with room operations
pub struct SessionService {
  session_manager: Arc<SessionManager>,
  room_manager: Arc<RoomManager>,
}

impl SessionService {
  pub fn new(session_manager: Arc<SessionManager>, room_manager: Arc<RoomManager>) -> Self {
    Self {
      session_manager,
      room_manager,
    }
  }

  /// Create a room using session_id for authentication
  /// Verifies the session exists and is not expired, then creates the room
  pub async fn create_room(
    &self, session_id: &str, name: &str, max_players: usize,
  ) -> Result<Room, SessionServiceError> {
    // Parse session_id string to SessionId
    let session_id_parsed = session_id
      .parse::<SessionId>()
      .map_err(|_| {
        SessionServiceError::InvalidOperation(format!("Invalid session_id format: {}", session_id))
      })?;

    // Get session info from SessionManager
    let session_info = self
      .session_manager
      .get_session(session_id_parsed)
      .await
      .map_err(|e| match e {
        UserError::NotFound => SessionServiceError::SessionNotFound(session_id.to_string()),
        UserError::InvalidOperation(msg) => SessionServiceError::InvalidOperation(msg),
        UserError::Database(err) => SessionServiceError::Database(err.to_string()),
        _ => SessionServiceError::InvalidOperation(format!("Failed to get session: {}", e)),
      })?;

    // Check if session is expired
    use user_context::domain::SessionStatus;
    if session_info.is_expired || session_info.status == SessionStatus::Expired {
      return Err(SessionServiceError::SessionExpired);
    }

    // Convert name to RoomName and max_players to MaxPlayers
    let room_name = RoomName::from(name);
    let max_players_vo = MaxPlayers::try_from(max_players)
      .map_err(|_| SessionServiceError::InvalidMaxPlayers)?;

    // Create the room using the user_id from the session
    self
      .room_manager
      .create_room(&room_name, session_info.user_id, max_players_vo)
      .await
      .map_err(|e| match e {
        RoomError::Database(err) => SessionServiceError::Database(err.to_string()),
        RoomError::InvalidOperation(msg) => SessionServiceError::InvalidOperation(msg),
        RoomError::InvalidMaxPlayers => SessionServiceError::InvalidMaxPlayers,
        RoomError::RoomNameExists => SessionServiceError::RoomNameExists,
        _ => SessionServiceError::InvalidOperation(format!("Failed to create room: {}", e)),
      })
  }
}

/// Errors that can occur in SessionService operations
#[derive(Debug, thiserror::Error)]
pub enum SessionServiceError {
  #[error("Session not found: {0}")]
  SessionNotFound(String),

  #[error("Session expired")]
  SessionExpired,

  #[error("Invalid operation: {0}")]
  InvalidOperation(String),

  #[error("Database error: {0}")]
  Database(String),

  #[error("Invalid max players: must be 4 or 6")]
  InvalidMaxPlayers,

  #[error("Room name already exists")]
  RoomNameExists,
}
