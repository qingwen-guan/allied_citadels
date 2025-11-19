use sqlx::Row;

use crate::domain::valueobjects::{UserId, UserToUserMessageId};

/// UserToUserRawMessage - raw database representation of a user-to-user message
#[derive(Debug, Clone)]
pub struct UserToUserRawMessage {
  id: UserToUserMessageId,
  sender_id: UserId,   // TODO:rename to from
  receiver_id: UserId, // TODO: rename to to
  topic: String,
  content: String,
  created_at: chrono::DateTime<chrono::Utc>,      // TODO: remove this field
  read_at: Option<chrono::DateTime<chrono::Utc>>, // TODO: remove this field
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for UserToUserRawMessage {
  fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
    Ok(UserToUserRawMessage {
      id: row.try_get::<uuid::Uuid, _>("id")?.into(),
      sender_id: row.try_get("sender_id")?,
      receiver_id: row.try_get("receiver_id")?,
      topic: row.try_get("topic")?,
      content: row.try_get("content")?,
      created_at: row.try_get("created_at")?,
      read_at: row.try_get("read_at")?,
    })
  }
}

impl UserToUserRawMessage {
  pub fn new(
    id: UserToUserMessageId, sender_id: UserId, receiver_id: UserId, topic: String, content: String,
    created_at: chrono::DateTime<chrono::Utc>,
  ) -> Self {
    Self {
      id,
      sender_id,
      receiver_id,
      topic,
      content,
      created_at,
      read_at: None,
    }
  }

  pub fn id(&self) -> UserToUserMessageId {
    self.id
  }

  pub fn sender_id(&self) -> UserId {
    self.sender_id
  }

  pub fn receiver_id(&self) -> UserId {
    self.receiver_id
  }

  pub fn topic(&self) -> &str {
    &self.topic
  }

  pub fn content(&self) -> &str {
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

  // TODO: remove this fn
  pub fn mark_as_read(&mut self, read_at: chrono::DateTime<chrono::Utc>) {
    self.read_at = Some(read_at);
  }
}
