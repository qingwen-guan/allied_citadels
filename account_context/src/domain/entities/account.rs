use sqlx::Row;
use uuid::Uuid;

use crate::domain::valueobjects::{NickName, SaltedPassword};

#[derive(Debug)]
pub struct Account {
  uuid: Uuid,
  nickname: NickName,
  salted_password: SaltedPassword,
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for Account {
  fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
    Ok(Account {
      uuid: row.try_get("uuid")?,
      nickname: NickName::from(row.try_get::<String, _>("nickname")?),
      salted_password: SaltedPassword::from_string(row.try_get::<String, _>("salted_password")?),
    })
  }
}

impl Account {
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
