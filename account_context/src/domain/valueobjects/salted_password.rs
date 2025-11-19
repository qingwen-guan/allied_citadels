use std::fmt;

use serde::{Deserialize, Serialize};

use super::{RawPassword, Salt};

/// SaltedPassword - newtype wrapper for String to provide type safety
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaltedPassword(String);

impl SaltedPassword {
  pub fn from_string(value: String) -> Self {
    Self(value)
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }

  pub fn into_string(self) -> String {
    self.0
  }

  /// Create a new SaltedPassword by hashing a password with the given salt
  pub fn new(password: &RawPassword, salt: &Salt) -> Self {
    use sha2::{Digest, Sha256};
    let salted_password = format!("{}{}", password.as_str(), salt.as_str());
    let hash = Sha256::digest(salted_password.as_bytes());
    Self(hex::encode(hash))
  }
}

impl From<SaltedPassword> for String {
  fn from(value: SaltedPassword) -> Self {
    value.0
  }
}

// For sqlx::FromRow
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for SaltedPassword {
  fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
    let s = <String as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
    Ok(SaltedPassword::from_string(s))
  }
}

impl sqlx::Type<sqlx::Postgres> for SaltedPassword {
  fn type_info() -> sqlx::postgres::PgTypeInfo {
    <String as sqlx::Type<sqlx::Postgres>>::type_info()
  }
}

impl fmt::Debug for SaltedPassword {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}
