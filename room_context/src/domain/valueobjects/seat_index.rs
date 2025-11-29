use std::fmt;

use crate::errors::RoomError;

/// SeatIndex - value object for seat index (0-5) to provide type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SeatIndex(usize);

impl SeatIndex {
  pub fn new(value: usize) -> Result<Self, RoomError> {
    if value > 5 {
      return Err(RoomError::InvalidOperation(
        "Seat index must be between 0 and 5".to_string(),
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

impl fmt::Display for SeatIndex {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<SeatIndex> for usize {
  fn from(value: SeatIndex) -> Self {
    value.value()
  }
}
