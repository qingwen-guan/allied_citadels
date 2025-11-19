use sqlx::Row;

use crate::domain::valueobjects::{AccountId, AccountToAccountMessageId};

/// AccountToAccountRawMessage - raw database representation of an account-to-account message
#[derive(Debug, Clone)]
pub struct AccountToAccountRawMessage {
  id: AccountToAccountMessageId,
  sender_id: AccountId,
  receiver_id: AccountId,
  topic: String,
  content: String,
  created_at: chrono::DateTime<chrono::Utc>,
  read_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for AccountToAccountRawMessage {
  fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
    Ok(AccountToAccountRawMessage {
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

impl AccountToAccountRawMessage {
  pub fn new(
    id: AccountToAccountMessageId, sender_id: AccountId, receiver_id: AccountId, topic: String, content: String,
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

  pub fn id(&self) -> AccountToAccountMessageId {
    self.id
  }

  pub fn sender_id(&self) -> AccountId {
    self.sender_id
  }

  pub fn receiver_id(&self) -> AccountId {
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

  pub fn mark_as_read(&mut self, read_at: chrono::DateTime<chrono::Utc>) {
    self.read_at = Some(read_at);
  }
}
