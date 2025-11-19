use std::fmt;

use uuid::Uuid;

/// AccountToRoomMessageId - value object for account to room message UUID to provide type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccountToRoomMessageId(Uuid);

impl AccountToRoomMessageId {
  pub fn new(value: Uuid) -> Self {
    Self(value)
  }

  pub fn value(&self) -> Uuid {
    self.0
  }
}

impl fmt::Display for AccountToRoomMessageId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<Uuid> for AccountToRoomMessageId {
  fn from(value: Uuid) -> Self {
    Self::new(value)
  }
}

impl From<AccountToRoomMessageId> for Uuid {
  fn from(value: AccountToRoomMessageId) -> Self {
    value.value()
  }
}

// Implement sqlx traits to allow direct binding
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for AccountToRoomMessageId {
  fn encode_by_ref(
    &self, buf: &mut sqlx::postgres::PgArgumentBuffer,
  ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
    self.0.encode_by_ref(buf)
  }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for AccountToRoomMessageId {
  fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
    let uuid = Uuid::decode(value)?;
    Ok(AccountToRoomMessageId::from(uuid))
  }
}

impl sqlx::Type<sqlx::Postgres> for AccountToRoomMessageId {
  fn type_info() -> sqlx::postgres::PgTypeInfo {
    <Uuid as sqlx::Type<sqlx::Postgres>>::type_info()
  }
}
