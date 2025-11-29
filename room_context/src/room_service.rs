use tracing::{error, info, instrument};
use user_context::{UserId, UserRepository};
use uuid::Uuid;

use common_context::domain::valueobjects::Pagination;

use crate::domain::repositories::RawMessageRepository;
use crate::domain::valueobjects::{MaxPlayers, RoomId, RoomName, SeatNumber};
use crate::domain::{Room, RoomManager, RoomParticipant, RoomRepository};
use crate::error::RoomError;

/// Detailed room information for listing
#[derive(Debug, Clone)]
pub struct RoomDetails {
  pub id: String,
  pub number: u32,
  pub name: String,
  pub creator_id: String,
  pub creator_name: String,
  pub max_players: usize,
  pub seated_players: usize,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub struct RoomService {
  room_manager: RoomManager,
}

impl RoomService {
  pub fn new(
    room_repository: Box<dyn RoomRepository>, user_repository: Box<dyn UserRepository>,
    message_repository: Box<dyn RawMessageRepository>,
  ) -> Self {
    let room_manager = RoomManager::new(room_repository, user_repository, message_repository);
    Self { room_manager }
  }

  /// Generate a random room name (delegates to RoomManager)
  pub fn generate_random_room_name() -> String {
    crate::domain::RoomManager::generate_random_room_name()
  }

  /// Create a new room
  #[instrument(skip(self), fields(name = name_str))]
  pub async fn create_room(&self, name_str: &str, creator: &str, max_players: usize) -> Result<Room, RoomError> {
    // Parse creator string to UserId
    let creator_uuid = creator
      .parse::<Uuid>()
      .map_err(|_| RoomError::InvalidOperation(format!("Invalid creator UUID format: {}", creator)))?;
    let creator_id = UserId::from(creator_uuid);

    // Convert max_players to MaxPlayers value object
    let max_players = MaxPlayers::try_from(max_players).map_err(|_| RoomError::InvalidMaxPlayers)?;

    let name = RoomName::from(name_str);
    let result = self.room_manager.create_room(&name, creator_id, max_players).await;
    match &result {
      Ok(_) => info!("Successfully created room: {}", name_str),
      Err(e) => error!("Failed to create room {}: {:?}", name_str, e),
    }
    result
  }

  /// Get room by ID
  #[instrument(skip(self), fields(id = id_str))]
  pub async fn get_room_by_id(&self, id_str: &str) -> Result<Option<Room>, RoomError> {
    // Parse string to UUID
    let uuid = id_str
      .parse::<Uuid>()
      .map_err(|_| RoomError::InvalidOperation(format!("Invalid room UUID format: {}", id_str)))?;
    let room_id = RoomId::from(uuid);

    let result = self.room_manager.get_room_by_id(room_id).await;
    if let Err(e) = &result {
      error!("Error getting room by ID {}: {:?}", id_str, e);
    }
    result
  }

  /// Get room by name
  #[instrument(skip(self), fields(name = name_str))]
  pub async fn get_room_by_name(&self, name_str: &str) -> Result<Option<Room>, RoomError> {
    let name = RoomName::from(name_str);
    let result = self.room_manager.get_room_by_name(&name).await;
    if let Err(e) = &result {
      error!("Error getting room by name {}: {:?}", name_str, e);
    }
    result
  }

  /// List all rooms with optional pagination
  #[instrument(skip(self))]
  pub async fn list_rooms(&self, offset: Option<usize>, limit: Option<usize>) -> Result<Vec<Room>, RoomError> {
    let pagination = Pagination::from_options(limit, offset);
    let result = self.room_manager.list_rooms(pagination).await;
    if let Err(ref e) = result {
      error!("Error listing rooms: {:?}", e);
    }
    result
  }

  /// List all active (non-expired) rooms with optional pagination
  #[instrument(skip(self))]
  pub async fn list_active_rooms(&self, offset: Option<usize>, limit: Option<usize>) -> Result<Vec<Room>, RoomError> {
    let pagination = Pagination::from_options(limit, offset);
    let result = self.room_manager.list_active_rooms(pagination).await;
    if let Err(ref e) = result {
      error!("Error listing active rooms: {:?}", e);
    }
    result
  }

  /// List all active (non-expired) rooms with detailed information
  #[instrument(skip(self))]
  pub async fn list_active_rooms_detailed(
    &self, offset: Option<usize>, limit: Option<usize>,
  ) -> Result<Vec<RoomDetails>, RoomError> {
    let pagination = Pagination::from_options(limit, offset);
    let result = self.room_manager.list_active_rooms_info(pagination).await;
    if let Err(ref e) = result {
      error!("Error listing active rooms with detailed information: {:?}", e);
    }
    Ok(
      result?
        .into_iter()
        .map(|room_info| RoomDetails {
          id: room_info.room.id().to_string(),
          number: room_info.room.number().value(),
          name: room_info.room.name().as_str().to_string(),
          creator_id: room_info.room.creator().to_string(),
          creator_name: room_info.creator_name,
          max_players: room_info.room.max_players().value(),
          seated_players: room_info.seated_players,
          created_at: room_info.room.created_at(),
          expires_at: room_info.room.expires_at(),
        })
        .collect(),
    )
  }

  /// Update room name
  #[instrument(skip(self), fields(id = %id, new_name = new_name_str))]
  pub async fn update_room_name(&self, id: uuid::Uuid, new_name_str: &str) -> Result<(), RoomError> {
    let room_id = RoomId::from(id);
    let new_name = RoomName::from(new_name_str);
    let result = self.room_manager.update_room_name(room_id, &new_name).await;
    match &result {
      Ok(_) => info!("Successfully updated room name for ID: {}", id),
      Err(e) => error!("Failed to update room name for ID {}: {:?}", id, e),
    }
    result
  }

  /// Update room max players
  #[instrument(skip(self), fields(id = %id, max_players = max_players_value))]
  pub async fn update_room_max_players(&self, id: RoomId, max_players_value: usize) -> Result<(), RoomError> {
    let max_players = MaxPlayers::new(max_players_value)?;
    let result = self.room_manager.update_room_max_players(id, max_players).await;
    match &result {
      Ok(_) => {
        info!("Successfully updated max players for ID: {}", id);
      },
      Err(e) => error!("Failed to update max players for ID {}: {:?}", id, e),
    }
    result.map(|_| ())
  }

  /// Delete room by ID
  #[instrument(skip(self), fields(id = %id))]
  pub async fn delete_room(&self, id: RoomId) -> Result<(), RoomError> {
    let result = self.room_manager.delete_room(id).await;
    match &result {
      Ok(_) => {
        info!("Successfully deleted room with ID: {}", id);
      },
      Err(e) => error!("Failed to delete room with ID {}: {:?}", id, e),
    }
    result
  }

  /// Enter a room (always enters standing by, use change_seat to take a seat)
  #[instrument(skip(self), fields(room_id = %room_id, user_id = %user_id))]
  pub async fn enter_room(&self, room_id: RoomId, user_id: UserId) -> Result<(), RoomError> {
    let result = self.room_manager.enter_room_standing_by(user_id, room_id).await;
    match &result {
      Ok(crate::domain::EnterRoomOutcome::Success) => {
        info!("User {} entered room {} and is standing by", user_id, room_id);
      },
      Ok(crate::domain::EnterRoomOutcome::AlreadyInRoom) => {
        info!("User {} is already in room {}", user_id, room_id);
      },
      Ok(crate::domain::EnterRoomOutcome::RoomExpired) => {
        return Err(RoomError::InvalidOperation("Room has expired".to_string()));
      },
      Err(e) => error!("Failed to enter room {} for user {}: {:?}", room_id, user_id, e),
    }
    result.map(|_| ())
  }

  /// Enter a room and take a random available seat
  #[instrument(skip(self), fields(user_id = %user_id, room_id = %room_id))]
  pub async fn enter_room_and_take_random_seat(
    &self, user_id: UserId, room_id: RoomId,
  ) -> Result<Option<SeatNumber>, RoomError> {
    let seat = self
      .room_manager
      .enter_room_and_take_random_seat(user_id, room_id)
      .await?;
    match seat {
      Some(seat_num) => {
        info!(
          "User {} entered room {} and took random seat {}",
          user_id,
          room_id,
          seat_num.value()
        );
      },
      None => {
        info!("User {} entered room {} but no seats available", user_id, room_id);
      },
    }
    Ok(seat)
  }

  /// Take a random available seat in a room
  #[instrument(skip(self), fields(room_id = %room_id, user_id = %user_id))]
  pub async fn take_random_seat(&self, room_id: RoomId, user_id: UserId) -> Result<Option<SeatNumber>, RoomError> {
    self.room_manager.take_random_seat(user_id, room_id).await
  }

  /// Leave a room
  #[instrument(skip(self), fields(room_id = %room_id, user_id = %user_id))]
  pub async fn leave_room(&self, room_id: RoomId, user_id: UserId) -> Result<(), RoomError> {
    let result = self.room_manager.leave_room(room_id, user_id).await;
    match &result {
      Ok(_) => info!("User {} left room {}", user_id, room_id),
      Err(e) => error!("Failed to leave room {} for user {}: {:?}", room_id, user_id, e),
    }
    result
  }

  /// Change seat in a room
  /// Returns true if seat was successfully changed, false otherwise
  #[instrument(skip(self), fields(room_id = %room_id, user_id = %user_id, seat = new_seat.value()))]
  pub async fn change_seat(&self, room_id: RoomId, user_id: UserId, new_seat: SeatNumber) -> Result<bool, RoomError> {
    match self.room_manager.change_seat(room_id, user_id, new_seat).await {
      Ok(crate::domain::ChangeSeatOutcome::Success) => {
        info!(
          "User {} changed to seat {} in room {}",
          user_id,
          new_seat.value(),
          room_id
        );
        Ok(true)
      },
      Ok(crate::domain::ChangeSeatOutcome::AlreadyInSeat) => {
        info!(
          "User {} is already in seat {} in room {}",
          user_id,
          new_seat.value(),
          room_id
        );
        Ok(true)
      },
      Ok(crate::domain::ChangeSeatOutcome::SeatOccupied) => {
        info!(
          "User {} tried to change to seat {} in room {} but seat is occupied",
          user_id,
          new_seat.value(),
          room_id
        );
        Ok(false)
      },
      Ok(crate::domain::ChangeSeatOutcome::SeatOutOfRange) => Err(RoomError::InvalidOperation(format!(
        "Seat number {} is out of range",
        new_seat.value()
      ))),
      Err(e) => {
        error!(
          "Failed to change seat for user {} in room {}: {:?}",
          user_id, room_id, e
        );
        Err(e)
      },
    }
  }

  /// Stand up from seat (become standing by)
  #[instrument(skip(self), fields(room_id = %room_id, user_id = %user_id))]
  pub async fn stand_up(&self, room_id: RoomId, user_id: UserId) -> Result<(), RoomError> {
    match self.room_manager.stand_up(room_id, user_id).await {
      Ok(crate::domain::StandUpOutcome::Success) => {
        info!("User {} stood up in room {}", user_id, room_id);
        Ok(())
      },
      Ok(crate::domain::StandUpOutcome::AlreadyStanding) => {
        info!("User {} is already standing in room {}", user_id, room_id);
        Ok(())
      },
      Ok(crate::domain::StandUpOutcome::NotInRoom) => {
        Err(RoomError::InvalidOperation("User is not in this room".to_string()))
      },
      Err(e) => {
        error!("Failed to stand up for user {} in room {}: {:?}", user_id, room_id, e);
        Err(e)
      },
    }
  }

  /// View behind a seat (must be standing by)
  #[instrument(skip(self), fields(room_id = %room_id, user_id = %user_id, viewing_seat = viewing_seat.value()))]
  pub async fn view_behind_seat(
    &self, room_id: RoomId, user_id: UserId, viewing_seat: SeatNumber,
  ) -> Result<(), RoomError> {
    let result = self.room_manager.view_behind_seat(room_id, user_id, viewing_seat).await;
    match &result {
      Ok(_) => info!(
        "User {} is viewing behind seat {} in room {}",
        user_id,
        viewing_seat.value(),
        room_id
      ),
      Err(e) => error!(
        "Failed to view behind seat for user {} in room {}: {:?}",
        user_id, room_id, e
      ),
    }
    result
  }

  /// Stop viewing (but remain in room)
  #[instrument(skip(self), fields(room_id = %room_id, user_id = %user_id))]
  pub async fn stop_viewing(&self, room_id: RoomId, user_id: UserId) -> Result<(), RoomError> {
    let result = self.room_manager.stop_viewing(room_id, user_id).await;
    match &result {
      Ok(_) => info!("User {} stopped viewing in room {}", user_id, room_id),
      Err(e) => error!(
        "Failed to stop viewing for user {} in room {}: {:?}",
        user_id, room_id, e
      ),
    }
    result
  }

  /// Get all participants in a room
  #[instrument(skip(self), fields(room_id = %room_id))]
  pub async fn get_room_participants(&self, room_id: RoomId) -> Result<Vec<RoomParticipant>, RoomError> {
    let result = self.room_manager.get_room_participants(room_id).await;
    if let Err(e) = &result {
      error!("Error getting participants for room {}: {:?}", room_id, e);
    }
    result
  }

  /// Get participant info for a user in a room
  #[instrument(skip(self), fields(room_id = %room_id, user_id = %user_id))]
  pub async fn get_participant(
    &self, room_id: RoomId, user_id: UserId,
  ) -> Result<Option<RoomParticipant>, RoomError> {
    let result = self.room_manager.get_participant(room_id, user_id).await;
    if let Err(e) = &result {
      error!("Error getting participant {} in room {}: {:?}", user_id, room_id, e);
    }
    result
  }
}
