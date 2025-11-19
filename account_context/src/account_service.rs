use tracing::{error, info, instrument};
use uuid::Uuid;

use crate::config::Config;
use crate::domain::valueobjects::{AccountId, SessionId};
use crate::domain::{
  Account, AccountFactory, AccountManager, AccountRepository, NickName, SessionInfo, SessionManager,
  SessionRepository,
};
use crate::error::AccountError;

pub struct AccountService {
  account_manager: AccountManager,
  session_manager: SessionManager,
}

/// Result of resetting a password
pub struct ResetPasswordResult {
  pub uuid: String,
  pub nickname: Option<String>,
  pub password: String,
}

impl AccountService {
  pub fn new(
    config: &Config, account_repository: Box<dyn AccountRepository>, session_repository: Box<dyn SessionRepository>,
    account_factory: AccountFactory,
  ) -> Self {
    let account_manager = AccountManager::new(account_repository, account_factory);
    let session_manager = SessionManager::new(session_repository, config.session_duration_hours);
    Self {
      account_manager,
      session_manager,
    }
  }

  /// Create a new account with a randomly generated password
  #[instrument(skip(self), fields(nickname = nickname_str))]
  pub async fn create_account(&self, nickname_str: &str) -> Result<(Uuid, String), AccountError> {
    let nickname = NickName::from(nickname_str);
    let result = self.account_manager.create_account(&nickname).await;
    match &result {
      Ok(_) => info!("Successfully created account: {}", nickname_str),
      Err(e) => error!("Failed to create account {}: {:?}", nickname_str, e),
    }
    result.map(|(uuid, p)| (uuid, p.into_string()))
  }

  /// Get account by UUID
  #[instrument(skip(self), fields(uuid = %uuid))]
  pub async fn get_account_by_uuid(&self, uuid: Uuid) -> Result<Option<Account>, AccountError> {
    let result = self.account_manager.get_account_by_uuid(uuid).await;
    if let Err(e) = &result {
      error!("Error getting account by UUID {}: {:?}", uuid, e);
    }
    result
  }

  /// Get account by nickname
  #[instrument(skip(self), fields(nickname = nickname_str))]
  pub async fn get_account_by_nickname(&self, nickname_str: &str) -> Result<Option<Account>, AccountError> {
    let nickname = NickName::from(nickname_str);
    let result = self.account_manager.get_account_by_nickname(&nickname).await;
    if let Err(e) = &result {
      error!("Error getting account by nickname {}: {:?}", nickname_str, e);
    }
    result
  }

  /// List all accounts
  #[instrument(skip(self))]
  pub async fn list_accounts(&self) -> Result<Vec<Account>, AccountError> {
    let result = self.account_manager.list_accounts().await;
    if let Err(ref e) = result {
      error!("Error listing accounts: {:?}", e);
    }
    result
  }

  /// Update account nickname
  #[instrument(skip(self), fields(uuid = %uuid, new_nickname = %new_nickname))]
  pub async fn update_account_nickname(&self, uuid: Uuid, new_nickname: &NickName) -> Result<(), AccountError> {
    let result = self.account_manager.update_account_nickname(uuid, new_nickname).await;
    match &result {
      Ok(_) => info!("Successfully updated nickname for UUID: {}", uuid),
      Err(e) => error!("Failed to update nickname for UUID {}: {:?}", uuid, e),
    }
    result
  }

  /// Reset password for an account by UUID
  #[instrument(skip(self), fields(uuid = %uuid))]
  pub async fn reset_password_by_uuid(&self, uuid: Uuid) -> Result<String, AccountError> {
    let result = self.account_manager.reset_password_by_uuid(uuid).await;
    match &result {
      Ok(_) => info!("Successfully reset password for UUID: {}", uuid),
      Err(e) => error!("Failed to reset password for UUID {}: {:?}", uuid, e),
    }
    result.map(|p| p.into_string())
  }

  /// Reset password for an account by nickname
  #[instrument(skip(self), fields(nickname = nickname_str))]
  pub async fn reset_password_by_name(&self, nickname_str: &str) -> Result<(Uuid, String), AccountError> {
    let nickname = NickName::from(nickname_str);
    let result = self.account_manager.reset_password_by_name(&nickname).await;
    match &result {
      Ok(_) => info!("Successfully reset password for nickname: {}", nickname_str),
      Err(e) => error!("Failed to reset password for nickname {}: {:?}", nickname_str, e),
    }
    result.map(|(uuid, p)| (uuid, p.into_string()))
  }

  /// Reset password for an account by UUID or nickname
  /// Automatically detects whether the input is a UUID or nickname
  #[instrument(skip(self), fields(uuid_or_nickname = uuid_or_nickname))]
  pub async fn reset_password(&self, uuid_or_nickname: &str) -> Result<ResetPasswordResult, AccountError> {
    // Try to parse as UUID first
    if let Ok(uuid) = uuid_or_nickname.parse::<Uuid>() {
      let password = self.reset_password_by_uuid(uuid).await?;
      Ok(ResetPasswordResult {
        uuid: uuid.to_string(),
        nickname: None,
        password,
      })
    } else {
      // If not a valid UUID, treat it as a nickname
      let (uuid, password) = self.reset_password_by_name(uuid_or_nickname).await?;
      Ok(ResetPasswordResult {
        uuid: uuid.to_string(),
        nickname: Some(uuid_or_nickname.to_string()),
        password,
      })
    }
  }

  /// Delete account by UUID
  #[instrument(skip(self), fields(uuid = %uuid))]
  pub async fn delete_account(&self, uuid: Uuid) -> Result<(), AccountError> {
    let result = self.account_manager.delete_account(uuid).await;
    match &result {
      Ok(_) => info!("Successfully deleted account with UUID: {}", uuid),
      Err(e) => error!("Failed to delete account with UUID {}: {:?}", uuid, e),
    }
    result
  }

  /// Delete account by nickname
  #[instrument(skip(self), fields(nickname = nickname_str))]
  pub async fn delete_account_by_nickname(&self, nickname_str: &str) -> Result<(), AccountError> {
    let nickname = NickName::from(nickname_str);
    let result = self.account_manager.delete_account_by_nickname(&nickname).await;
    match &result {
      Ok(_) => info!("Successfully deleted account with nickname: {}", nickname_str),
      Err(e) => error!("Failed to delete account with nickname {}: {:?}", nickname_str, e),
    }
    result
  }

  /// Login with nickname and password
  /// Returns the session ID
  #[instrument(skip(self), fields(nickname = nickname_str))]
  pub async fn login(&self, nickname_str: &str, password_str: &str) -> Result<SessionId, AccountError> {
    use crate::domain::valueobjects::RawPassword;
    let nickname = NickName::from(nickname_str);
    let password = RawPassword::from(password_str);
    let account = self.account_manager.login(&nickname, &password).await?;

    let account_id = AccountId::from(account.uuid());

    // Set existing sessions to expire in 1 minute (grace period)
    // Uses LEAST() to ensure we don't extend sessions that expire sooner
    let grace_period_expires_at = chrono::Utc::now() + chrono::Duration::minutes(1);
    let updated_count = self
      .session_manager
      .set_expiration_for_account_sessions(account_id, grace_period_expires_at)
      .await?;
    if updated_count > 0 {
      info!(
        "Set {} existing session(s) to expire in 1 minute for account {}",
        updated_count, account_id
      );
    }

    // Create a new session for the logged-in account
    let session_id = self.session_manager.create_session(account_id).await?;

    info!(
      "Successfully logged in with nickname: {}, session_id: {}",
      nickname_str, session_id
    );

    Ok(session_id)
  }

  /// List all sessions
  #[instrument(skip(self))]
  pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>, AccountError> {
    let result = self.session_manager.list_sessions().await;
    if let Err(ref e) = result {
      error!("Error listing sessions: {:?}", e);
    }
    result
  }

  /// List non-expired sessions (Active and Expiring)
  #[instrument(skip(self))]
  pub async fn list_non_expired_sessions(&self) -> Result<Vec<SessionInfo>, AccountError> {
    let result = self.session_manager.list_non_expired_sessions().await;
    if let Err(ref e) = result {
      error!("Error listing non-expired sessions: {:?}", e);
    }
    result
  }

  /// Get session by session_id
  #[instrument(skip(self), fields(session_id = %session_id))]
  pub async fn get_session(&self, session_id: SessionId) -> Result<Option<SessionInfo>, AccountError> {
    let result = self.session_manager.get_session(session_id).await;
    if let Err(ref e) = result {
      error!("Error getting session {}: {:?}", session_id, e);
    }
    result
  }
}
