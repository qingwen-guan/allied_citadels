use std::fmt;

use uuid::Uuid;

/// UserId - value object for user UUID to provide type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(Uuid);

impl UserId {
  pub fn new(value: Uuid) -> Self {
    Self(value)
  }
}

impl fmt::Display for UserId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<Uuid> for UserId {
  fn from(value: Uuid) -> Self {
    Self::new(value)
  }
}

impl From<UserId> for Uuid {
  fn from(value: UserId) -> Self {
    value.0
  }
}

// Implement sqlx traits to allow direct binding
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for UserId {
  fn encode_by_ref(
    &self, buf: &mut sqlx::postgres::PgArgumentBuffer,
  ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
    self.0.encode_by_ref(buf)
  }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for UserId {
  fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
    let uuid = Uuid::decode(value)?;
    Ok(UserId::from(uuid))
  }
}

impl sqlx::Type<sqlx::Postgres> for UserId {
  fn type_info() -> sqlx::postgres::PgTypeInfo {
    <Uuid as sqlx::Type<sqlx::Postgres>>::type_info()
  }
}
