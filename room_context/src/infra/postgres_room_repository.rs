use sqlx::PgPool;
use user_context::UserId;

use common_context::domain::valueobjects::Pagination;

use crate::domain::repositories::RoomRepository;
use crate::domain::valueobjects::{MaxPlayers, RoomId, RoomName, RoomNumber, SeatNumber};
use crate::domain::{Room, RoomParticipant};
use crate::error::RoomError;

/// PostgreSQL implementation of RoomRepository
pub struct PostgresRoomRepository {
  pool: PgPool,
}

impl PostgresRoomRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait::async_trait]
impl RoomRepository for PostgresRoomRepository {
  async fn find_by_id(&self, id: RoomId) -> Result<Option<Room>, RoomError> {
    let room = sqlx::query_as::<_, Room>(
      "SELECT id, room_number, room_name, creator, max_players, created_at, expires_at FROM room WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(room)
  }

  async fn find_by_name(&self, name: &RoomName) -> Result<Vec<Room>, RoomError> {
    let rooms = sqlx::query_as::<_, Room>(
      "SELECT id, room_number, room_name, creator, max_players, created_at, expires_at FROM room WHERE room_name = $1",
    )
    .bind(name.as_str())
    .fetch_all(&self.pool)
    .await?;

    Ok(rooms)
  }

  async fn find_all(&self, pagination: Pagination) -> Result<Vec<Room>, RoomError> {
    let rooms = sqlx::query_as::<_, Room>(
      "SELECT id, room_number, room_name, creator, max_players, created_at, expires_at FROM room ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(pagination.limit as i64)
    .bind(pagination.offset as i64)
    .fetch_all(&self.pool)
    .await?;

    Ok(rooms)
  }

  async fn find_active(&self, pagination: Pagination) -> Result<Vec<Room>, RoomError> {
    let rooms = sqlx::query_as::<_, Room>(
      "SELECT id, room_number, room_name, creator, max_players, created_at, expires_at FROM room WHERE expires_at > NOW() ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(pagination.limit as i64)
    .bind(pagination.offset as i64)
    .fetch_all(&self.pool)
    .await?;

    Ok(rooms)
  }

  async fn create(&self, creator: UserId, name: &RoomName, max_players: MaxPlayers) -> Result<Room, RoomError> {
    // Get next available room number
    let number = self.get_next_room_number().await?;

    let id = RoomId::from(uuid::Uuid::new_v4());
    let created_at = chrono::Utc::now();
    let expires_at = created_at + chrono::Duration::hours(1);
    let room = Room::new(id, number, name.clone(), creator, max_players, created_at, expires_at);

    sqlx::query(
      "INSERT INTO room (id, room_number, room_name, creator, max_players, created_at, expires_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(room.id())
    .bind(room.number())
    .bind(room.name().as_str())
    .bind(room.creator())
    .bind(room.max_players())
    .bind(room.created_at())
    .bind(room.expires_at())
    .execute(&self.pool)
    .await?;

    Ok(room)
  }

  async fn update_name(&self, id: RoomId, new_name: &RoomName) -> Result<bool, RoomError> {
    let rows_affected = sqlx::query("UPDATE room SET room_name = $1 WHERE id = $2")
      .bind(new_name.as_str())
      .bind(id)
      .execute(&self.pool)
      .await?
      .rows_affected();

    Ok(rows_affected > 0)
  }

  async fn update_max_players(&self, id: RoomId, max_players: MaxPlayers) -> Result<bool, RoomError> {
    // Note: standing up all users and emitting events is handled by room_manager
    let rows_affected = sqlx::query("UPDATE room SET max_players = $1 WHERE id = $2")
      .bind(max_players)
      .bind(id)
      .execute(&self.pool)
      .await?
      .rows_affected();

    Ok(rows_affected > 0)
  }

  async fn delete(&self, id: RoomId) -> Result<bool, RoomError> {
    // Note: emitting delete_room event to all users is handled by room_service
    let rows_affected = sqlx::query("DELETE FROM room WHERE id = $1")
      .bind(id)
      .execute(&self.pool)
      .await?
      .rows_affected();

    Ok(rows_affected > 0)
  }

  async fn get_next_room_number(&self) -> Result<RoomNumber, RoomError> {
    // Get all existing room numbers
    let numbers: Vec<i32> = sqlx::query_scalar::<_, i32>("SELECT room_number FROM room WHERE expires_at > NOW()")
      .fetch_all(&self.pool)
      .await?;

    let existing_numbers: std::collections::HashSet<u32> = numbers.into_iter().map(|n| n as u32).collect();

    if existing_numbers.len() == 1_000_000 {
      return Err(RoomError::InvalidOperation(
        "No more room numbers available".to_string(),
      ));
    }

    // Generate a random room number and find the next available one
    use rand::Rng;
    let mut rng = rand::rng();
    let mut room_number = rng.random_range(0..=999999);

    // Find next available number
    for _ in 0..1_000_000 {
      if !existing_numbers.contains(&room_number) {
        return Ok(RoomNumber::from(room_number));
      }
      room_number = (room_number + 1) % 1_000_000;
    }

    Err(RoomError::InvalidOperation(
      "No more room numbers available".to_string(),
    ))
  }

  async fn add_participant(
    &self, room_id: RoomId, user_id: UserId, seat_number: Option<SeatNumber>, viewing_seat_number: Option<SeatNumber>,
  ) -> Result<RoomParticipant, RoomError> {
    let joined_at = chrono::Utc::now();
    sqlx::query(
      "INSERT INTO room_participant (room_id, user_id, seat_number, viewing_seat_number, joined_at)
VALUES ($1, $2, $3, $4, $5)
ON CONFLICT (room_id, user_id) DO UPDATE
SET seat_number = EXCLUDED.seat_number, viewing_seat_number = EXCLUDED.viewing_seat_number, joined_at = EXCLUDED.joined_at",
    )
    .bind(room_id)
    .bind(user_id)
    .bind(seat_number)
    .bind(viewing_seat_number)
    .bind(joined_at)
    .execute(&self.pool)
    .await?;

    Ok(RoomParticipant::new(
      room_id,
      user_id,
      seat_number,
      viewing_seat_number,
      joined_at,
    ))
  }

  async fn remove_participant(&self, room_id: RoomId, user_id: UserId) -> Result<bool, RoomError> {
    let rows_affected = sqlx::query("DELETE FROM room_participant WHERE room_id = $1 AND user_id = $2")
      .bind(room_id)
      .bind(user_id)
      .execute(&self.pool)
      .await?
      .rows_affected();

    Ok(rows_affected > 0)
  }

  async fn get_participants(&self, room_id: RoomId) -> Result<Vec<RoomParticipant>, RoomError> {
    let participants = sqlx::query_as::<_, RoomParticipant>(
      "SELECT room_id, user_id, seat_number, viewing_seat_number, joined_at
FROM room_participant
WHERE room_id = $1",
    )
    .bind(room_id)
    .fetch_all(&self.pool)
    .await?;

    Ok(participants)
  }

  async fn get_participant(&self, room_id: RoomId, user_id: UserId) -> Result<Option<RoomParticipant>, RoomError> {
    let participant = sqlx::query_as::<_, RoomParticipant>(
      "SELECT room_id, user_id, seat_number, viewing_seat_number, joined_at
FROM room_participant
WHERE room_id = $1 AND user_id = $2",
    )
    .bind(room_id)
    .bind(user_id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(participant)
  }

  async fn get_participant_by_seat(
    &self, room_id: RoomId, seat_number: SeatNumber,
  ) -> Result<Option<RoomParticipant>, RoomError> {
    let participant = sqlx::query_as::<_, RoomParticipant>(
      "SELECT room_id, user_id, seat_number, viewing_seat_number, joined_at
FROM room_participant
WHERE room_id = $1 AND seat_number = $2",
    )
    .bind(room_id)
    .bind(seat_number)
    .fetch_optional(&self.pool)
    .await?;

    Ok(participant)
  }

  async fn update_participant_seat(
    &self, room_id: RoomId, user_id: UserId, new_seat: Option<SeatNumber>,
  ) -> Result<bool, RoomError> {
    let rows_affected = sqlx::query(
      "UPDATE room_participant
SET seat_number = $3, viewing_seat_number = NULL
WHERE room_id = $1 AND user_id = $2",
    )
    .bind(room_id)
    .bind(user_id)
    .bind(new_seat)
    .execute(&self.pool)
    .await?
    .rows_affected();

    Ok(rows_affected > 0)
  }

  async fn update_participant_viewing(
    &self, room_id: RoomId, user_id: UserId, viewing_seat: Option<SeatNumber>,
  ) -> Result<bool, RoomError> {
    // Check if the user is seated, only update viewing_seat_number if the user is NOT seated
    let participant = self.get_participant(room_id, user_id).await?;
    if let Some(p) = participant
      && p.is_sitting()
    {
      return Err(RoomError::InvalidOperation(
        "Cannot update viewing position while seated".to_string(),
      ));
    }

    let rows_affected = sqlx::query(
      "UPDATE room_participant
SET viewing_seat_number = $3
WHERE room_id = $1 AND user_id = $2",
    )
    .bind(room_id)
    .bind(user_id)
    .bind(viewing_seat)
    .execute(&self.pool)
    .await?
    .rows_affected();

    Ok(rows_affected > 0)
  }

  async fn stand_up_participant(&self, room_id: RoomId, user_id: UserId) -> Result<bool, RoomError> {
    let rows_affected = sqlx::query(
      "UPDATE room_participant
SET seat_number = NULL, viewing_seat_number = NULL
WHERE room_id = $1 AND user_id = $2",
    )
    .bind(room_id)
    .bind(user_id)
    .execute(&self.pool)
    .await?
    .rows_affected();

    Ok(rows_affected > 0)
  }

  async fn count_participants(&self, room_id: RoomId) -> Result<usize, RoomError> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM room_participant WHERE room_id = $1")
      .bind(room_id)
      .fetch_one(&self.pool)
      .await?;

    Ok(count as usize)
  }

  async fn count_sitting_participants(&self, room_id: RoomId) -> Result<usize, RoomError> {
    let count: i64 =
      sqlx::query_scalar("SELECT COUNT(*) FROM room_participant WHERE room_id = $1 AND seat_number IS NOT NULL")
        .bind(room_id)
        .fetch_one(&self.pool)
        .await?;

    Ok(count as usize)
  }
}
