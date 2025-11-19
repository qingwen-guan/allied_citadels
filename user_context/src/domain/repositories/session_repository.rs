use async_trait::async_trait;

use crate::domain::valueobjects::{SessionId, UserId};
use crate::error::UserError;

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
  /// Session is active and has more than 1 minute remaining
  Active,
  /// Session is expiring (less than 1 minute remaining but not yet expired)
  Expiring,
  /// Session has expired
  Expired,
}

impl std::fmt::Display for SessionStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      SessionStatus::Active => write!(f, "ACTIVE"),
      SessionStatus::Expiring => write!(f, "EXPIRING"),
      SessionStatus::Expired => write!(f, "EXPIRED"),
    }
  }
}

/// Session information for listing
#[derive(Debug)]
pub struct SessionInfo {
  pub session_id: SessionId,
  pub user_id: UserId,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub expires_at: chrono::DateTime<chrono::Utc>,
  pub is_expired: bool,
  pub status: SessionStatus,
}

/// SessionRepository trait - interface for session data access
#[async_trait]
pub trait SessionRepository: Send + Sync {
  /// Create a new session for a user
  async fn create(
    &self, session_id: SessionId, user_id: UserId, expires_at: chrono::DateTime<chrono::Utc>,
  ) -> Result<(), UserError>;

  /// Find session by session_id
  async fn find_by_session_id(
    &self, session_id: SessionId,
  ) -> Result<Option<(UserId, chrono::DateTime<chrono::Utc>)>, UserError>;

  /// Delete a session by session_id
  async fn delete(&self, session_id: SessionId) -> Result<bool, UserError>;

  /// Delete all expired sessions
  async fn delete_expired(&self) -> Result<u64, UserError>;

  /// Delete all sessions for a specific user
  async fn delete_by_user_id(&self, user_id: UserId) -> Result<u64, UserError>;

  /// Update expiration time for all active sessions of a specific user
  /// Returns the number of sessions that were updated
  async fn update_expiration_by_user_id(
    &self, user_id: UserId, expires_at: chrono::DateTime<chrono::Utc>,
  ) -> Result<u64, UserError>;
  /// List all sessions with user information
  async fn list_all(&self) -> Result<Vec<SessionInfo>, UserError>;

  /// List non-expired sessions (Active and Expiring) with user information
  async fn list_non_expired(&self) -> Result<Vec<SessionInfo>, UserError>;

  /// Get session information by session_id
  async fn get_by_session_id(&self, session_id: SessionId) -> Result<Option<SessionInfo>, UserError>;
}
