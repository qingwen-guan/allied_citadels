use std::fmt;
use std::str::FromStr;

use crate::error::RoomError;

/// RoomState - value object representing the lifecycle status of a room
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoomState {
  Waiting,
  Playing,
  Finished,
  Closing, // closing means the room has less than 1 minutes before close
  Closed,
}

impl RoomState {
  pub const fn as_str(&self) -> &'static str {
    match self {
      RoomState::Waiting => "waiting",
      RoomState::Playing => "playing",
      RoomState::Finished => "finished",
      RoomState::Closing => "closing",
      RoomState::Closed => "closed",
    }
  }
}

impl fmt::Display for RoomState {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

impl From<RoomState> for String {
  fn from(state: RoomState) -> Self {
    state.as_str().to_string()
  }
}

impl TryFrom<&str> for RoomState {
  type Error = RoomError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    RoomState::from_str(value)
  }
}

impl FromStr for RoomState {
  type Err = RoomError;

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    match value.to_ascii_lowercase().as_str() {
      "waiting" => Ok(RoomState::Waiting),
      "playing" => Ok(RoomState::Playing),
      "finished" => Ok(RoomState::Finished),
      "closing" => Ok(RoomState::Closing),
      "closed" => Ok(RoomState::Closed),
      invalid => Err(RoomError::InvalidOperation(format!("Invalid room state: {invalid}"))),
    }
  }
}

// Implement sqlx traits to allow direct binding
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for RoomState {
  fn encode_by_ref(
    &self, buf: &mut sqlx::postgres::PgArgumentBuffer,
  ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
    let value: String = (*self).into();
    <String as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&value, buf)
  }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for RoomState {
  fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
    let raw = <String as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
    RoomState::from_str(&raw).map_err(|err| err.into())
  }
}

impl sqlx::Type<sqlx::Postgres> for RoomState {
  fn type_info() -> sqlx::postgres::PgTypeInfo {
    <String as sqlx::Type<sqlx::Postgres>>::type_info()
  }
}
