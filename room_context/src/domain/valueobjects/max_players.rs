use std::fmt;

use crate::error::RoomError;

/// MaxPlayers - value object for maximum number of players to provide type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaxPlayers {
  Four,
  Six,
}

impl MaxPlayers {
  pub fn new(value: usize) -> Result<Self, RoomError> {
    match value {
      4 => Ok(MaxPlayers::Four),
      6 => Ok(MaxPlayers::Six),
      _ => Err(RoomError::InvalidMaxPlayers),
    }
  }

  pub fn value(&self) -> usize {
    match self {
      MaxPlayers::Four => 4,
      MaxPlayers::Six => 6,
    }
  }

  pub fn four() -> Self {
    MaxPlayers::Four
  }

  pub fn six() -> Self {
    MaxPlayers::Six
  }
}

impl fmt::Display for MaxPlayers {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.value())
  }
}

impl TryFrom<usize> for MaxPlayers {
  type Error = RoomError;

  fn try_from(value: usize) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<MaxPlayers> for usize {
  fn from(value: MaxPlayers) -> Self {
    value.value()
  }
}

// Implement sqlx traits to allow direct binding
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for MaxPlayers {
  fn encode_by_ref(
    &self, buf: &mut sqlx::postgres::PgArgumentBuffer,
  ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
    let val: i32 = self.value() as i32;
    <i32 as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&val, buf)
  }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for MaxPlayers {
  fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
    let num = <i32 as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
    MaxPlayers::new(num as usize).map_err(|e| e.into())
  }
}

impl sqlx::Type<sqlx::Postgres> for MaxPlayers {
  fn type_info() -> sqlx::postgres::PgTypeInfo {
    <i32 as sqlx::Type<sqlx::Postgres>>::type_info()
  }
}
