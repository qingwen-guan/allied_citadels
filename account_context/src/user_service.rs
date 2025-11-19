use tracing::{error, info, instrument};
use uuid::Uuid;

use crate::config::Config;
use crate::domain::valueobjects::{SessionId, UserId};
use crate::domain::{
  NickName, SessionInfo, SessionManager, SessionRepository, User, UserFactory, UserManager, UserRepository,
};
use crate::error::UserError;

pub struct UserService {
  user_manager: UserManager,
  session_manager: SessionManager,
}

/// Result of resetting a password
#[allow(dead_code)]
pub struct ResetPasswordResult {
  pub uuid: String,
  pub nickname: Option<String>,
  pub password: String,
}

impl UserService {
  pub fn new(
    config: &Config, user_repository: Box<dyn UserRepository>, session_repository: Box<dyn SessionRepository>,
    user_factory: UserFactory,
  ) -> Self {
    let user_manager = UserManager::new(user_repository, user_factory);
    let session_manager = SessionManager::new(session_repository, config.session_duration_hours);
    Self {
      user_manager,
      session_manager,
    }
  }

  /// Create a new user with a randomly generated password
  #[instrument(skip(self), fields(nickname = nickname_str))]
  pub async fn create_user(&self, nickname_str: &str) -> Result<(Uuid, String), UserError> {
    let nickname = NickName::from(nickname_str);
    let result = self.user_manager.create_user(&nickname).await;
    match &result {
      Ok(_) => info!("Successfully created user: {}", nickname_str),
      Err(e) => error!("Failed to create user {}: {:?}", nickname_str, e),
    }
    result.map(|(uuid, p)| (uuid, p.into_string()))
  }

  /// Get user by UUID
  #[instrument(skip(self), fields(uuid = %uuid))]
  pub async fn get_user_by_uuid(&self, uuid: Uuid) -> Result<Option<User>, UserError> {
    let result = self.user_manager.get_user_by_uuid(uuid).await;
    if let Err(e) = &result {
      error!("Error getting user by UUID {}: {:?}", uuid, e);
    }
    result
  }

  /// Get user by nickname
  #[instrument(skip(self), fields(nickname = nickname_str))]
  pub async fn get_user_by_nickname(&self, nickname_str: &str) -> Result<Option<User>, UserError> {
    let nickname = NickName::from(nickname_str);
    let result = self.user_manager.get_user_by_nickname(&nickname).await;
    if let Err(e) = &result {
      error!("Error getting user by nickname {}: {:?}", nickname_str, e);
    }
    result
  }

  /// List all users
  #[instrument(skip(self))]
  pub async fn list_users(&self) -> Result<Vec<User>, UserError> {
    let result = self.user_manager.list_users().await;
    if let Err(ref e) = result {
      error!("Error listing users: {:?}", e);
    }
    result
  }

  /// Update user nickname
  #[instrument(skip(self), fields(uuid = %uuid, new_nickname = new_nickname_str))]
  pub async fn update_user_nickname(&self, uuid: Uuid, new_nickname_str: &str) -> Result<(), UserError> {
    let new_nickname = NickName::from(new_nickname_str);
    let result = self.user_manager.update_user_nickname(uuid, &new_nickname).await;
    match &result {
      Ok(_) => info!("Successfully updated user nickname: {} -> {}", uuid, new_nickname_str),
      Err(e) => error!(
        "Failed to update user nickname {} -> {}: {:?}",
        uuid, new_nickname_str, e
      ),
    }
    result
  }

  /// Reset password for a user by UUID
  #[instrument(skip(self), fields(uuid = %uuid))]
  pub async fn reset_password_by_uuid(&self, uuid: Uuid) -> Result<String, UserError> {
    let result = self.user_manager.reset_password_by_uuid(uuid).await;
    match &result {
      Ok(_) => info!("Successfully reset password for user: {}", uuid),
      Err(e) => error!("Failed to reset password for user {}: {:?}", uuid, e),
    }
    result.map(|p| p.into_string())
  }

  /// Reset password for a user by nickname
  #[instrument(skip(self), fields(nickname = nickname_str))]
  pub async fn reset_password_by_name(&self, nickname_str: &str) -> Result<(Uuid, String), UserError> {
    let nickname = NickName::from(nickname_str);
    let result = self.user_manager.reset_password_by_name(&nickname).await;
    match &result {
      Ok(_) => info!("Successfully reset password for user: {}", nickname_str),
      Err(e) => error!("Failed to reset password for user {}: {:?}", nickname_str, e),
    }
    result.map(|(uuid, p)| (uuid, p.into_string()))
  }

  /// Delete user by UUID
  #[instrument(skip(self), fields(uuid = %uuid))]
  pub async fn delete_user(&self, uuid: Uuid) -> Result<(), UserError> {
    let result = self.user_manager.delete_user(uuid).await;
    match &result {
      Ok(_) => info!("Successfully deleted user: {}", uuid),
      Err(e) => error!("Failed to delete user {}: {:?}", uuid, e),
    }
    result
  }

  /// Delete user by nickname
  #[instrument(skip(self), fields(nickname = nickname_str))]
  pub async fn delete_user_by_nickname(&self, nickname_str: &str) -> Result<(), UserError> {
    let nickname = NickName::from(nickname_str);
    let result = self.user_manager.delete_user_by_nickname(&nickname).await;
    match &result {
      Ok(_) => info!("Successfully deleted user: {}", nickname_str),
      Err(e) => error!("Failed to delete user {}: {:?}", nickname_str, e),
    }
    result
  }

  /// Verify login credentials and create a session
  #[instrument(skip(self), fields(nickname = nickname_str))]
  pub async fn login(&self, nickname_str: &str, password_str: &str) -> Result<(SessionId, UserId), UserError> {
    use crate::domain::valueobjects::RawPassword;

    let nickname = NickName::from(nickname_str);
    let password = RawPassword::from(password_str);
    let user = self.user_manager.login(&nickname, &password).await?;

    let user_id = UserId::from(user.uuid());
    let session_id = self.session_manager.create_session(user_id).await?;

    info!("User {} logged in successfully, session: {}", nickname_str, session_id);
    Ok((session_id, user_id))
  }

  /// Verify session and get user ID
  #[instrument(skip(self), fields(session_id = %session_id))]
  pub async fn verify_session(&self, session_id: SessionId) -> Result<UserId, UserError> {
    let session_info = self.session_manager.get_session(session_id).await?;
    match session_info.status {
      crate::domain::SessionStatus::Active | crate::domain::SessionStatus::Expiring => Ok(session_info.user_id),
      crate::domain::SessionStatus::Expired => Err(UserError::InvalidOperation("Session expired".to_string())),
    }
  }

  /// List all sessions
  #[instrument(skip(self))]
  pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>, UserError> {
    self.session_manager.list_sessions().await
  }

  /// List non-expired sessions (Active and Expiring)
  #[instrument(skip(self))]
  pub async fn list_non_expired_sessions(&self) -> Result<Vec<SessionInfo>, UserError> {
    self.session_manager.list_non_expired_sessions().await
  }

  /// Get session by session_id
  #[instrument(skip(self), fields(session_id = %session_id))]
  pub async fn get_session(&self, session_id: SessionId) -> Result<Option<SessionInfo>, UserError> {
    match self.session_manager.get_session(session_id).await {
      Ok(info) => Ok(Some(info)),
      Err(UserError::NotFound) => Ok(None),
      Err(e) => Err(e),
    }
  }

  /// Logout (delete session)
  #[instrument(skip(self), fields(session_id = %session_id))]
  pub async fn logout(&self, session_id: SessionId) -> Result<(), UserError> {
    self.session_manager.delete_session(session_id).await?;
    info!("Session {} logged out", session_id);
    Ok(())
  }
}
