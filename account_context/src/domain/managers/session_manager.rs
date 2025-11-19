use crate::domain::repositories::{SessionInfo, SessionRepository};
use crate::domain::valueobjects::{SessionId, UserId};
use crate::error::UserError;

pub struct SessionManager {
  session_repository: Box<dyn SessionRepository>,
  session_duration_hours: u64,
}

impl SessionManager {
  pub fn new(session_repository: Box<dyn SessionRepository>, session_duration_hours: u64) -> Self {
    Self {
      session_repository,
      session_duration_hours,
    }
  }

  /// Create a new session for a user
  /// The session expiration is calculated automatically based on the configured duration
  pub async fn create_session(&self, user_id: UserId) -> Result<SessionId, UserError> {
    let session_id = SessionId::make();
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(self.session_duration_hours as i64);
    self.session_repository.create(session_id, user_id, expires_at).await?;
    Ok(session_id)
  }

  /// Find session by session_id
  pub async fn find_session(
    &self, session_id: SessionId,
  ) -> Result<Option<(UserId, chrono::DateTime<chrono::Utc>)>, UserError> {
    self.session_repository.find_by_session_id(session_id).await
  }

  /// Delete a session by session_id
  pub async fn delete_session(&self, session_id: SessionId) -> Result<bool, UserError> {
    self.session_repository.delete(session_id).await
  }

  /// Delete all expired sessions
  pub async fn delete_expired_sessions(&self) -> Result<u64, UserError> {
    self.session_repository.delete_expired().await
  }

  /// Delete all sessions for a specific user
  pub async fn delete_sessions_by_user(&self, user_id: UserId) -> Result<u64, UserError> {
    self.session_repository.delete_by_user_id(user_id).await
  }

  /// Set expiration time for all active sessions of a specific user
  /// This gives existing sessions a grace period before they expire
  /// Returns the number of sessions that were updated
  pub async fn set_expiration_for_user_sessions(
    &self, user_id: UserId, expires_at: chrono::DateTime<chrono::Utc>,
  ) -> Result<u64, UserError> {
    self
      .session_repository
      .update_expiration_by_user_id(user_id, expires_at)
      .await
  }

  /// List all sessions
  pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>, UserError> {
    self.session_repository.list_all().await
  }

  /// List non-expired sessions (Active and Expiring)
  pub async fn list_non_expired_sessions(&self) -> Result<Vec<SessionInfo>, UserError> {
    self.session_repository.list_non_expired().await
  }

  /// Get session information by session_id
  pub async fn get_session(&self, session_id: SessionId) -> Result<SessionInfo, UserError> {
    self
      .session_repository
      .get_by_session_id(session_id)
      .await?
      .ok_or(UserError::NotFound)
  }
}
