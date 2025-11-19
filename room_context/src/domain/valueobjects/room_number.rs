use std::fmt;

/// RoomNumber - value object for room number to provide type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RoomNumber(u32);

impl RoomNumber {
  pub fn new(value: u32) -> Self {
    Self(value)
  }

  pub fn value(&self) -> u32 {
    self.0
  }
}

impl fmt::Display for RoomNumber {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<u32> for RoomNumber {
  fn from(value: u32) -> Self {
    Self::new(value)
  }
}

impl From<RoomNumber> for u32 {
  fn from(value: RoomNumber) -> Self {
    value.value()
  }
}

// Implement sqlx traits to allow direct binding
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for RoomNumber {
  fn encode_by_ref(
    &self, buf: &mut sqlx::postgres::PgArgumentBuffer,
  ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
    let val: i32 = self.0 as i32;
    <i32 as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&val, buf)
  }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for RoomNumber {
  fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
    let num = <i32 as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
    Ok(RoomNumber::from(num as u32))
  }
}

impl sqlx::Type<sqlx::Postgres> for RoomNumber {
  fn type_info() -> sqlx::postgres::PgTypeInfo {
    <i32 as sqlx::Type<sqlx::Postgres>>::type_info()
  }
}
