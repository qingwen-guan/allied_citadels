use sqlx::Row;

use crate::domain::valueobjects::{NickName, SaltedPassword, UserId};

#[derive(Debug)]
pub struct User {
  id: UserId,
  nickname: NickName,
  salted_password: SaltedPassword,
  password_change_deadline: chrono::DateTime<chrono::Utc>,
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for User {
  fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
    Ok(User {
      id: row.try_get("uuid")?,
      nickname: NickName::from(row.try_get::<String, _>("nickname")?),
      salted_password: SaltedPassword::from_string(row.try_get::<String, _>("salted_password")?),
      password_change_deadline: row.try_get("password_change_deadline")?,
    })
  }
}

impl User {
  pub fn new(
    id: UserId, nickname: impl Into<NickName>, salted_password: SaltedPassword,
    password_change_deadline: chrono::DateTime<chrono::Utc>,
  ) -> Self {
    Self {
      id,
      nickname: nickname.into(),
      salted_password,
      password_change_deadline,
    }
  }

  pub fn id(&self) -> UserId {
    self.id
  }

  pub fn nickname(&self) -> &NickName {
    &self.nickname
  }

  pub fn salted_password(&self) -> &SaltedPassword {
    &self.salted_password
  }

  pub fn password_change_deadline(&self) -> chrono::DateTime<chrono::Utc> {
    self.password_change_deadline
  }
}
