use crate::domain::entities::RoomToAccountMessage;
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

  /// Insert a single room to account message
  #[allow(dead_code)] // TODO: remove later
  pub async fn insert_room_to_account_message(&self, message: RoomToAccountMessage) -> Result<(), RoomError> {
    let room_id = message.room_id();
    let account_id = message.account_id();
    let topic = message.topic();
    let content = message.content();
    self
      .message_repository
      .insert_room_to_account_raw_message(room_id, account_id, topic, content)
      .await?;
    Ok(())
  }

  /// Batch insert multiple room to account messages
  pub async fn batch_insert_room_to_account_message(
    &self, messages: Vec<RoomToAccountMessage>,
  ) -> Result<(), RoomError> {
    let raw_messages: Vec<_> = messages
      .into_iter()
      .map(|message| {
        let room_id = message.room_id();
        let account_id = message.account_id();
        let topic = message.topic();
        let content = message.content();
        (room_id, account_id, topic, content)
      })
      .collect();

    self
      .message_repository
      .batch_insert_room_to_account_raw_messages(raw_messages)
      .await?;
    Ok(())
  }
}
