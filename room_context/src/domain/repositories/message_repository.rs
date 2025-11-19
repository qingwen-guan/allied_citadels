use async_trait::async_trait;
use user_context::UserId;

use crate::domain::entities::{RoomToUserRawMessage, UserToRoomRawMessage};
use crate::domain::valueobjects::{MessageTopic, RoomId, RoomToUserMessageId, UserToRoomMessageId};
use crate::error::RoomError;

/// RawMessageRepository trait - interface for raw message data access
#[async_trait]
pub trait RawMessageRepository: Send + Sync {
  // Room to User messages
  async fn insert_room_to_user_raw_message(
    &self, message: RoomToUserRawMessage,
  ) -> Result<RoomToUserRawMessage, RoomError>;
  async fn batch_insert_room_to_user_raw_messages(
    &self,
    messages: Vec<RoomToUserRawMessage>, // TODO: take [RoomToUserRawMessage] as paramater
  ) -> Result<Vec<RoomToUserRawMessage>, RoomError>;
  async fn query_next_unread_room_to_user_raw_message(
    &self, room_id: RoomId, user_id: UserId, topic: Option<&MessageTopic>,
  ) -> Result<Option<RoomToUserRawMessage>, RoomError>;
  async fn batch_query_next_unread_room_to_user_raw_messages(
    &self, room_id: RoomId, user_id: UserId, topic: Option<&MessageTopic>, limit: usize,
  ) -> Result<Vec<RoomToUserRawMessage>, RoomError>;
  async fn mark_room_to_user_raw_message_as_read(
    &self, message_id: RoomToUserMessageId, user_id: UserId,
  ) -> Result<bool, RoomError>;

  // User to Room messages
  async fn insert_user_to_room_raw_message(
    &self, message: UserToRoomRawMessage,
  ) -> Result<UserToRoomRawMessage, RoomError>;
  async fn batch_insert_user_to_room_raw_messages(
    &self, messages: Vec<UserToRoomRawMessage>,
  ) -> Result<Vec<UserToRoomRawMessage>, RoomError>;
  async fn query_next_unread_user_to_room_raw_message(
    &self, room_id: RoomId, topic: Option<&MessageTopic>,
  ) -> Result<Option<UserToRoomRawMessage>, RoomError>;
  async fn batch_query_next_unread_user_to_room_raw_messages(
    &self, room_id: RoomId, topic: Option<&MessageTopic>, limit: usize,
  ) -> Result<Vec<UserToRoomRawMessage>, RoomError>;
  async fn mark_user_to_room_raw_message_as_read(
    &self, message_id: UserToRoomMessageId, room_id: RoomId,
  ) -> Result<bool, RoomError>;
}
