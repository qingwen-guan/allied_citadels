use std::fmt;

use uuid::Uuid;

/// UserToRoomMessageId - value object for user to room message UUID to provide type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserToRoomMessageId(Uuid);

impl UserToRoomMessageId {
  pub fn new(value: Uuid) -> Self {
    Self(value)
  }

  pub fn value(&self) -> Uuid {
    self.0
  }
}

impl fmt::Display for UserToRoomMessageId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<Uuid> for UserToRoomMessageId {
  fn from(value: Uuid) -> Self {
    Self::new(value)
  }
}

impl From<UserToRoomMessageId> for Uuid {
  fn from(value: UserToRoomMessageId) -> Self {
    value.value()
  }
}

// Implement sqlx traits to allow direct binding
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for UserToRoomMessageId {
  fn encode_by_ref(
    &self, buf: &mut sqlx::postgres::PgArgumentBuffer,
  ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
    self.0.encode_by_ref(buf)
  }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for UserToRoomMessageId {
  fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
    let uuid = Uuid::decode(value)?;
    Ok(UserToRoomMessageId::from(uuid))
  }
}

impl sqlx::Type<sqlx::Postgres> for UserToRoomMessageId {
  fn type_info() -> sqlx::postgres::PgTypeInfo {
    <Uuid as sqlx::Type<sqlx::Postgres>>::type_info()
  }
}
