use account_context::UserId;
use async_trait::async_trait;

use crate::domain::entities::{RoomToUserRawMessage, UserToRoomRawMessage};
use crate::domain::valueobjects::{MessageContent, MessageTopic, RoomId, RoomToUserMessageId, UserToRoomMessageId};
use crate::error::RoomError;

/// RawMessageRepository trait - interface for raw message data access
#[async_trait]
pub trait RawMessageRepository: Send + Sync {
  // Room to User messages
  async fn insert_room_to_user_raw_message(
    &self, room_id: RoomId, user_id: UserId, topic: MessageTopic, content: MessageContent,
  ) -> Result<RoomToUserRawMessage, RoomError>;
  async fn batch_insert_room_to_user_raw_messages(
    &self, messages: Vec<(RoomId, UserId, MessageTopic, MessageContent)>,
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
    &self, room_id: RoomId, user_id: UserId, topic: MessageTopic, content: MessageContent,
  ) -> Result<UserToRoomRawMessage, RoomError>;
  async fn batch_insert_user_to_room_raw_messages(
    &self, messages: Vec<(RoomId, UserId, MessageTopic, MessageContent)>,
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
