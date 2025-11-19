use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::Account;
use crate::domain::valueobjects::{NickName, SaltedPassword};
use crate::error::AccountError;

/// AccountRepository trait - interface for account data access
#[async_trait]
pub trait AccountRepository: Send + Sync {
  async fn find_by_nickname(&self, nickname: &NickName) -> Result<Account, AccountError>;
  async fn find_uuid_by_nickname(&self, nickname: &NickName) -> Result<Uuid, AccountError>;
  async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<Account>, AccountError>;
  async fn find_all(&self) -> Result<Vec<Account>, AccountError>;
  async fn create(&self, account: &Account) -> Result<(), AccountError>;
  async fn update_nickname(&self, uuid: Uuid, new_nickname: &str) -> Result<bool, AccountError>;
  async fn update_password(&self, uuid: Uuid, salted_password: &SaltedPassword) -> Result<bool, AccountError>;
  async fn delete(&self, uuid: Uuid) -> Result<bool, AccountError>;
  async fn exists_by_nickname(&self, nickname: &NickName) -> Result<bool, AccountError>;
  async fn exists_by_nickname_excluding(&self, nickname: &NickName, exclude_uuid: Uuid)
  -> Result<bool, AccountError>;
}
