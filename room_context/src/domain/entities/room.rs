use sqlx::Row;
use user_context::domain::valueobjects::UserId;

use crate::domain::valueobjects::{MaxPlayers, RoomId, RoomName, RoomNumber, SeatNumber};

/// Parameters for creating a Room with stand_by_limit
#[derive(Debug)]
pub struct RoomWithStandByLimitParams {
  pub id: RoomId,
  pub number: RoomNumber,
  pub name: RoomName,
  pub creator: UserId,
  pub max_players: MaxPlayers,
  pub stand_by_limit: Option<usize>,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct Room {
  id: RoomId,
  number: RoomNumber,
  name: RoomName,
  creator: UserId,
  max_players: MaxPlayers,
  stand_by_limit: Option<usize>,
  created_at: chrono::DateTime<chrono::Utc>,
  expires_at: chrono::DateTime<chrono::Utc>,
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for Room {
  fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
    Ok(Room {
      id: row.try_get("id")?,
      number: row.try_get("room_number")?,
      name: RoomName::from(row.try_get::<String, _>("room_name")?),
      creator: row.try_get("creator")?,
      max_players: row.try_get("max_players")?,
      stand_by_limit: row
        .try_get::<Option<i32>, _>("stand_by_limit")
        .ok()
        .flatten()
        .map(|v| v as usize),
      created_at: row.try_get("created_at")?,
      expires_at: row.try_get("expires_at")?,
    })
  }
}

impl Room {
  pub fn new(
    id: RoomId, number: RoomNumber, name: impl Into<RoomName>, creator: UserId, max_players: MaxPlayers,
    created_at: chrono::DateTime<chrono::Utc>, expires_at: chrono::DateTime<chrono::Utc>,
  ) -> Self {
    Self {
      id,
      number,
      name: name.into(),
      creator,
      max_players,
      stand_by_limit: None,
      created_at,
      expires_at,
    }
  }

  pub fn with_stand_by_limit(params: RoomWithStandByLimitParams) -> Self {
    Self {
      id: params.id,
      number: params.number,
      name: params.name,
      creator: params.creator,
      max_players: params.max_players,
      stand_by_limit: params.stand_by_limit,
      created_at: params.created_at,
      expires_at: params.expires_at,
    }
  }

  pub fn id(&self) -> RoomId {
    self.id
  }

  pub fn number(&self) -> RoomNumber {
    self.number
  }

  pub fn name(&self) -> &RoomName {
    &self.name
  }

  pub fn creator(&self) -> UserId {
    self.creator
  }

  pub fn max_players(&self) -> MaxPlayers {
    self.max_players
  }

  pub fn stand_by_limit(&self) -> Option<usize> {
    self.stand_by_limit
  }

  pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
    self.created_at
  }

  pub fn expires_at(&self) -> chrono::DateTime<chrono::Utc> {
    self.expires_at
  }

  /// Validate if a seat number is valid for this room's max_players
  pub fn is_valid_seat_number(&self, seat_number: SeatNumber) -> bool {
    let max_seat = (self.max_players.value() - 1) as u8;
    seat_number.value() <= max_seat
  }
}
