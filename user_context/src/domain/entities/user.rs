use sqlx::Row;
use uuid::Uuid;

use crate::domain::valueobjects::{NickName, SaltedPassword};

#[derive(Debug)]
pub struct User {
  uuid: Uuid,
  nickname: NickName,
  salted_password: SaltedPassword,
  password_change_deadline: Option<chrono::DateTime<chrono::Utc>>, // TODO: make it required
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for User {
  fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
    Ok(User {
      uuid: row.try_get("uuid")?,
      nickname: NickName::from(row.try_get::<String, _>("nickname")?),
      salted_password: SaltedPassword::from_string(row.try_get::<String, _>("salted_password")?),
      password_change_deadline: row.try_get("password_change_deadline").ok().flatten(),
    })
  }
}

impl User {
  pub fn new(uuid: Uuid, nickname: impl Into<NickName>, salted_password: SaltedPassword) -> Self {
    Self {
      uuid,
      nickname: nickname.into(),
      salted_password,
      password_change_deadline: None,
    }
  }

  pub fn uuid(&self) -> Uuid {
    self.uuid
  }

  pub fn nickname(&self) -> &NickName {
    &self.nickname
  }

  pub fn salted_password(&self) -> &SaltedPassword {
    &self.salted_password
  }

  pub fn password_change_deadline(&self) -> Option<chrono::DateTime<chrono::Utc>> {
    self.password_change_deadline
  }
}
