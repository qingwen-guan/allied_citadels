use std::fmt;

use crate::errors::RoomError;

/// SeatNumber - value object for seat position in a room (0-based index)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SeatNumber(usize);

impl SeatNumber {
  pub fn new(value: usize) -> Result<Self, RoomError> {
    if value > 5 {
      return Err(RoomError::InvalidOperation(
        "Seat number must be between 0 and 5".to_string(),
      ));
    }
    Ok(Self(value))
  }

  pub fn value(&self) -> usize {
    self.0
  }

  pub fn zero() -> Self {
    Self(0)
  }
}

impl fmt::Display for SeatNumber {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<SeatNumber> for usize {
  fn from(value: SeatNumber) -> Self {
    value.value()
  }
}

// Implement sqlx traits to allow direct binding
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for SeatNumber {
  fn encode_by_ref(
    &self, buf: &mut sqlx::postgres::PgArgumentBuffer,
  ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
    let val: i16 = self.0 as i16;
    <i16 as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&val, buf)
  }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for SeatNumber {
  fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
    let num = <i16 as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
    SeatNumber::new(num as usize).map_err(|e| e.into())
  }
}

impl sqlx::Type<sqlx::Postgres> for SeatNumber {
  fn type_info() -> sqlx::postgres::PgTypeInfo {
    <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
  }
}
