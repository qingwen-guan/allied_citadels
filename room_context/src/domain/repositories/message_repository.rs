use async_trait::async_trait;
use user_context::domain::valueobjects::UserId;

use crate::domain::entities::RoomToUserRawMessage;
use crate::domain::valueobjects::{MessageTopic, RoomId, RoomToUserMessageId};
use crate::error::RoomError;

/// RawMessageRepository trait - interface for raw message data access
#[async_trait]
pub trait RawMessageRepository: Send + Sync {
  // Room to User messages
  async fn insert_room_to_user_raw_message(&self, message: RoomToUserRawMessage) -> Result<(), RoomError>;
  async fn batch_insert_room_to_user_raw_messages(&self, messages: &[RoomToUserRawMessage]) -> Result<(), RoomError>;
  async fn query_next_unread_room_to_user_raw_message(
    &self, room_id: RoomId, user_id: UserId, topic: Option<&MessageTopic>,
  ) -> Result<Option<RoomToUserRawMessage>, RoomError>;
  async fn batch_query_next_unread_room_to_user_raw_messages(
    &self, room_id: RoomId, user_id: UserId, topic: Option<&MessageTopic>, limit: usize,
  ) -> Result<Vec<RoomToUserRawMessage>, RoomError>;
  async fn mark_room_to_user_raw_message_as_read(
    &self, message_id: RoomToUserMessageId, user_id: UserId,
  ) -> Result<bool, RoomError>;
}
