use sqlx::PgPool;

use crate::domain::valueobjects::UserId;
use crate::domain::{NickName, SaltedPassword, User, UserRepository};
use crate::errors::UserError;

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
    let mut users = sqlx::query_as::<_, User>(
      "SELECT uuid, nickname, salted_password, password_change_deadline FROM \"user\" WHERE nickname = $1 LIMIT 2",
    )
    .bind(nickname.as_str())
    .fetch_all(&self.pool)
    .await?;

    match users.len() {
      0 => Err(UserError::NotFound),
      1 => Ok(users.swap_remove(0)),
      _ => Err(UserError::DuplicateNickname),
    }
  }

  async fn find_id_by_nickname(&self, nickname: &NickName) -> Result<UserId, UserError> {
    let mut ids: Vec<UserId> =
      sqlx::query_scalar::<_, UserId>("SELECT uuid FROM \"user\" WHERE nickname = $1 LIMIT 2")
        .bind(nickname.as_str())
        .fetch_all(&self.pool)
        .await?;

    match ids.len() {
      0 => Err(UserError::NotFound),
      1 => Ok(ids.swap_remove(0)),
      _ => Err(UserError::DuplicateNickname),
    }
  }

  async fn find_by_id(&self, id: UserId) -> Result<Option<User>, UserError> {
    let user = sqlx::query_as::<_, User>(
      "SELECT uuid, nickname, salted_password, password_change_deadline FROM \"user\" WHERE uuid = $1",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;
    Ok(user)
  }

  async fn find_all(&self) -> Result<Vec<User>, UserError> {
    let users = sqlx::query_as::<_, User>(
      "SELECT uuid, nickname, salted_password, password_change_deadline FROM \"user\" ORDER BY nickname",
    )
    .fetch_all(&self.pool)
    .await?;
    Ok(users)
  }

  async fn create(&self, user: &User) -> Result<(), UserError> {
    sqlx::query(
      "INSERT INTO \"user\" (uuid, nickname, salted_password, password_change_deadline) VALUES ($1, $2, $3, $4)",
    )
    .bind(user.id())
    .bind(user.nickname().as_str())
    .bind(user.salted_password().as_str())
    .bind(user.password_change_deadline())
    .execute(&self.pool)
    .await?;
    Ok(())
  }

  async fn update_nickname(&self, id: UserId, new_nickname: &NickName) -> Result<bool, UserError> {
    let rows_affected = sqlx::query("UPDATE \"user\" SET nickname = $1 WHERE uuid = $2")
      .bind(new_nickname.as_str())
      .bind(id)
      .execute(&self.pool)
      .await?
      .rows_affected();
    Ok(rows_affected > 0)
  }

  async fn update_password(&self, id: UserId, salted_password: &SaltedPassword) -> Result<bool, UserError> {
    let rows_affected = sqlx::query("UPDATE \"user\" SET salted_password = $1 WHERE uuid = $2")
      .bind(salted_password.as_str())
      .bind(id)
      .execute(&self.pool)
      .await?
      .rows_affected();
    Ok(rows_affected > 0)
  }

  async fn delete(&self, id: UserId) -> Result<bool, UserError> {
    let rows_affected = sqlx::query("DELETE FROM \"user\" WHERE uuid = $1")
      .bind(id)
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

  async fn exists_by_nickname_excluding(&self, nickname: &NickName, exclude_id: UserId) -> Result<bool, UserError> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM \"user\" WHERE nickname = $1 AND uuid != $2")
      .bind(nickname.as_str())
      .bind(exclude_id)
      .fetch_one(&self.pool)
      .await?;
    Ok(count > 0)
  }
}
