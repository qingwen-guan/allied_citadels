use std::fmt;

use uuid::Uuid;

/// RoomToAccountMessageId - value object for room to account message UUID to provide type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RoomToAccountMessageId(Uuid);

impl RoomToAccountMessageId {
  pub fn new(value: Uuid) -> Self {
    Self(value)
  }

  pub fn value(&self) -> Uuid {
    self.0
  }
}

impl fmt::Display for RoomToAccountMessageId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<Uuid> for RoomToAccountMessageId {
  fn from(value: Uuid) -> Self {
    Self::new(value)
  }
}

impl From<RoomToAccountMessageId> for Uuid {
  fn from(value: RoomToAccountMessageId) -> Self {
    value.value()
  }
}

// Implement sqlx traits to allow direct binding
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for RoomToAccountMessageId {
  fn encode_by_ref(
    &self, buf: &mut sqlx::postgres::PgArgumentBuffer,
  ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
    self.0.encode_by_ref(buf)
  }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for RoomToAccountMessageId {
  fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
    let uuid = Uuid::decode(value)?;
    Ok(RoomToAccountMessageId::from(uuid))
  }
}

impl sqlx::Type<sqlx::Postgres> for RoomToAccountMessageId {
  fn type_info() -> sqlx::postgres::PgTypeInfo {
    <Uuid as sqlx::Type<sqlx::Postgres>>::type_info()
  }
}
