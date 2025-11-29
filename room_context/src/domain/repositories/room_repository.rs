use async_trait::async_trait;
use common_context::domain::valueobjects::Pagination;
use user_context::UserId;

use crate::domain::entities::{Room, RoomParticipant};
use crate::domain::valueobjects::{MaxPlayers, RoomId, RoomName, RoomNumber, SeatNumber};
use crate::error::RoomError;

/// RoomRepository trait - interface for room data access
#[async_trait]
pub trait RoomRepository: Send + Sync {
  async fn find_by_id(&self, id: RoomId) -> Result<Option<Room>, RoomError>;
  async fn find_by_name(&self, name: &RoomName) -> Result<Vec<Room>, RoomError>;
  async fn find_all(&self, pagination: Pagination) -> Result<Vec<Room>, RoomError>;
  async fn find_active(&self, pagination: Pagination) -> Result<Vec<Room>, RoomError>;
  async fn create(&self, creator: UserId, name: &RoomName, max_players: MaxPlayers) -> Result<Room, RoomError>;
  async fn update_name(&self, id: RoomId, new_name: &RoomName) -> Result<bool, RoomError>;
  async fn update_max_players(&self, id: RoomId, max_players: MaxPlayers) -> Result<bool, RoomError>;
  async fn delete(&self, id: RoomId) -> Result<bool, RoomError>;
  async fn get_next_room_number(&self) -> Result<RoomNumber, RoomError>;

  // Room participant methods
  async fn add_participant(
    &self, room_id: RoomId, user_id: UserId, seat_number: Option<SeatNumber>, viewing_seat_number: Option<SeatNumber>,
  ) -> Result<RoomParticipant, RoomError>;
  async fn remove_participant(&self, room_id: RoomId, user_id: UserId) -> Result<bool, RoomError>;
  async fn get_participants(&self, room_id: RoomId) -> Result<Vec<RoomParticipant>, RoomError>;
  async fn get_participant(&self, room_id: RoomId, user_id: UserId) -> Result<Option<RoomParticipant>, RoomError>;
  async fn get_participant_by_seat(
    &self, room_id: RoomId, seat_number: SeatNumber,
  ) -> Result<Option<RoomParticipant>, RoomError>;
  async fn update_participant_seat(
    &self, room_id: RoomId, user_id: UserId, new_seat: Option<SeatNumber>,
  ) -> Result<bool, RoomError>;
  async fn update_participant_viewing(
    &self, room_id: RoomId, user_id: UserId, viewing_seat: Option<SeatNumber>,
  ) -> Result<bool, RoomError>;
  async fn stand_up_participant(&self, room_id: RoomId, user_id: UserId) -> Result<bool, RoomError>;
  async fn count_participants(&self, room_id: RoomId) -> Result<usize, RoomError>;
  async fn count_sitting_participants(&self, room_id: RoomId) -> Result<usize, RoomError>;
}
