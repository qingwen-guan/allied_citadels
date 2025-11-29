use std::fmt;

use crate::domain::valueobjects::{MaxPlayers, SeatIndex};
use crate::errors::RoomError;

/// Seat - value object for seat position in a room
///
/// Internally stores the encoded value using special encoding:
/// - 4 max rooms: 0x40 (64) to 0x43 (67) for seats 0-3
/// - 6 max rooms: 0x60 (96) to 0x65 (101) for seats 0-5
///
/// The encoded value is stored directly, allowing the encoding scheme to be
/// consistent throughout the codebase and database.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Seat(i16);

impl Seat {
  /// Create a Seat from a logical seat number (0-5) with max_players context
  /// The seat number will be encoded internally based on max_players
  pub fn new(seat_index: SeatIndex, max_players: MaxPlayers) -> Result<Self, RoomError> {
    // Validate logical seat number is valid for max_players
    let max_seat = max_players.value() - 1;
    let index_value = seat_index.value();
    if index_value > max_seat {
      return Err(RoomError::InvalidOperation(format!(
        "Seat number {} is invalid for {} max players room (must be 0-{})",
        index_value,
        max_players.value(),
        max_seat
      )));
    }

    // Encode the seat number
    let base = match max_players {
      MaxPlayers::Four => 0x40i16,
      MaxPlayers::Six => 0x60i16,
    };
    let encoded = base + (index_value as i16);

    Ok(Self(encoded))
  }

  /// Create a Seat directly from an encoded value (for database/decoding)
  /// Automatically detects the encoding scheme based on the value range
  pub fn from_encoded(encoded: i16) -> Result<Self, RoomError> {
    // Validate the encoded value is in the expected range
    if !((0x40..=0x43).contains(&encoded) || (0x60..=0x65).contains(&encoded)) {
      return Err(RoomError::InvalidOperation(format!(
        "Invalid encoded seat number: 0x{:02X} (must be 0x40-0x43 or 0x60-0x65)",
        encoded
      )));
    }

    Ok(Self(encoded))
  }

  /// Get the logical seat index (0-5) from the encoded value
  pub fn seat_index(&self) -> SeatIndex {
    let index = if self.0 >= 0x40 && self.0 <= 0x43 {
      // 4 max room encoding
      (self.0 - 0x40) as usize
    } else if self.0 >= 0x60 && self.0 <= 0x65 {
      // 6 max room encoding
      (self.0 - 0x60) as usize
    } else {
      // Should never happen due to validation, but handle gracefully
      0
    };
    SeatIndex::new(index).expect("Seat index should always be valid (0-5)")
  }

  /// Get the max_players value based on the encoded value
  pub fn max_players(&self) -> MaxPlayers {
    if self.0 >= 0x40 && self.0 <= 0x43 {
      MaxPlayers::Four
    } else if self.0 >= 0x60 && self.0 <= 0x65 {
      MaxPlayers::Six
    } else {
      panic!(
        "Invalid encoded seat number: 0x{:02X} (must be 0x40-0x43 or 0x60-0x65)",
        self.0
      );
    }
  }

  /// Get the encoded value (0x40-0x65) stored internally
  pub fn encoded_value(&self) -> i16 {
    self.0
  }

  /// Create seat number 0 for a given max_players
  pub fn zero(max_players: MaxPlayers) -> Self {
    Self::new(SeatIndex::zero(), max_players).expect("Seat 0 should always be valid")
  }
}

impl fmt::Display for Seat {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // Display in format {seat_index}/{max_players}
    write!(f, "{}/{}", self.seat_index().value(), self.max_players().value())
  }
}

impl From<Seat> for usize {
  fn from(value: Seat) -> Self {
    value.seat_index().value()
  }
}

// Implement sqlx traits to allow direct binding
// Seat stores i16 internally, which matches database SMALLINT
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for Seat {
  fn encode_by_ref(
    &self, buf: &mut sqlx::postgres::PgArgumentBuffer,
  ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
    // Seat already stores i16, so we can encode it directly
    <i16 as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&self.0, buf)
  }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for Seat {
  fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
    let num: i16 = <i16 as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
    // Decode using the special encoding scheme
    Seat::from_encoded(num).map_err(|e| e.into())
  }
}

impl sqlx::Type<sqlx::Postgres> for Seat {
  fn type_info() -> sqlx::postgres::PgTypeInfo {
    <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
  }
}
