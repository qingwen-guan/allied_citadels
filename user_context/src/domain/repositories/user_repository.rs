use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::User;
use crate::domain::valueobjects::{NickName, SaltedPassword};
use crate::error::UserError;

/// UserRepository trait - interface for user data access
#[async_trait]
pub trait UserRepository: Send + Sync {
  async fn find_by_nickname(&self, nickname: &NickName) -> Result<User, UserError>;
  async fn find_uuid_by_nickname(&self, nickname: &NickName) -> Result<Uuid, UserError>;
  async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<User>, UserError>;
  async fn find_all(&self) -> Result<Vec<User>, UserError>;
  async fn create(&self, user: &User) -> Result<(), UserError>;
  async fn update_nickname(&self, uuid: Uuid, new_nickname: &str) -> Result<bool, UserError>;
  async fn update_password(&self, uuid: Uuid, salted_password: &SaltedPassword) -> Result<bool, UserError>;
  async fn delete(&self, uuid: Uuid) -> Result<bool, UserError>;
  async fn exists_by_nickname(&self, nickname: &NickName) -> Result<bool, UserError>;
  async fn exists_by_nickname_excluding(&self, nickname: &NickName, exclude_uuid: Uuid) -> Result<bool, UserError>;
}
