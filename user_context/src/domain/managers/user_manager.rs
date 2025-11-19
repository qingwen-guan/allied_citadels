use uuid::Uuid;

use crate::domain::entities::User;
use crate::domain::factories::UserFactory;
use crate::domain::repositories::UserRepository;
use crate::domain::valueobjects::{NickName, RawPassword, SaltedPassword};
use crate::error::UserError;

pub struct UserManager {
  user_repository: Box<dyn UserRepository>,
  user_factory: UserFactory,
}

impl UserManager {
  pub fn new(user_repository: Box<dyn UserRepository>, user_factory: UserFactory) -> Self {
    Self {
      user_repository,
      user_factory,
    }
  }

  /// Create a new user with a randomly generated password
  pub async fn create_user(&self, nickname: &NickName) -> Result<(Uuid, RawPassword), UserError> {
    // Check if nickname already exists
    if self.user_repository.exists_by_nickname(nickname).await? {
      return Err(UserError::NicknameExists);
    }

    // Use cryptographically secure random number generation
    let password = RawPassword::generate_random_default(6);
    let user = self.user_factory.create(nickname, &password);

    self.user_repository.create(&user).await?;

    Ok((user.uuid(), password))
  }

  /// Get user by UUID
  pub async fn get_user_by_uuid(&self, uuid: Uuid) -> Result<Option<User>, UserError> {
    self.user_repository.find_by_uuid(uuid).await
  }

  /// Get user by nickname
  pub async fn get_user_by_nickname(&self, nickname: &NickName) -> Result<Option<User>, UserError> {
    self
      .user_repository
      .find_by_nickname(nickname)
      .await
      .map(Some)
      .or_else(|e| match e {
        UserError::NotFound => Ok(None),
        e => Err(e),
      })
  }

  /// List all users
  pub async fn list_users(&self) -> Result<Vec<User>, UserError> {
    self.user_repository.find_all().await
  }

  /// Update user nickname
  pub async fn update_user_nickname(&self, uuid: Uuid, new_nickname: &NickName) -> Result<(), UserError> {
    // Check if new nickname already exists (excluding current user)
    if self
      .user_repository
      .exists_by_nickname_excluding(new_nickname, uuid)
      .await?
    {
      return Err(UserError::NicknameExists);
    }

    // Update nickname
    self
      .user_repository
      .update_nickname(uuid, new_nickname.as_str())
      .await?;

    Ok(())
  }

  /// Reset password for a user by UUID
  pub async fn reset_password_by_uuid(&self, uuid: Uuid) -> Result<RawPassword, UserError> {
    // Check if user exists
    let _user = self
      .user_repository
      .find_by_uuid(uuid)
      .await?
      .ok_or(UserError::NotFound)?;

    // Generate new password
    let new_password = RawPassword::generate_random_default(6);
    let salted_password = SaltedPassword::new(&new_password, self.user_factory.password_salt());

    // Update password
    self.user_repository.update_password(uuid, &salted_password).await?;

    Ok(new_password)
  }

  /// Reset password for a user by nickname
  pub async fn reset_password_by_name(&self, nickname: &NickName) -> Result<(Uuid, RawPassword), UserError> {
    // Find user by nickname
    let user = self.user_repository.find_by_nickname(nickname).await?;

    // Generate new password
    let new_password = RawPassword::generate_random_default(6);
    let salted_password = SaltedPassword::new(&new_password, self.user_factory.password_salt());

    // Update password
    self
      .user_repository
      .update_password(user.uuid(), &salted_password)
      .await?;

    Ok((user.uuid(), new_password))
  }

  /// Delete user by UUID
  pub async fn delete_user(&self, uuid: Uuid) -> Result<(), UserError> {
    let deleted = self.user_repository.delete(uuid).await?;
    if !deleted {
      return Err(UserError::NotFound);
    }
    Ok(())
  }

  /// Delete user by nickname
  pub async fn delete_user_by_nickname(&self, nickname: &NickName) -> Result<(), UserError> {
    let user = self.user_repository.find_by_nickname(nickname).await?;
    self.delete_user(user.uuid()).await
  }

  /// Login with nickname and password
  /// Returns the user if credentials are valid
  pub async fn login(&self, nickname: &NickName, password: &RawPassword) -> Result<User, UserError> {
    // Find user by nickname
    let user = self.user_repository.find_by_nickname(nickname).await?;

    // Verify password
    let expected_salted_password = SaltedPassword::new(password, self.user_factory.password_salt());
    if user.salted_password() != &expected_salted_password {
      return Err(UserError::InvalidCredentials);
    }

    Ok(user)
  }
}
