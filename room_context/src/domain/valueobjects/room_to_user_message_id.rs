use std::fmt;

use uuid::Uuid;

/// RoomToUserMessageId - value object for room to user message UUID to provide type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RoomToUserMessageId(Uuid);

impl RoomToUserMessageId {
  pub fn new(value: Uuid) -> Self {
    Self(value)
  }

  pub fn value(&self) -> Uuid {
    self.0
  }
}

impl fmt::Display for RoomToUserMessageId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<Uuid> for RoomToUserMessageId {
  fn from(value: Uuid) -> Self {
    Self::new(value)
  }
}

impl From<RoomToUserMessageId> for Uuid {
  fn from(value: RoomToUserMessageId) -> Self {
    value.value()
  }
}

// Implement sqlx traits to allow direct binding
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for RoomToUserMessageId {
  fn encode_by_ref(
    &self, buf: &mut sqlx::postgres::PgArgumentBuffer,
  ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
    self.0.encode_by_ref(buf)
  }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for RoomToUserMessageId {
  fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
    let uuid = Uuid::decode(value)?;
    Ok(RoomToUserMessageId::from(uuid))
  }
}

impl sqlx::Type<sqlx::Postgres> for RoomToUserMessageId {
  fn type_info() -> sqlx::postgres::PgTypeInfo {
    <Uuid as sqlx::Type<sqlx::Postgres>>::type_info()
  }
}
