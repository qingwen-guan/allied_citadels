use account_context::AccountId;
use sqlx::PgPool;

use crate::domain::repositories::RawMessageRepository;
use crate::domain::valueobjects::{
  AccountToRoomMessageId, MessageContent, MessageTopic, RoomId, RoomToAccountMessageId,
};
use crate::domain::{AccountToRoomRawMessage, RoomToAccountRawMessage};
use crate::error::RoomError;

/// PostgreSQL implementation of RawMessageRepository
pub struct PostgresMessageRepository {
  pool: PgPool,
}

impl PostgresMessageRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait::async_trait]
impl RawMessageRepository for PostgresMessageRepository {
  async fn insert_room_to_account_raw_message(
    &self, room_id: RoomId, account_id: AccountId, topic: MessageTopic, content: MessageContent,
  ) -> Result<RoomToAccountRawMessage, RoomError> {
    let id = RoomToAccountMessageId::from(uuid::Uuid::new_v4());
    let created_at = chrono::Utc::now();

    sqlx::query(
      "INSERT INTO room_to_account_message (id, room_id, account_id, topic, content, created_at)
       VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(room_id)
    .bind(account_id)
    .bind(&topic)
    .bind(&content)
    .bind(created_at)
    .execute(&self.pool)
    .await?;

    Ok(RoomToAccountRawMessage::new(
      id, room_id, account_id, topic, content, created_at,
    ))
  }

  async fn batch_insert_room_to_account_raw_messages(
    &self, messages: Vec<(RoomId, AccountId, MessageTopic, MessageContent)>,
  ) -> Result<Vec<RoomToAccountRawMessage>, RoomError> {
    if messages.is_empty() {
      return Ok(Vec::new());
    }

    let created_at = chrono::Utc::now();
    let mut results = Vec::with_capacity(messages.len());
    let mut query_builder = sqlx::QueryBuilder::new(
      "INSERT INTO room_to_account_message (id, room_id, account_id, topic, content, created_at) ",
    );

    // Collect all data first to avoid borrowing issues
    let data: Vec<(RoomToAccountMessageId, RoomId, AccountId, MessageTopic, MessageContent)> = messages
      .into_iter()
      .map(|(room_id, account_id, topic, content)| {
        let id = RoomToAccountMessageId::from(uuid::Uuid::new_v4());
        results.push(RoomToAccountRawMessage::new(
          id,
          room_id,
          account_id,
          topic.clone(),
          content.clone(),
          created_at,
        ));
        (id, room_id, account_id, topic, content)
      })
      .collect();

    query_builder.push_values(data, |mut b, (id, room_id, account_id, topic, content)| {
      b.push_bind(id)
        .push_bind(room_id)
        .push_bind(account_id)
        .push_bind(topic)
        .push_bind(content)
        .push_bind(created_at);
    });

    let query = query_builder.build();
    query.execute(&self.pool).await?;

    Ok(results)
  }

  async fn query_next_unread_room_to_account_raw_message(
    &self, room_id: RoomId, account_id: AccountId, topic: Option<&MessageTopic>,
  ) -> Result<Option<RoomToAccountRawMessage>, RoomError> {
    let message = if let Some(topic) = topic {
      sqlx::query_as::<_, RoomToAccountRawMessage>(
        "SELECT id, room_id, account_id, topic, content, created_at, read_at
         FROM room_to_account_message
         WHERE room_id = $1 AND account_id = $2 AND topic = $3 AND read_at IS NULL
         ORDER BY created_at ASC
         LIMIT 1",
      )
      .bind(room_id)
      .bind(account_id)
      .bind(topic)
      .fetch_optional(&self.pool)
      .await?
    } else {
      sqlx::query_as::<_, RoomToAccountRawMessage>(
        "SELECT id, room_id, account_id, topic, content, created_at, read_at
         FROM room_to_account_message
         WHERE room_id = $1 AND account_id = $2 AND read_at IS NULL
         ORDER BY created_at ASC
         LIMIT 1",
      )
      .bind(room_id)
      .bind(account_id)
      .fetch_optional(&self.pool)
      .await?
    };

    Ok(message)
  }

  async fn batch_query_next_unread_room_to_account_raw_messages(
    &self, room_id: RoomId, account_id: AccountId, topic: Option<&MessageTopic>, limit: usize,
  ) -> Result<Vec<RoomToAccountRawMessage>, RoomError> {
    let messages = if let Some(topic) = topic {
      sqlx::query_as::<_, RoomToAccountRawMessage>(
        "SELECT id, room_id, account_id, topic, content, created_at, read_at
         FROM room_to_account_message
         WHERE room_id = $1 AND account_id = $2 AND topic = $3 AND read_at IS NULL
         ORDER BY created_at ASC
         LIMIT $4",
      )
      .bind(room_id)
      .bind(account_id)
      .bind(topic)
      .bind(limit as i64)
      .fetch_all(&self.pool)
      .await?
    } else {
      sqlx::query_as::<_, RoomToAccountRawMessage>(
        "SELECT id, room_id, account_id, topic, content, created_at, read_at
         FROM room_to_account_message
         WHERE room_id = $1 AND account_id = $2 AND read_at IS NULL
         ORDER BY created_at ASC
         LIMIT $3",
      )
      .bind(room_id)
      .bind(account_id)
      .bind(limit as i64)
      .fetch_all(&self.pool)
      .await?
    };

    Ok(messages)
  }

  async fn mark_room_to_account_raw_message_as_read(
    &self, message_id: RoomToAccountMessageId, account_id: AccountId,
  ) -> Result<bool, RoomError> {
    let read_at = chrono::Utc::now();
    let rows_affected = sqlx::query(
      "UPDATE room_to_account_message
       SET read_at = $1
       WHERE id = $2 AND account_id = $3 AND read_at IS NULL",
    )
    .bind(read_at)
    .bind(message_id)
    .bind(account_id)
    .execute(&self.pool)
    .await?
    .rows_affected();

    Ok(rows_affected > 0)
  }

  async fn insert_account_to_room_raw_message(
    &self, room_id: RoomId, account_id: AccountId, topic: MessageTopic, content: MessageContent,
  ) -> Result<AccountToRoomRawMessage, RoomError> {
    let id = AccountToRoomMessageId::from(uuid::Uuid::new_v4());
    let created_at = chrono::Utc::now();

    sqlx::query(
      "INSERT INTO account_to_room_message (id, room_id, account_id, topic, content, created_at)
       VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(room_id)
    .bind(account_id)
    .bind(&topic)
    .bind(&content)
    .bind(created_at)
    .execute(&self.pool)
    .await?;

    Ok(AccountToRoomRawMessage::new(
      id, room_id, account_id, topic, content, created_at,
    ))
  }

  async fn batch_insert_account_to_room_raw_messages(
    &self, messages: Vec<(RoomId, AccountId, MessageTopic, MessageContent)>,
  ) -> Result<Vec<AccountToRoomRawMessage>, RoomError> {
    if messages.is_empty() {
      return Ok(Vec::new());
    }

    let created_at = chrono::Utc::now();
    let mut results = Vec::with_capacity(messages.len());
    let mut query_builder = sqlx::QueryBuilder::new(
      "INSERT INTO account_to_room_message (id, room_id, account_id, topic, content, created_at) ",
    );

    // Collect all data first to avoid borrowing issues
    let data: Vec<(AccountToRoomMessageId, RoomId, AccountId, MessageTopic, MessageContent)> = messages
      .into_iter()
      .map(|(room_id, account_id, topic, content)| {
        let id = AccountToRoomMessageId::from(uuid::Uuid::new_v4());
        results.push(AccountToRoomRawMessage::new(
          id,
          room_id,
          account_id,
          topic.clone(),
          content.clone(),
          created_at,
        ));
        (id, room_id, account_id, topic, content)
      })
      .collect();

    query_builder.push_values(data, |mut b, (id, room_id, account_id, topic, content)| {
      b.push_bind(id)
        .push_bind(room_id)
        .push_bind(account_id)
        .push_bind(topic)
        .push_bind(content)
        .push_bind(created_at);
    });

    let query = query_builder.build();
    query.execute(&self.pool).await?;

    Ok(results)
  }

  async fn query_next_unread_account_to_room_raw_message(
    &self, room_id: RoomId, topic: Option<&MessageTopic>,
  ) -> Result<Option<AccountToRoomRawMessage>, RoomError> {
    let message = if let Some(topic) = topic {
      sqlx::query_as::<_, AccountToRoomRawMessage>(
        "SELECT id, room_id, account_id, topic, content, created_at, read_at
         FROM account_to_room_message
         WHERE room_id = $1 AND topic = $2 AND read_at IS NULL
         ORDER BY created_at ASC
         LIMIT 1",
      )
      .bind(room_id)
      .bind(topic)
      .fetch_optional(&self.pool)
      .await?
    } else {
      sqlx::query_as::<_, AccountToRoomRawMessage>(
        "SELECT id, room_id, account_id, topic, content, created_at, read_at
         FROM account_to_room_message
         WHERE room_id = $1 AND read_at IS NULL
         ORDER BY created_at ASC
         LIMIT 1",
      )
      .bind(room_id)
      .fetch_optional(&self.pool)
      .await?
    };

    Ok(message)
  }

  async fn batch_query_next_unread_account_to_room_raw_messages(
    &self, room_id: RoomId, topic: Option<&MessageTopic>, limit: usize,
  ) -> Result<Vec<AccountToRoomRawMessage>, RoomError> {
    let messages = if let Some(topic) = topic {
      sqlx::query_as::<_, AccountToRoomRawMessage>(
        "SELECT id, room_id, account_id, topic, content, created_at, read_at
         FROM account_to_room_message
         WHERE room_id = $1 AND topic = $2 AND read_at IS NULL
         ORDER BY created_at ASC
         LIMIT $3",
      )
      .bind(room_id)
      .bind(topic)
      .bind(limit as i64)
      .fetch_all(&self.pool)
      .await?
    } else {
      sqlx::query_as::<_, AccountToRoomRawMessage>(
        "SELECT id, room_id, account_id, topic, content, created_at, read_at
         FROM account_to_room_message
         WHERE room_id = $1 AND read_at IS NULL
         ORDER BY created_at ASC
         LIMIT $2",
      )
      .bind(room_id)
      .bind(limit as i64)
      .fetch_all(&self.pool)
      .await?
    };

    Ok(messages)
  }

  async fn mark_account_to_room_raw_message_as_read(
    &self, message_id: AccountToRoomMessageId, room_id: RoomId,
  ) -> Result<bool, RoomError> {
    let read_at = chrono::Utc::now();
    let rows_affected = sqlx::query(
      "UPDATE account_to_room_message
       SET read_at = $1
       WHERE id = $2 AND room_id = $3 AND read_at IS NULL",
    )
    .bind(read_at)
    .bind(message_id)
    .bind(room_id)
    .execute(&self.pool)
    .await?
    .rows_affected();

    Ok(rows_affected > 0)
  }
}
