use async_trait::async_trait;

use crate::domain::valueobjects::{AccountId, SessionId};
use crate::error::AccountError;

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
  pub account_id: AccountId,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub expires_at: chrono::DateTime<chrono::Utc>,
  pub is_expired: bool,
  pub status: SessionStatus,
}

/// SessionRepository trait - interface for session data access
#[async_trait]
pub trait SessionRepository: Send + Sync {
  /// Create a new session for an account
  async fn create(
    &self, session_id: SessionId, account_id: AccountId, expires_at: chrono::DateTime<chrono::Utc>,
  ) -> Result<(), AccountError>;

  /// Find session by session_id
  async fn find_by_session_id(
    &self, session_id: SessionId,
  ) -> Result<Option<(AccountId, chrono::DateTime<chrono::Utc>)>, AccountError>;

  /// Delete a session by session_id
  async fn delete(&self, session_id: SessionId) -> Result<bool, AccountError>;

  /// Delete all expired sessions
  async fn delete_expired(&self) -> Result<u64, AccountError>;

  /// Delete all sessions for a specific account
  async fn delete_by_account_id(&self, account_id: AccountId) -> Result<u64, AccountError>;

  /// Update expiration time for all active sessions of a specific account
  /// Returns the number of sessions that were updated
  async fn update_expiration_by_account_id(
    &self, account_id: AccountId, expires_at: chrono::DateTime<chrono::Utc>,
  ) -> Result<u64, AccountError>;
  /// List all sessions with account information
  async fn list_all(&self) -> Result<Vec<SessionInfo>, AccountError>;

  /// List non-expired sessions (Active and Expiring) with account information
  async fn list_non_expired(&self) -> Result<Vec<SessionInfo>, AccountError>;

  /// Get session information by session_id
  async fn get_by_session_id(&self, session_id: SessionId) -> Result<Option<SessionInfo>, AccountError>;
}
