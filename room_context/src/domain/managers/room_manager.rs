use user_context::domain::UserRepository;
use user_context::domain::valueobjects::UserId;

use crate::domain::entities::{Room, RoomParticipant, RoomToUserMessage, RoomToUserMessageDetails};
use crate::domain::managers::MessageManager;
use common_context::domain::valueobjects::Pagination;

use crate::domain::repositories::{RawMessageRepository, RoomRepository};
use crate::domain::valueobjects::{MaxPlayers, RoomId, RoomName, Seat, SeatIndex};
use crate::errors::RoomError;

/// Outcome of update_room_max_players operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateMaxPlayersOutcome {
  Changed,
  Unchanged,
}

/// Outcome of enter_room_standing_by operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnterRoomOutcome {
  AlreadyInRoom,
  RoomExpired,
  Success,
}

/// Outcome of change_seat operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeSeatOutcome {
  AlreadyInSeat,
  SeatOutOfRange,
  SeatOccupied,
  Success,
}

/// Outcome of stand_up operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StandUpOutcome {
  AlreadyStanding,
  NotInRoom,
  Success,
}

/// Outcome of take_seat operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TakeSeatOutcome {
  RoomExpired,
  SeatOutOfRange,
  SeatOccupied,
  NotStandingBy,
  Success,
}

/// Room listing information with creator name and seated players count
#[derive(Debug, Clone)]
pub struct RoomListInfo {
  pub room: Room,
  pub creator_name: String,
  pub seated_players: usize,
}

pub struct RoomManager {
  room_repository: Box<dyn RoomRepository>,
  user_repository: Box<dyn UserRepository>,
  message_manager: MessageManager,
}

impl RoomManager {
  pub fn new(
    room_repository: Box<dyn RoomRepository>, user_repository: Box<dyn UserRepository>,
    raw_message_repository: Box<dyn RawMessageRepository>,
  ) -> Self {
    Self {
      room_repository,
      user_repository,
      message_manager: MessageManager::new(raw_message_repository),
    }
  }

  /// Helper method to send messages to multiple participants in a room
  /// Attempts batch insert first, falls back to individual sends on failure
  async fn send_messages_to_participants(
    &self, room_id: RoomId, participants: &[RoomParticipant], details: RoomToUserMessageDetails,
  ) {
    // Create messages for each participant
    let messages: Vec<_> = participants
      .iter()
      .map(|p| RoomToUserMessage::new(room_id, p.user_id(), details.clone()))
      .collect();

    if let Err(e) = self.message_manager.batch_insert_room_to_user_message(messages).await {
      tracing::error!("Failed to batch send messages in room {}: {:?}", room_id, e);
    }
  }

  /// Generate a random room name with Chinese characters
  pub fn generate_random_room_name() -> String {
    use rand::Rng;
    let mut rng = rand::rng();

    // Chinese adjectives and nouns for room names
    let chinese_adjectives = ["勇敢", "智慧", "强大", "高贵", "聪明", "大胆", "明亮", "敏捷"];
    let chinese_nouns = ["雄鹰", "雄狮", "野狼", "神龙", "凤凰", "猛虎", "巨熊", "猎鹰"];

    let adj = chinese_adjectives[rng.random_range(0..chinese_adjectives.len())];
    let noun = chinese_nouns[rng.random_range(0..chinese_nouns.len())];
    format!("{}{}", adj, noun)
  }

  /// Create a new room
  pub async fn create_room(
    &self, name: &RoomName, creator: UserId, max_players: MaxPlayers,
  ) -> Result<Room, RoomError> {
    // Validate that the creator exists
    let user = self
      .user_repository
      .find_by_id(creator)
      .await
      .map_err(|e| RoomError::InvalidOperation(format!("Failed to validate creator: {}", e)))?;

    if user.is_none() {
      return Err(RoomError::InvalidOperation(format!(
        "Creator {} does not exist",
        creator
      )));
    }

    // Create the room (repository handles room number generation)
    self.room_repository.create(creator, name, max_players).await
  }

  /// Get room by ID
  pub async fn get_room_by_id(&self, id: RoomId) -> Result<Option<Room>, RoomError> {
    self.room_repository.find_by_id(id).await
  }

  /// Get room by name (returns first match if multiple exist)
  pub async fn get_room_by_name(&self, name: &RoomName) -> Result<Option<Room>, RoomError> {
    let rooms = self.room_repository.find_by_name(name).await?;
    Ok(rooms.into_iter().next())
  }

  /// List all rooms with pagination
  pub async fn list_rooms(&self, pagination: Pagination) -> Result<Vec<Room>, RoomError> {
    self.room_repository.find_all(pagination).await
  }

  /// List all active (non-expired) rooms with pagination
  pub async fn list_active_rooms(&self, pagination: Pagination) -> Result<Vec<Room>, RoomError> {
    self.room_repository.find_active(pagination).await
  }

  /// List all active (non-expired) rooms with detailed information
  pub async fn list_active_rooms_info(&self, pagination: Pagination) -> Result<Vec<RoomListInfo>, RoomError> {
    let rooms = self.room_repository.find_active(pagination).await?;

    // TODO: batch get creator names and seated players count
    let mut rooms_info = Vec::new();
    for room in rooms {
      let creator_id = room.creator();
      let creator_name = match self.user_repository.find_by_id(creator_id).await {
        Ok(Some(user)) => user.nickname().as_str().to_string(),
        Ok(None) => format!("Unknown ({})", creator_id),
        Err(_) => format!("Unknown ({})", creator_id),
      };
      let seated_count = self.room_repository.count_sitting_participants(room.id()).await?;
      rooms_info.push(RoomListInfo {
        room,
        creator_name,
        seated_players: seated_count,
      });
    }

    Ok(rooms_info)
  }

  /// Update room name
  pub async fn update_room_name(&self, id: RoomId, new_name: &RoomName) -> Result<(), RoomError> {
    let updated = self.room_repository.update_name(id, new_name).await?;

    if !updated {
      return Err(RoomError::NotFound);
    }

    Ok(())
  }

  /// Update room max players
  pub async fn update_room_max_players(
    &self, id: RoomId, max_players: MaxPlayers,
  ) -> Result<UpdateMaxPlayersOutcome, RoomError> {
    // Check if room exists first
    let room = self.room_repository.find_by_id(id).await?;
    let room = room.ok_or(RoomError::NotFound)?;

    // Check if the value is unchanged
    if room.max_players() == max_players {
      return Ok(UpdateMaxPlayersOutcome::Unchanged);
    }

    let updated = self.room_repository.update_max_players(id, max_players).await?;
    if !updated {
      // This shouldn't happen if room exists, but handle it anyway
      return Err(RoomError::NotFound);
    }

    // Get participants once for both messaging and standing up
    let participants = self.get_room_participants(id).await?;

    // Send message to acknowledge all users in the room about the max_players update
    let old_max_players = room.max_players();
    let details = RoomToUserMessageDetails::MaxPlayersUpdated {
      room_id: id,
      from: old_max_players,
      to: max_players,
    };
    self.send_messages_to_participants(id, &participants, details).await;

    // Stand up all users in the room when max_players changes
    // (stand_up already clears both seat_number and viewing_seat_number)
    let sitting_participants: Vec<_> = participants.iter().filter(|p| p.is_sitting()).cloned().collect();
    let mut force_stand_up_messages = Vec::new();

    for participant in &sitting_participants {
      if let Err(e) = self.stand_up(id, participant.user_id()).await {
        // Log error but continue with other participants
        tracing::error!(
          "Failed to stand up user {} in room {}: {:?}",
          participant.user_id(),
          id,
          e
        );
      } else {
        // Collect messages for batch insert
        let details = RoomToUserMessageDetails::ForceStandUp {
          room_id: id,
          user_id: participant.user_id(),
          reason: Some("Room max players changed".to_string()),
        };
        let message = RoomToUserMessage::new(id, participant.user_id(), details);
        force_stand_up_messages.push(message);
      }
    }

    // Send force_stand_up messages using batch insert
    if !force_stand_up_messages.is_empty()
      && let Err(e) = self
        .message_manager
        .batch_insert_room_to_user_message(force_stand_up_messages)
        .await
    {
      tracing::error!("Failed to batch send force_stand_up messages in room {}: {:?}", id, e);
    }

    Ok(UpdateMaxPlayersOutcome::Changed)
  }

  /// Delete room by ID
  pub async fn delete_room(&self, id: RoomId) -> Result<(), RoomError> {
    // Get participants before deletion to send messages
    let participants = self.get_room_participants(id).await?;

    // Send delete_room message to all users in the room before deletion
    let details = RoomToUserMessageDetails::RoomDeleted { room_id: id };
    self.send_messages_to_participants(id, &participants, details).await;

    // Delete all participants explicitly (don't rely on CASCADE)
    for participant in participants {
      if let Err(e) = self.room_repository.remove_participant(id, participant.user_id()).await {
        tracing::error!(
          "Failed to remove participant {} from room {}: {:?}",
          participant.user_id(),
          id,
          e
        );
        // Continue deleting other participants even if one fails
      }
    }

    let is_deleted = self.room_repository.delete(id).await?;
    if !is_deleted {
      return Err(RoomError::NotFound);
    }

    Ok(())
  }

  /// Enter a room (always enters standing by, use change_seat to take a seat)
  pub async fn enter_room_standing_by(
    &self, user_id: UserId, room_id: RoomId,
  ) -> Result<EnterRoomOutcome, RoomError> {
    // Check if room exists and is not expired
    let room = self
      .room_repository
      .find_by_id(room_id)
      .await?
      .ok_or(RoomError::NotFound)?;

    if chrono::Utc::now() > room.expires_at() {
      return Ok(EnterRoomOutcome::RoomExpired);
    }

    // Check if user is already in the room
    if self.room_repository.get_participant(room_id, user_id).await?.is_some() {
      // If already in room, return AlreadyInRoom
      return Ok(EnterRoomOutcome::AlreadyInRoom);
    }

    // Add participant (always standing by)
    self
      .room_repository
      .add_participant(room_id, user_id, None, None)
      .await?;

    Ok(EnterRoomOutcome::Success)
  }

  /// Leave a room
  pub async fn leave_room(&self, user_id: UserId, room_id: RoomId) -> Result<(), RoomError> {
    // Check if user is in the room
    let _participant = self
      .room_repository
      .get_participant(room_id, user_id)
      .await?
      .ok_or_else(|| RoomError::InvalidOperation("User is not in this room".to_string()))?;

    // Remove participant
    let removed = self.room_repository.remove_participant(room_id, user_id).await?;
    if !removed {
      return Err(RoomError::InvalidOperation("Failed to remove participant".to_string()));
    }

    Ok(())
  }

  /// Change seat in a room
  pub async fn change_seat(
    &self, room_id: RoomId, user_id: UserId, new_seat: Seat,
  ) -> Result<ChangeSeatOutcome, RoomError> {
    // Check if room exists and is not expired
    let room = self
      .room_repository
      .find_by_id(room_id)
      .await?
      .ok_or(RoomError::NotFound)?;

    if chrono::Utc::now() > room.expires_at() {
      return Err(RoomError::InvalidOperation("Room has expired".to_string()));
    }

    // Check if user is in the room
    let participant = self
      .room_repository
      .get_participant(room_id, user_id)
      .await?
      .ok_or_else(|| RoomError::InvalidOperation("User is not in this room".to_string()))?;

    // If already in the same seat, return AlreadyInSeat
    if participant.seat_number() == Some(new_seat) {
      return Ok(ChangeSeatOutcome::AlreadyInSeat);
    }

    // Check if new seat is already taken
    if let Some(existing) = self.room_repository.get_participant_by_seat(room_id, new_seat).await?
      && existing.user_id() != user_id
    {
      return Ok(ChangeSeatOutcome::SeatOccupied);
    }

    // Check if room is full (only count sitting participants)
    let sitting_count = self.room_repository.count_sitting_participants(room_id).await?;
    if sitting_count >= room.max_players().value() {
      return Err(RoomError::InvalidOperation("Room is full".to_string()));
    }

    // Update seat (clears viewing when sitting)
    self
      .room_repository
      .update_participant_seat(room_id, user_id, Some(new_seat))
      .await?;

    Ok(ChangeSeatOutcome::Success)
  }

  /// Take a seat (for users standing by)
  pub async fn take_seat(&self, room_id: RoomId, user_id: UserId, seat: Seat) -> Result<TakeSeatOutcome, RoomError> {
    // Check if room exists and is not expired
    let room = self
      .room_repository
      .find_by_id(room_id)
      .await?
      .ok_or(RoomError::NotFound)?;

    if chrono::Utc::now() > room.expires_at() {
      return Ok(TakeSeatOutcome::RoomExpired);
    }

    // Check if user is in the room
    let participant = self
      .room_repository
      .get_participant(room_id, user_id)
      .await?
      .ok_or_else(|| RoomError::InvalidOperation("User is not in this room".to_string()))?;

    // Must be standing by to take a seat
    if participant.is_sitting() {
      return Ok(TakeSeatOutcome::NotStandingBy);
    }

    // Check if seat is already taken
    if let Some(existing) = self.room_repository.get_participant_by_seat(room_id, seat).await?
      && existing.user_id() != user_id
    {
      return Ok(TakeSeatOutcome::SeatOccupied);
    }

    // Check if room is full (only count sitting participants)
    let sitting_count = self.room_repository.count_sitting_participants(room_id).await?;
    if sitting_count >= room.max_players().value() {
      return Err(RoomError::InvalidOperation("Room is full".to_string()));
    }

    // Update seat (clears viewing when sitting)
    self
      .room_repository
      .update_participant_seat(room_id, user_id, Some(seat))
      .await?;

    Ok(TakeSeatOutcome::Success)
  }

  /// Stand up from seat (become standing by)
  pub async fn stand_up(&self, room_id: RoomId, user_id: UserId) -> Result<StandUpOutcome, RoomError> {
    // Check if room exists and is not expired
    let _room = self
      .room_repository
      .find_by_id(room_id)
      .await?
      .ok_or(RoomError::NotFound)?;

    // Check if user is in the room
    let participant = match self.room_repository.get_participant(room_id, user_id).await? {
      Some(p) => p,
      None => return Ok(StandUpOutcome::NotInRoom),
    };

    // If already standing, return AlreadyStanding
    if participant.is_standing_by() {
      return Ok(StandUpOutcome::AlreadyStanding);
    }

    // Stand up (set seat to None, viewing to None)
    self.room_repository.stand_up_participant(room_id, user_id).await?;

    Ok(StandUpOutcome::Success)
  }

  /// View behind a seat (must be standing by)
  pub async fn view_behind_seat(
    &self, room_id: RoomId, user_id: UserId, viewing_seat: Seat,
  ) -> Result<(), RoomError> {
    // Check if room exists and is not expired
    let room = self
      .room_repository
      .find_by_id(room_id)
      .await?
      .ok_or(RoomError::NotFound)?;

    if chrono::Utc::now() > room.expires_at() {
      return Err(RoomError::InvalidOperation("Room has expired".to_string()));
    }

    // Check if user is in the room
    let participant = self
      .room_repository
      .get_participant(room_id, user_id)
      .await?
      .ok_or_else(|| RoomError::InvalidOperation("User is not in this room".to_string()))?;

    // Must be standing by to view
    if participant.is_sitting() {
      return Err(RoomError::InvalidOperation(
        "Cannot view while sitting. Please stand up first.".to_string(),
      ));
    }

    // Update viewing position
    self
      .room_repository
      .update_participant_viewing(room_id, user_id, Some(viewing_seat))
      .await?;

    Ok(())
  }

  /// Stop viewing (but remain in room)
  // Note: viewing_seat_number is None means viewing globally, rather than not viewing
  pub async fn stop_viewing(&self, user_id: UserId, room_id: RoomId) -> Result<(), RoomError> {
    // Check if user is in the room
    let _participant = self
      .room_repository
      .get_participant(room_id, user_id)
      .await?
      .ok_or_else(|| RoomError::InvalidOperation("User is not in this room".to_string()))?;

    // Update viewing to None
    self
      .room_repository
      .update_participant_viewing(room_id, user_id, None)
      .await?;

    Ok(())
  }

  /// Get all participants in a room
  pub async fn get_room_participants(&self, room_id: RoomId) -> Result<Vec<RoomParticipant>, RoomError> {
    // Check if room exists
    let _room = self
      .room_repository
      .find_by_id(room_id)
      .await?
      .ok_or(RoomError::NotFound)?;

    self.room_repository.get_participants(room_id).await
  }

  /// Get participant info for a user in a room
  pub async fn get_participant(
    &self, room_id: RoomId, user_id: UserId,
  ) -> Result<Option<RoomParticipant>, RoomError> {
    self.room_repository.get_participant(room_id, user_id).await
  }

  /// Take a random available seat in a room
  pub async fn take_random_seat(&self, user_id: UserId, room_id: RoomId) -> Result<Option<Seat>, RoomError> {
    let room = self.get_room_by_id(room_id).await?.ok_or(RoomError::NotFound)?;

    // Get occupied seats from participants (using encoded values)
    let participants = self.room_repository.get_participants(room_id).await?;
    let occupied_seats: std::collections::HashSet<i16> = participants
      .into_iter()
      .filter_map(|p| p.seat_number().map(|s| s.encoded_value()))
      .collect();

    // Generate all possible seats using the encoding scheme
    let max_seat = room.max_players().value() - 1;
    let all_seats: Vec<Seat> = (0..=max_seat)
      .map(|idx| {
        let seat_index = SeatIndex::new(idx)?;
        Seat::new(seat_index, room.max_players())
      })
      .collect::<Result<Vec<_>, _>>()?;

    // Find available seats (not occupied)
    let available_seats: Vec<Seat> = all_seats
      .into_iter()
      .filter(|seat| !occupied_seats.contains(&seat.encoded_value()))
      .collect();

    if available_seats.is_empty() {
      return Ok(None);
    }

    // Pick a random available seat
    use rand::Rng;
    let mut rng = rand::rng();
    let random_index = rng.random_range(0..available_seats.len());
    let random_seat = available_seats[random_index];

    // Try to take the seat
    match self.take_seat(room_id, user_id, random_seat).await? {
      TakeSeatOutcome::Success => Ok(Some(random_seat)),
      _ => Ok(None), // Seat was taken between checking and attempting (race condition)
    }
  }

  /// Enter a room and take a random available seat
  pub async fn enter_room_and_take_random_seat(
    &self, user_id: UserId, room_id: RoomId,
  ) -> Result<Option<Seat>, RoomError> {
    // Check if room exists and is not expired
    let room = self
      .room_repository
      .find_by_id(room_id)
      .await?
      .ok_or(RoomError::NotFound)?;

    if chrono::Utc::now() > room.expires_at() {
      return Err(RoomError::InvalidOperation("Room has expired".to_string()));
    }

    // Get occupied seats from participants before adding participant (using encoded values)
    let participants = self.room_repository.get_participants(room_id).await?;
    let occupied_seats: std::collections::HashSet<i16> = participants
      .into_iter()
      .filter_map(|p| p.seat_number().map(|s| s.encoded_value()))
      .collect();

    // Generate all possible seats using the encoding scheme
    let max_seat = room.max_players().value() - 1;
    let all_seats: Vec<Seat> = (0..=max_seat)
      .map(|idx| {
        let seat_index = SeatIndex::new(idx)?;
        Seat::new(seat_index, room.max_players())
      })
      .collect::<Result<Vec<_>, _>>()?;

    // Find available seats (not occupied)
    let available_seats: Vec<Seat> = all_seats
      .iter()
      .filter(|seat| !occupied_seats.contains(&seat.encoded_value()))
      .cloned()
      .collect();

    // Check if user is already in the room
    let participant = self.room_repository.get_participant(room_id, user_id).await?;
    let random_seat = if participant.is_none() {
      if available_seats.is_empty() {
        return Ok(None);
      }

      // Pick a random available seat
      use rand::Rng;
      let mut rng = rand::rng();
      let random_index = rng.random_range(0..available_seats.len());
      let random_seat = available_seats[random_index];

      // Add participant directly with the seat_number to avoid multiple db queries
      self
        .room_repository
        .add_participant(room_id, user_id, Some(random_seat), None)
        .await?;

      Some(random_seat)
    } else {
      // User already in room, find available seat and update
      if available_seats.is_empty() {
        return Ok(None);
      }

      // Pick a random available seat
      use rand::Rng;
      let mut rng = rand::rng();
      let random_index = rng.random_range(0..available_seats.len());
      let random_seat = available_seats[random_index];

      // Update seat directly
      self
        .room_repository
        .update_participant_seat(room_id, user_id, Some(random_seat))
        .await?;

      Some(random_seat)
    };

    // Verify the seat is still available (race condition check)
    if let Some(seat) = random_seat {
      if let Some(existing) = self.room_repository.get_participant_by_seat(room_id, seat).await?
        && existing.user_id() != user_id
      {
        return Ok(None); // Seat was taken between checking and attempting
      }

      // Check if room is full
      let sitting_count = self.room_repository.count_sitting_participants(room_id).await?;
      if sitting_count > room.max_players().value() {
        return Ok(None);
      }

      Ok(Some(seat))
    } else {
      Ok(None)
    }
  }
}
