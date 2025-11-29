use async_trait::async_trait;

use crate::domain::entities::User;
use crate::domain::valueobjects::{NickName, SaltedPassword, UserId};
use crate::errors::UserError;

/// UserRepository trait - interface for user data access
#[async_trait]
pub trait UserRepository: Send + Sync {
  async fn find_by_nickname(&self, nickname: &NickName) -> Result<User, UserError>;
  async fn find_id_by_nickname(&self, nickname: &NickName) -> Result<UserId, UserError>;
  async fn find_by_id(&self, id: UserId) -> Result<Option<User>, UserError>;
  async fn find_all(&self) -> Result<Vec<User>, UserError>;
  async fn create(&self, user: &User) -> Result<(), UserError>;
  async fn update_nickname(&self, id: UserId, new_nickname: &NickName) -> Result<bool, UserError>;
  async fn update_password(&self, id: UserId, salted_password: &SaltedPassword) -> Result<bool, UserError>;
  async fn delete(&self, id: UserId) -> Result<bool, UserError>;
  async fn exists_by_nickname(&self, nickname: &NickName) -> Result<bool, UserError>;
  async fn exists_by_nickname_excluding(&self, nickname: &NickName, exclude_id: UserId) -> Result<bool, UserError>;
}
