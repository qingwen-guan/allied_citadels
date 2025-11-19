use uuid::Uuid;

use crate::domain::entities::Account;
use crate::domain::factories::AccountFactory;
use crate::domain::repositories::AccountRepository;
use crate::domain::valueobjects::{NickName, RawPassword, SaltedPassword};
use crate::error::AccountError;

pub struct AccountManager {
  account_repository: Box<dyn AccountRepository>,
  account_factory: AccountFactory,
}

impl AccountManager {
  pub fn new(account_repository: Box<dyn AccountRepository>, account_factory: AccountFactory) -> Self {
    Self {
      account_repository,
      account_factory,
    }
  }

  /// Create a new account with a randomly generated password
  pub async fn create_account(&self, nickname: &NickName) -> Result<(Uuid, RawPassword), AccountError> {
    // Check if nickname already exists
    if self.account_repository.exists_by_nickname(nickname).await? {
      return Err(AccountError::NicknameExists);
    }

    // Use cryptographically secure random number generation
    let password = RawPassword::generate_random_default(6);
    let account = self.account_factory.create(nickname, &password);

    self.account_repository.create(&account).await?;

    Ok((account.uuid(), password))
  }

  /// Get account by UUID
  pub async fn get_account_by_uuid(&self, uuid: Uuid) -> Result<Option<Account>, AccountError> {
    self.account_repository.find_by_uuid(uuid).await
  }

  /// Get account by nickname
  pub async fn get_account_by_nickname(&self, nickname: &NickName) -> Result<Option<Account>, AccountError> {
    self
      .account_repository
      .find_by_nickname(nickname)
      .await
      .map(Some)
      .or_else(|e| match e {
        AccountError::NotFound => Ok(None),
        e => Err(e),
      })
  }

  /// List all accounts
  pub async fn list_accounts(&self) -> Result<Vec<Account>, AccountError> {
    self.account_repository.find_all().await
  }

  /// Update account nickname
  pub async fn update_account_nickname(&self, uuid: Uuid, new_nickname: &NickName) -> Result<(), AccountError> {
    // Check if new nickname already exists for a different account using optimized query
    if self
      .account_repository
      .exists_by_nickname_excluding(new_nickname, uuid)
      .await?
    {
      return Err(AccountError::NicknameExists);
    }

    let updated = self
      .account_repository
      .update_nickname(uuid, new_nickname.as_str())
      .await?;

    if !updated {
      return Err(AccountError::NotFound);
    }

    Ok(())
  }

  /// Generate a new random password and create its salted hash
  fn generate_salted_password(&self) -> (RawPassword, SaltedPassword) {
    let password = RawPassword::generate_random_default(6);
    let salted_password = SaltedPassword::new(&password, self.account_factory.password_salt());
    (password, salted_password)
  }

  /// Update password for an account by UUID
  async fn update_password_for_uuid(&self, uuid: Uuid, salted_password: &SaltedPassword) -> Result<(), AccountError> {
    let is_updated = self.account_repository.update_password(uuid, salted_password).await?;
    if !is_updated {
      return Err(AccountError::NotFound);
    }
    Ok(())
  }

  /// Reset password for an account by UUID
  pub async fn reset_password_by_uuid(&self, uuid: Uuid) -> Result<RawPassword, AccountError> {
    let (password, salted_password) = self.generate_salted_password();
    self.update_password_for_uuid(uuid, &salted_password).await?;
    Ok(password)
  }

  /// Reset password for an account by nickname
  pub async fn reset_password_by_name(&self, nickname: &NickName) -> Result<(Uuid, RawPassword), AccountError> {
    let uuid = self.account_repository.find_uuid_by_nickname(nickname).await?;
    let password = self.reset_password_by_uuid(uuid).await?;
    Ok((uuid, password))
  }

  /// Delete account by UUID
  pub async fn delete_account(&self, uuid: Uuid) -> Result<(), AccountError> {
    let is_deleted = self.account_repository.delete(uuid).await?;
    if !is_deleted {
      return Err(AccountError::NotFound);
    }
    Ok(())
  }

  /// Delete account by nickname
  pub async fn delete_account_by_nickname(&self, nickname: &NickName) -> Result<(), AccountError> {
    let uuid = self.account_repository.find_uuid_by_nickname(nickname).await?;
    self.delete_account(uuid).await
  }

  /// Verify login credentials (nickname and password)
  /// Returns the account if credentials are valid
  pub async fn login(&self, nickname: &NickName, password: &RawPassword) -> Result<Account, AccountError> {
    let account = self.get_account_by_nickname(nickname).await?;
    let account = account.ok_or(AccountError::NotFound)?;

    // Hash the provided password with the salt
    let provided_salted_password = SaltedPassword::new(password, self.account_factory.password_salt());

    // Compare with stored salted password
    if provided_salted_password.as_str() != account.salted_password().as_str() {
      return Err(AccountError::InvalidCredentials);
    }

    Ok(account)
  }
}
