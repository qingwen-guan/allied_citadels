use std::fmt;
use std::str::FromStr;

use uuid::Uuid;

/// SessionId - value object for session UUID to provide type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(Uuid);

impl SessionId {
  /// Create a new SessionId with a randomly generated UUID
  pub fn make() -> Self {
    Self(Uuid::new_v4())
  }
}

impl fmt::Display for SessionId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<Uuid> for SessionId {
  fn from(value: Uuid) -> Self {
    Self(value)
  }
}

impl From<SessionId> for Uuid {
  fn from(value: SessionId) -> Self {
    value.0
  }
}

impl FromStr for SessionId {
  type Err = uuid::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let uuid = s.parse::<Uuid>()?;
    Ok(SessionId::from(uuid))
  }
}

// Implement sqlx traits to allow direct binding
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for SessionId {
  fn encode_by_ref(
    &self, buf: &mut sqlx::postgres::PgArgumentBuffer,
  ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
    self.0.encode_by_ref(buf)
  }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for SessionId {
  fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
    let uuid = Uuid::decode(value)?;
    Ok(SessionId::from(uuid))
  }
}

impl sqlx::Type<sqlx::Postgres> for SessionId {
  fn type_info() -> sqlx::postgres::PgTypeInfo {
    <Uuid as sqlx::Type<sqlx::Postgres>>::type_info()
  }
}
