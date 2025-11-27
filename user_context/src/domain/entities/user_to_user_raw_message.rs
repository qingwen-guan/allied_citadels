use sqlx::Row;

use crate::domain::valueobjects::{UserId, UserToUserMessageId};

/// UserToUserRawMessage - raw database representation of a user-to-user message
#[derive(Debug, Clone)]
pub struct UserToUserRawMessage {
  id: UserToUserMessageId,
  from_user_id: UserId,
  to_user_id: UserId,
  topic: String,
  content: String,
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for UserToUserRawMessage {
  fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
    Ok(UserToUserRawMessage {
      id: row.try_get::<uuid::Uuid, _>("id")?.into(),
      from_user_id: row.try_get("from_user_id")?,
      to_user_id: row.try_get("to_user_id")?,
      topic: row.try_get("topic")?,
      content: row.try_get("content")?,
    })
  }
}

impl UserToUserRawMessage {
  pub fn new(
    id: UserToUserMessageId, from_user_id: UserId, to_user_id: UserId, topic: String, content: String,
  ) -> Self {
    Self {
      id,
      from_user_id,
      to_user_id,
      topic,
      content,
    }
  }

  pub fn id(&self) -> UserToUserMessageId {
    self.id
  }

  pub fn from_id(&self) -> UserId {
    self.from_user_id
  }

  pub fn to_id(&self) -> UserId {
    self.to_user_id
  }

  pub fn topic(&self) -> &str {
    &self.topic
  }

  pub fn content(&self) -> &str {
    &self.content
  }

  // Deprecated: Use from_id() instead
  #[deprecated(note = "Use from_id() instead. Database migration required.")]
  pub fn sender_id(&self) -> UserId {
    self.from_user_id
  }

  // Deprecated: Use to_id() instead
  #[deprecated(note = "Use to_id() instead. Database migration required.")]
  pub fn receiver_id(&self) -> UserId {
    self.to_user_id
  }
}
