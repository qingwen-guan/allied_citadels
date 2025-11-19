use account_context::AccountId;
use sqlx::Row;

use crate::domain::valueobjects::{MessageContent, MessageTopic, RoomId, RoomToAccountMessageId};

#[derive(Debug, Clone)]
pub struct RoomToAccountRawMessage {
  id: RoomToAccountMessageId,
  room_id: RoomId,
  account_id: AccountId,
  topic: MessageTopic,
  content: MessageContent,
  created_at: chrono::DateTime<chrono::Utc>,
  read_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for RoomToAccountRawMessage {
  fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
    Ok(RoomToAccountRawMessage {
      id: row.try_get::<uuid::Uuid, _>("id")?.into(),
      room_id: row.try_get("room_id")?,
      account_id: row.try_get("account_id")?,
      topic: row.try_get::<String, _>("topic")?.into(),
      content: row.try_get::<String, _>("content")?.into(),
      created_at: row.try_get("created_at")?,
      read_at: row.try_get("read_at")?,
    })
  }
}

impl RoomToAccountRawMessage {
  pub fn new(
    id: RoomToAccountMessageId, room_id: RoomId, account_id: AccountId, topic: MessageTopic, content: MessageContent,
    created_at: chrono::DateTime<chrono::Utc>,
  ) -> Self {
    Self {
      id,
      room_id,
      account_id,
      topic,
      content,
      created_at,
      read_at: None,
    }
  }

  pub fn id(&self) -> RoomToAccountMessageId {
    self.id
  }

  pub fn room_id(&self) -> RoomId {
    self.room_id
  }

  pub fn account_id(&self) -> AccountId {
    self.account_id
  }

  pub fn topic(&self) -> &MessageTopic {
    &self.topic
  }

  pub fn content(&self) -> &MessageContent {
    &self.content
  }

  pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
    self.created_at
  }

  pub fn read_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
    self.read_at
  }

  pub fn is_read(&self) -> bool {
    self.read_at.is_some()
  }

  pub fn mark_as_read(&mut self, read_at: chrono::DateTime<chrono::Utc>) {
    self.read_at = Some(read_at);
  }
}
