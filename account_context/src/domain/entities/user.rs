use sqlx::Row;
use uuid::Uuid;

use crate::domain::valueobjects::{NickName, SaltedPassword};

#[derive(Debug)]
pub struct User {
  uuid: Uuid,
  nickname: NickName,
  salted_password: SaltedPassword,
  // TODO: add field to indicate password change deadline
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for User {
  fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
    Ok(User {
      uuid: row.try_get("uuid")?,
      nickname: NickName::from(row.try_get::<String, _>("nickname")?),
      salted_password: SaltedPassword::from_string(row.try_get::<String, _>("salted_password")?),
    })
  }
}

impl User {
  pub fn new(uuid: Uuid, nickname: impl Into<NickName>, salted_password: SaltedPassword) -> Self {
    Self {
      uuid,
      nickname: nickname.into(),
      salted_password,
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
}
