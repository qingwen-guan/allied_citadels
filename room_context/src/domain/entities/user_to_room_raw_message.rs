use account_context::UserId;
use sqlx::Row;

use crate::domain::valueobjects::{MessageContent, MessageTopic, RoomId, UserToRoomMessageId};

#[derive(Debug, Clone)]
pub struct UserToRoomRawMessage {
  id: UserToRoomMessageId,
  room_id: RoomId,
  user_id: UserId,
  topic: MessageTopic,
  content: MessageContent,
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for UserToRoomRawMessage {
  fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
    Ok(UserToRoomRawMessage {
      id: row.try_get::<uuid::Uuid, _>("id")?.into(),
      room_id: row.try_get("room_id")?,
      user_id: row.try_get("user_id")?,
      topic: row.try_get::<String, _>("topic")?.into(),
      content: row.try_get::<String, _>("content")?.into(),
    })
  }
}

impl UserToRoomRawMessage {
  pub fn new(
    id: UserToRoomMessageId, room_id: RoomId, user_id: UserId, topic: MessageTopic, content: MessageContent,
  ) -> Self {
    Self {
      id,
      room_id,
      user_id,
      topic,
      content,
    }
  }

  pub fn id(&self) -> UserToRoomMessageId {
    self.id
  }

  pub fn room_id(&self) -> RoomId {
    self.room_id
  }

  pub fn user_id(&self) -> UserId {
    self.user_id
  }

  pub fn topic(&self) -> &MessageTopic {
    &self.topic
  }

  pub fn content(&self) -> &MessageContent {
    &self.content
  }
}
