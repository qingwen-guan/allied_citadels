//! UserService - Application Service Layer
//!
//! IMPORTANT: This service layer MUST NOT expose domain types in its public API.
//! All parameters and return types must use primitive types (String, bool, etc.)
//! or service-specific response structs. Domain types (UserId, SessionId, User, etc.)
//! should only be used internally and converted at the boundary.

use tracing::{error, info, instrument};

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

/// Response for creating a new user
/// NOTE: Uses String instead of domain types per service layer rules
#[derive(Debug, Clone)]
pub struct CreateUserResponse {
  pub user_id: String,
  pub password: String,
}

/// Response for resetting a password
#[allow(dead_code)]
pub struct ResetPasswordResponse {
  pub user_id: String,
  pub nickname: Option<String>,
  pub password: String,
}

/// Response for logging in
/// NOTE: Uses String instead of domain types (SessionId, UserId) per service layer rules
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginResponse {
  pub session_id: String,
  pub user_id: String,
}

/// User response struct
/// NOTE: Uses String instead of domain types (UserId, NickName) per service layer rules
#[derive(Debug, Clone)]
pub struct UserResponse {
  pub user_id: String,
  pub nickname: String,
}

/// Response for resetting password by name
/// NOTE: Uses String instead of domain types per service layer rules
#[derive(Debug, Clone)]
pub struct ResetPasswordByNameResponse {
  pub user_id: String,
  pub password: String,
}

/// Session information response struct
/// NOTE: Uses String instead of domain types (SessionId, UserId, SessionStatus) per service layer rules
#[derive(Debug, Clone)]
pub struct SessionInfoResponse {
  pub session_id: String,
  pub user_id: String,
  pub created_at: String,
  pub expires_at: String,
  pub is_expired: bool,
  pub status: String,
}

/// Response for renaming a user
/// NOTE: Uses String instead of domain types per service layer rules
#[derive(Debug, Clone)]
pub struct RenameUserResponse {
  pub user_id: String,
  pub old_nickname: String,
  pub new_nickname: String,
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
  /// NOTE: Returns String (user_id) instead of UserId per service layer rules
  #[instrument(skip(self), fields(nickname = nickname_str))]
  pub async fn create_user(&self, nickname_str: &str) -> Result<CreateUserResponse, UserError> {
    let nickname = NickName::from(nickname_str);
    let result = self.user_manager.create_user(&nickname).await;
    match &result {
      Ok(_) => info!("Successfully created user: {}", nickname_str),
      Err(e) => error!("Failed to create user {}: {:?}", nickname_str, e),
    }
    result.map(|(user_id, p)| CreateUserResponse {
      user_id: user_id.to_string(),
      password: p.into_string(),
    })
  }

  /// Get user by user_id (UUID string)
  /// NOTE: Takes String and returns UserResponse instead of domain types per service layer rules
  #[instrument(skip(self), fields(user_id = user_id_str))]
  pub async fn get_user_by_id(&self, user_id_str: &str) -> Result<Option<UserResponse>, UserError> {
    let user_id = parse_user_id(user_id_str)?;
    let result = self.user_manager.get_user_by_id(user_id).await;
    if let Err(e) = &result {
      error!("Error getting user by ID {}: {:?}", user_id_str, e);
    }
    Ok(result?.map(|user| UserResponse {
      user_id: user.id().to_string(),
      nickname: user.nickname().as_str().to_string(),
    }))
  }

  /// Get user by nickname
  /// NOTE: Returns UserResponse instead of User per service layer rules
  #[instrument(skip(self), fields(nickname = nickname_str))]
  pub async fn get_user_by_nickname(&self, nickname_str: &str) -> Result<UserResponse, UserError> {
    let nickname = NickName::from(nickname_str);
    let result = self.user_manager.get_user_by_nickname(&nickname).await;
    if let Err(e) = &result {
      error!("Error getting user by nickname {}: {:?}", nickname_str, e);
    }
    let user = result?.ok_or(UserError::NotFound)?;
    Ok(UserResponse {
      user_id: user.id().to_string(),
      nickname: user.nickname().as_str().to_string(),
    })
  }

  /// List all users
  /// NOTE: Returns Vec<UserResponse> instead of Vec<User> per service layer rules
  #[instrument(skip(self))]
  pub async fn list_users(&self) -> Result<Vec<UserResponse>, UserError> {
    let result = self.user_manager.list_users().await;
    if let Err(ref e) = result {
      error!("Error listing users: {:?}", e);
    }
    Ok(
      result?
        .into_iter()
        .map(|user| UserResponse {
          user_id: user.id().to_string(),
          nickname: user.nickname().as_str().to_string(),
        })
        .collect(),
    )
  }

  /// Update user nickname
  /// NOTE: Takes String (user_id) instead of UserId per service layer rules
  #[instrument(skip(self), fields(user_id = user_id_str, new_nickname = new_nickname_str))]
  pub async fn update_user_nickname(&self, user_id_str: &str, new_nickname_str: &str) -> Result<(), UserError> {
    let user_id = parse_user_id(user_id_str)?;
    let new_nickname = NickName::from(new_nickname_str);
    let result = self.user_manager.update_nickname(user_id, &new_nickname).await;
    match &result {
      Ok(_) => info!(
        "Successfully updated user nickname: {} -> {}",
        user_id_str, new_nickname_str
      ),
      Err(e) => error!(
        "Failed to update user nickname {} -> {}: {:?}",
        user_id_str, new_nickname_str, e
      ),
    }
    result
  }

  async fn rename_user_internal(
    &self, user: User, new_nickname: &NickName, new_nickname_str: &str,
  ) -> Result<RenameUserResponse, UserError> {
    let old_nickname = user.nickname().as_str().to_string();

    let update_nickname_result = self.user_manager.update_nickname(user.id(), new_nickname).await;
    match &update_nickname_result {
      Ok(_) => info!("Successfully renamed user: {} -> {}", old_nickname, new_nickname_str),
      Err(e) => error!(
        "Failed to rename user {} -> {}: {:?}",
        old_nickname, new_nickname_str, e
      ),
    }

    update_nickname_result.map(|_| RenameUserResponse {
      user_id: user.id().to_string(),
      old_nickname,
      new_nickname: new_nickname_str.to_string(),
    })
  }

  /// Rename user by UUID or current nickname
  /// NOTE: Accepts either a user_id (UUID string) or current nickname as identifier
  #[instrument(skip(self), fields(identifier = uuid_or_nickname, new_nickname = new_nickname_str))]
  pub async fn rename_user(
    &self, uuid_or_nickname: &str, new_nickname_str: &str,
  ) -> Result<Option<RenameUserResponse>, UserError> {
    let new_nickname = NickName::from(new_nickname_str);

    // Try to parse as UUID first
    if let Ok(user_id) = parse_user_id(uuid_or_nickname) {
      // Fetch current user to capture old nickname
      let user_opt = self.user_manager.get_user_by_id(user_id).await?;
      let Some(user) = user_opt else {
        return Err(UserError::NotFound);
      };

      self
        .rename_user_internal(user, &new_nickname, new_nickname_str)
        .await
        .map(Some)
    } else {
      // Treat identifier as current nickname
      let current_nickname = NickName::from(uuid_or_nickname);
      let user_opt = self.user_manager.get_user_by_nickname(&current_nickname).await?;
      let Some(user) = user_opt else {
        return Err(UserError::NotFound);
      };

      self
        .rename_user_internal(user, &new_nickname, new_nickname_str)
        .await
        .map(Some)
    }
  }

  /// Reset password for a user by user_id (UUID string)
  /// NOTE: Takes String instead of UserId per service layer rules
  #[instrument(skip(self), fields(user_id = user_id_str))]
  pub async fn reset_password_by_id(&self, user_id_str: &str) -> Result<String, UserError> {
    let user_id = parse_user_id(user_id_str)?;
    let result = self.user_manager.reset_password_by_id(user_id).await;
    match &result {
      Ok(_) => info!("Successfully reset password for user: {}", user_id_str),
      Err(e) => error!("Failed to reset password for user {}: {:?}", user_id_str, e),
    }
    result.map(|p| p.into_string())
  }

  /// Reset password for a user by nickname
  /// NOTE: Returns ResetPasswordByNameResponse instead of tuple with UserId per service layer rules
  #[instrument(skip(self), fields(nickname = nickname_str))]
  pub async fn reset_password_by_name(&self, nickname_str: &str) -> Result<ResetPasswordByNameResponse, UserError> {
    let nickname = NickName::from(nickname_str);
    let result = self.user_manager.reset_password_by_name(&nickname).await;
    match &result {
      Ok(_) => info!("Successfully reset password for user: {}", nickname_str),
      Err(e) => error!("Failed to reset password for user {}: {:?}", nickname_str, e),
    }
    result.map(|(user_id, p)| ResetPasswordByNameResponse {
      user_id: user_id.to_string(),
      password: p.into_string(),
    })
  }

  /// Delete user by user_id (UUID string)
  /// NOTE: Takes String instead of UserId per service layer rules
  #[instrument(skip(self), fields(user_id = user_id_str))]
  pub async fn delete_user(&self, user_id_str: &str) -> Result<(), UserError> {
    let user_id = parse_user_id(user_id_str)?;
    let result = self.user_manager.delete_user(user_id).await;
    match &result {
      Ok(_) => info!("Successfully deleted user: {}", user_id_str),
      Err(e) => error!("Failed to delete user {}: {:?}", user_id_str, e),
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
  /// NOTE: Returns LoginResponse with String fields instead of domain types per service layer rules
  #[instrument(skip(self), fields(nickname = nickname_str))]
  pub async fn login(&self, nickname_str: &str, password_str: &str) -> Result<LoginResponse, UserError> {
    use crate::domain::valueobjects::RawPassword;

    let nickname = NickName::from(nickname_str);
    let password = RawPassword::from(password_str);
    let user = self.user_manager.login(&nickname, &password).await?;

    let user_id = user.id();

    // Auto-expire all existing active sessions for this user before creating a new one
    let now: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
    let active_sessions = self.session_manager.list_active_sessions_by_user_id(user_id).await?;
    let sessions_ids: Vec<SessionId> = active_sessions.into_iter().map(|s| s.session_id).collect();

    if !sessions_ids.is_empty() {
      self
        .session_manager
        .set_expiration_for_sessions(&sessions_ids, now)
        .await?;
    }

    let session_id = self.session_manager.create_session(user_id).await?;

    info!("User {} logged in successfully, session: {}", nickname_str, session_id);
    Ok(LoginResponse {
      session_id: session_id.to_string(),
      user_id: user_id.to_string(),
    })
  }

  /// Verify session and get user ID
  /// NOTE: Takes String and returns String instead of domain types per service layer rules
  #[instrument(skip(self), fields(session_id = session_id_str))]
  pub async fn verify_session(&self, session_id_str: &str) -> Result<String, UserError> {
    let session_id = parse_session_id(session_id_str)?;
    let session_info = self.session_manager.get_session(session_id).await?;
    match session_info.status {
      crate::domain::SessionStatus::Active => Ok(session_info.user_id.to_string()),
      crate::domain::SessionStatus::Expired => Err(UserError::InvalidOperation("Session expired".to_string())),
    }
  }

  /// List all sessions
  /// NOTE: Returns Vec<SessionInfoResponse> instead of Vec<SessionInfo> per service layer rules
  #[instrument(skip(self))]
  pub async fn list_sessions(&self) -> Result<Vec<SessionInfoResponse>, UserError> {
    let sessions = self.session_manager.list_sessions().await?;
    Ok(sessions.into_iter().map(convert_session_info).collect())
  }

  /// List active (non-expired) sessions
  /// NOTE: Returns Vec<SessionInfoResponse> instead of Vec<SessionInfo> per service layer rules
  #[instrument(skip(self))]
  pub async fn list_active_sessions(&self) -> Result<Vec<SessionInfoResponse>, UserError> {
    let sessions = self.session_manager.list_active_sessions().await?;
    Ok(sessions.into_iter().map(convert_session_info).collect())
  }

  /// Get session by session_id
  /// NOTE: Takes String and returns SessionInfoResponse instead of domain types per service layer rules
  #[instrument(skip(self), fields(session_id = session_id_str))]
  pub async fn get_session(&self, session_id_str: &str) -> Result<Option<SessionInfoResponse>, UserError> {
    let session_id = parse_session_id(session_id_str)?;
    match self.session_manager.get_session(session_id).await {
      Ok(info) => Ok(Some(convert_session_info(info))),
      Err(UserError::NotFound) => Ok(None),
      Err(e) => Err(e),
    }
  }

  /// Logout (delete session)
  /// NOTE: Takes String instead of SessionId per service layer rules
  #[instrument(skip(self), fields(session_id = session_id_str))]
  pub async fn logout(&self, session_id_str: &str) -> Result<(), UserError> {
    let session_id = parse_session_id(session_id_str)?;
    self.session_manager.delete_session(session_id).await?;
    info!("Session {} logged out", session_id_str);
    Ok(())
  }
}

// Helper functions to convert between domain types and primitives at the boundary
// NOTE: These functions are used internally to convert between domain types and primitives
// Domain types should never leak into the public API

fn parse_user_id(user_id_str: &str) -> Result<UserId, UserError> {
  user_id_str
    .parse::<UserId>()
    .map_err(|_| UserError::InvalidOperation(format!("Invalid user_id format: {}", user_id_str)))
}

fn parse_session_id(session_id_str: &str) -> Result<SessionId, UserError> {
  session_id_str
    .parse::<SessionId>()
    .map_err(|_| UserError::InvalidOperation(format!("Invalid session_id format: {}", session_id_str)))
}

fn convert_session_info(info: SessionInfo) -> SessionInfoResponse {
  SessionInfoResponse {
    session_id: info.session_id.to_string(),
    user_id: info.user_id.to_string(),
    created_at: info.created_at.to_rfc3339(),
    expires_at: info.expires_at.to_rfc3339(),
    is_expired: info.is_expired,
    status: info.status.to_string(),
  }
}
