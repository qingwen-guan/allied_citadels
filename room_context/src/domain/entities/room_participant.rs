use account_context::AccountId;
use sqlx::Row;

use crate::domain::valueobjects::{RoomId, SeatNumber};

#[derive(Debug, Clone)]
pub struct RoomParticipant {
  room_id: RoomId,
  account_id: AccountId,
  seat_number: Option<SeatNumber>, // None = standing by, Some(seat) = sitting in that seat
  viewing_seat_number: Option<SeatNumber>, // None = not viewing, Some(seat) = viewing behind that seat
  joined_at: chrono::DateTime<chrono::Utc>,
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for RoomParticipant {
  fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
    Ok(RoomParticipant {
      room_id: row.try_get("room_id")?,
      account_id: row.try_get("account_id")?,
      seat_number: row.try_get("seat_number")?,
      viewing_seat_number: row.try_get("viewing_seat_number")?,
      joined_at: row.try_get("joined_at")?,
    })
  }
}

impl RoomParticipant {
  pub fn new(
    room_id: RoomId, account_id: AccountId, seat_number: Option<SeatNumber>, viewing_seat_number: Option<SeatNumber>,
    joined_at: chrono::DateTime<chrono::Utc>,
  ) -> Self {
    Self {
      room_id,
      account_id,
      seat_number,
      viewing_seat_number,
      joined_at,
    }
  }

  pub fn room_id(&self) -> RoomId {
    self.room_id
  }

  pub fn account_id(&self) -> AccountId {
    self.account_id
  }

  /// Get the seat number if the participant is sitting, None if standing by
  pub fn seat_number(&self) -> Option<SeatNumber> {
    self.seat_number
  }

  /// Get the viewing seat number if the participant is viewing, None if not viewing
  pub fn viewing_seat_number(&self) -> Option<SeatNumber> {
    self.viewing_seat_number
  }

  /// Check if the participant is sitting (has a seat)
  pub fn is_sitting(&self) -> bool {
    self.seat_number.is_some()
  }

  /// Check if the participant is standing by (no seat)
  pub fn is_standing_by(&self) -> bool {
    self.seat_number.is_none()
  }

  /// Check if the participant is viewing behind a seat
  pub fn is_viewing(&self) -> bool {
    self.viewing_seat_number.is_some()
  }

  pub fn joined_at(&self) -> chrono::DateTime<chrono::Utc> {
    self.joined_at
  }
}
