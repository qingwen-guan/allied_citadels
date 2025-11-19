use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio_postgres::Row;
use uuid::Uuid;

const PASSWORD_SALT: &str = "CITY_OF_GOLDEN_HORSES";

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
  uuid: Uuid,
  nickname: String,
  salted_password: String,
}

impl From<Row> for User {
  fn from(row: Row) -> Self {
    Self {
      uuid: row.get("uuid"),
      nickname: row.get("nickname"),
      salted_password: row.get("salted_password"),
    }
  }
}

impl User {
  pub fn new(uuid: Uuid, nickname: &str, password: &str) -> Self {
    let salted_password = format!("{}{}", password, PASSWORD_SALT);
    let hash = Sha256::digest(salted_password.as_bytes());
    let salted_password = hex::encode(hash);
    Self {
      uuid,
      nickname: nickname.to_string(),
      salted_password,
    }
  }

  pub fn uuid(&self) -> Uuid {
    self.uuid
  }

  pub fn nickname(&self) -> &str {
    &self.nickname
  }

  pub fn salted_password(&self) -> &str {
    &self.salted_password
  }
}
