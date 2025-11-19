use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NickName, SaltedPassword, User, UserRepository};
use crate::error::UserError;

/// Access Control Layer - abstracts database operations to allow for future storage changes
pub struct PostgresUserRepository {
  pool: PgPool,
}

impl PostgresUserRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait::async_trait]
impl UserRepository for PostgresUserRepository {
  async fn find_by_nickname(&self, nickname: &NickName) -> Result<User, UserError> {
    let users =
      sqlx::query_as::<_, User>("SELECT uuid, nickname, salted_password FROM \"user\" WHERE nickname = $1 LIMIT 2")
        .bind(nickname.as_str())
        .fetch_all(&self.pool)
        .await?;

    match users.len() {
      0 => Err(UserError::NotFound),
      1 => Ok(users.into_iter().next().unwrap()),
      _ => Err(UserError::DuplicateNickname),
    }
  }

  async fn find_uuid_by_nickname(&self, nickname: &NickName) -> Result<Uuid, UserError> {
    let uuids: Vec<Uuid> = sqlx::query_scalar::<_, Uuid>("SELECT uuid FROM \"user\" WHERE nickname = $1 LIMIT 2")
      .bind(nickname.as_str())
      .fetch_all(&self.pool)
      .await?;

    match uuids.len() {
      0 => Err(UserError::NotFound),
      1 => Ok(uuids.into_iter().next().unwrap()),
      _ => Err(UserError::DuplicateNickname),
    }
  }

  async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<User>, UserError> {
    let user = sqlx::query_as::<_, User>("SELECT uuid, nickname, salted_password FROM \"user\" WHERE uuid = $1")
      .bind(uuid)
      .fetch_optional(&self.pool)
      .await?;
    Ok(user)
  }

  async fn find_all(&self) -> Result<Vec<User>, UserError> {
    let users = sqlx::query_as::<_, User>("SELECT uuid, nickname, salted_password FROM \"user\" ORDER BY nickname")
      .fetch_all(&self.pool)
      .await?;
    Ok(users)
  }

  async fn create(&self, user: &User) -> Result<(), UserError> {
    sqlx::query("INSERT INTO \"user\" (uuid, nickname, salted_password) VALUES ($1, $2, $3)")
      .bind(user.uuid())
      .bind(user.nickname().as_str())
      .bind(user.salted_password().as_str())
      .execute(&self.pool)
      .await?;
    Ok(())
  }

  async fn update_nickname(&self, uuid: Uuid, new_nickname: &str) -> Result<bool, UserError> {
    let rows_affected = sqlx::query("UPDATE \"user\" SET nickname = $1 WHERE uuid = $2")
      .bind(new_nickname)
      .bind(uuid)
      .execute(&self.pool)
      .await?
      .rows_affected();
    Ok(rows_affected > 0)
  }

  async fn update_password(&self, uuid: Uuid, salted_password: &SaltedPassword) -> Result<bool, UserError> {
    let rows_affected = sqlx::query("UPDATE \"user\" SET salted_password = $1 WHERE uuid = $2")
      .bind(salted_password.as_str())
      .bind(uuid)
      .execute(&self.pool)
      .await?
      .rows_affected();
    Ok(rows_affected > 0)
  }

  async fn delete(&self, uuid: Uuid) -> Result<bool, UserError> {
    let rows_affected = sqlx::query("DELETE FROM \"user\" WHERE uuid = $1")
      .bind(uuid)
      .execute(&self.pool)
      .await?
      .rows_affected();
    Ok(rows_affected > 0)
  }

  async fn exists_by_nickname(&self, nickname: &NickName) -> Result<bool, UserError> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM \"user\" WHERE nickname = $1")
      .bind(nickname.as_str())
      .fetch_one(&self.pool)
      .await?;
    Ok(count > 0)
  }

  async fn exists_by_nickname_excluding(&self, nickname: &NickName, exclude_uuid: Uuid) -> Result<bool, UserError> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM \"user\" WHERE nickname = $1 AND uuid != $2")
      .bind(nickname.as_str())
      .bind(exclude_uuid)
      .fetch_one(&self.pool)
      .await?;
    Ok(count > 0)
  }
}
