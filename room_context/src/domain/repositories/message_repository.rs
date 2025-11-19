use account_context::AccountId;
use async_trait::async_trait;

use crate::domain::entities::{AccountToRoomRawMessage, RoomToAccountRawMessage};
use crate::domain::valueobjects::{
  AccountToRoomMessageId, MessageContent, MessageTopic, RoomId, RoomToAccountMessageId,
};
use crate::error::RoomError;

/// RawMessageRepository trait - interface for raw message data access
#[async_trait]
pub trait RawMessageRepository: Send + Sync {
  // Room to Account messages
  async fn insert_room_to_account_raw_message(
    &self, room_id: RoomId, account_id: AccountId, topic: MessageTopic, content: MessageContent,
  ) -> Result<RoomToAccountRawMessage, RoomError>;
  async fn batch_insert_room_to_account_raw_messages(
    &self, messages: Vec<(RoomId, AccountId, MessageTopic, MessageContent)>,
  ) -> Result<Vec<RoomToAccountRawMessage>, RoomError>;
  async fn query_next_unread_room_to_account_raw_message(
    &self, room_id: RoomId, account_id: AccountId, topic: Option<&MessageTopic>,
  ) -> Result<Option<RoomToAccountRawMessage>, RoomError>;
  async fn batch_query_next_unread_room_to_account_raw_messages(
    &self, room_id: RoomId, account_id: AccountId, topic: Option<&MessageTopic>, limit: usize,
  ) -> Result<Vec<RoomToAccountRawMessage>, RoomError>;
  async fn mark_room_to_account_raw_message_as_read(
    &self, message_id: RoomToAccountMessageId, account_id: AccountId,
  ) -> Result<bool, RoomError>;

  // Account to Room messages
  async fn insert_account_to_room_raw_message(
    &self, room_id: RoomId, account_id: AccountId, topic: MessageTopic, content: MessageContent,
  ) -> Result<AccountToRoomRawMessage, RoomError>;
  async fn batch_insert_account_to_room_raw_messages(
    &self, messages: Vec<(RoomId, AccountId, MessageTopic, MessageContent)>,
  ) -> Result<Vec<AccountToRoomRawMessage>, RoomError>;
  async fn query_next_unread_account_to_room_raw_message(
    &self, room_id: RoomId, topic: Option<&MessageTopic>,
  ) -> Result<Option<AccountToRoomRawMessage>, RoomError>;
  async fn batch_query_next_unread_account_to_room_raw_messages(
    &self, room_id: RoomId, topic: Option<&MessageTopic>, limit: usize,
  ) -> Result<Vec<AccountToRoomRawMessage>, RoomError>;
  async fn mark_account_to_room_raw_message_as_read(
    &self, message_id: AccountToRoomMessageId, room_id: RoomId,
  ) -> Result<bool, RoomError>;
}
