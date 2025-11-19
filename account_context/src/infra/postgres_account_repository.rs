use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{Account, AccountRepository, NickName, SaltedPassword};
use crate::error::AccountError;

/// Access Control Layer - abstracts database operations to allow for future storage changes
pub struct PostgresAccountRepository {
  pool: PgPool,
}

impl PostgresAccountRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait::async_trait]
impl AccountRepository for PostgresAccountRepository {
  async fn find_by_nickname(&self, nickname: &NickName) -> Result<Account, AccountError> {
    let accounts =
      sqlx::query_as::<_, Account>("SELECT uuid, nickname, salted_password FROM account WHERE nickname = $1 LIMIT 2")
        .bind(nickname.as_str())
        .fetch_all(&self.pool)
        .await?;

    match accounts.len() {
      0 => Err(AccountError::NotFound),
      1 => Ok(accounts.into_iter().next().unwrap()),
      _ => Err(AccountError::DuplicateNickname),
    }
  }

  async fn find_uuid_by_nickname(&self, nickname: &NickName) -> Result<Uuid, AccountError> {
    let uuids: Vec<Uuid> = sqlx::query_scalar::<_, Uuid>("SELECT uuid FROM account WHERE nickname = $1 LIMIT 2")
      .bind(nickname.as_str())
      .fetch_all(&self.pool)
      .await?;

    match uuids.len() {
      0 => Err(AccountError::NotFound),
      1 => Ok(uuids.into_iter().next().unwrap()),
      _ => Err(AccountError::DuplicateNickname),
    }
  }

  async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<Account>, AccountError> {
    let account = sqlx::query_as::<_, Account>("SELECT uuid, nickname, salted_password FROM account WHERE uuid = $1")
      .bind(uuid)
      .fetch_optional(&self.pool)
      .await?;

    Ok(account)
  }

  async fn find_all(&self) -> Result<Vec<Account>, AccountError> {
    let accounts =
      sqlx::query_as::<_, Account>("SELECT uuid, nickname, salted_password FROM account ORDER BY nickname")
        .fetch_all(&self.pool)
        .await?;

    Ok(accounts)
  }

  async fn create(&self, account: &Account) -> Result<(), AccountError> {
    sqlx::query("INSERT INTO account (uuid, nickname, salted_password) VALUES ($1, $2, $3)")
      .bind(account.uuid())
      .bind(account.nickname().as_str())
      .bind(account.salted_password().as_str())
      .execute(&self.pool)
      .await?;

    Ok(())
  }

  async fn update_nickname(&self, uuid: Uuid, new_nickname: &str) -> Result<bool, AccountError> {
    let rows_affected = sqlx::query("UPDATE account SET nickname = $1 WHERE uuid = $2")
      .bind(new_nickname)
      .bind(uuid)
      .execute(&self.pool)
      .await?
      .rows_affected();

    Ok(rows_affected > 0)
  }

  async fn update_password(&self, uuid: Uuid, salted_password: &SaltedPassword) -> Result<bool, AccountError> {
    let rows_affected = sqlx::query("UPDATE account SET salted_password = $1 WHERE uuid = $2")
      .bind(salted_password.as_str())
      .bind(uuid)
      .execute(&self.pool)
      .await?
      .rows_affected();

    Ok(rows_affected > 0)
  }

  async fn delete(&self, uuid: Uuid) -> Result<bool, AccountError> {
    let rows_affected = sqlx::query("DELETE FROM account WHERE uuid = $1")
      .bind(uuid)
      .execute(&self.pool)
      .await?
      .rows_affected();

    Ok(rows_affected > 0)
  }

  async fn exists_by_nickname(&self, nickname: &NickName) -> Result<bool, AccountError> {
    let existing: Option<Uuid> = sqlx::query_scalar::<_, Uuid>("SELECT uuid FROM account WHERE nickname = $1")
      .bind(nickname.as_str())
      .fetch_optional(&self.pool)
      .await?;

    Ok(existing.is_some())
  }

  async fn exists_by_nickname_excluding(
    &self, nickname: &NickName, exclude_uuid: Uuid,
  ) -> Result<bool, AccountError> {
    let existing: Option<Uuid> =
      sqlx::query_scalar::<_, Uuid>("SELECT uuid FROM account WHERE nickname = $1 AND uuid != $2")
        .bind(nickname.as_str())
        .bind(exclude_uuid)
        .fetch_optional(&self.pool)
        .await?;

    Ok(existing.is_some())
  }
}
