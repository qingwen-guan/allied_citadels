use crate::domain::entities::{RoomToUserMessage, RoomToUserRawMessage};
use crate::domain::repositories::RawMessageRepository;
use crate::error::RoomError;

/// MessageManager - manages message operations using the message repository
pub struct MessageManager {
  message_repository: Box<dyn RawMessageRepository>,
}

impl MessageManager {
  pub fn new(message_repository: Box<dyn RawMessageRepository>) -> Self {
    Self { message_repository }
  }

  /// Batch insert multiple room to user messages
  pub async fn batch_insert_room_to_user_message(&self, messages: Vec<RoomToUserMessage>) -> Result<(), RoomError> {
    let raw_messages: Vec<RoomToUserRawMessage> = messages
      .into_iter()
      .map(|message| {
        let room_id = message.room_id();
        let user_id = message.user_id();
        let topic = message.topic();
        let content = message.content();
        RoomToUserRawMessage::without_id(room_id, user_id, topic, content)
      })
      .collect();

    self
      .message_repository
      .batch_insert_room_to_user_raw_messages(&raw_messages)
      .await?;
    Ok(())
  }
}
