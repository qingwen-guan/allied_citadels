use sqlx::PgPool;
use user_context::domain::valueobjects::UserId;

use crate::domain::RoomToUserRawMessage;
use crate::domain::repositories::RawMessageRepository;
use crate::domain::valueobjects::{MessageContent, MessageTopic, RoomId, RoomToUserMessageId};
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
  async fn insert_room_to_user_raw_message(&self, message: RoomToUserRawMessage) -> Result<(), RoomError> {
    let id = message.id();

    sqlx::query(
      "INSERT INTO room_to_user_message (id, from_room_id, to_user_id, topic, content, created_at)
       VALUES ($1, $2, $3, $4, $5, NOW())",
    )
    .bind(id)
    .bind(message.room_id())
    .bind(message.user_id())
    .bind(message.topic())
    .bind(message.content())
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn batch_insert_room_to_user_raw_messages(&self, messages: &[RoomToUserRawMessage]) -> Result<(), RoomError> {
    if messages.is_empty() {
      return Ok(());
    }

    let mut query_builder = sqlx::QueryBuilder::new(
      "INSERT INTO room_to_user_message (id, from_room_id, to_user_id, topic, content, created_at) ",
    );

    // Collect all data first to avoid borrowing issues
    let data: Vec<(RoomToUserMessageId, RoomId, UserId, MessageTopic, MessageContent)> = messages
      .iter()
      .map(|message| {
        let id = message.id();
        let room_id = message.room_id();
        let user_id = message.user_id();
        let topic = message.topic().clone();
        let content = message.content().clone();
        (id, room_id, user_id, topic, content)
      })
      .collect();

    query_builder.push_values(data, |mut b, (id, room_id, user_id, topic, content)| {
      b.push_bind(id)
        .push_bind(room_id)
        .push_bind(user_id)
        .push_bind(topic)
        .push_bind(content)
        .push("NOW()");
    });

    let query = query_builder.build();
    query.execute(&self.pool).await?;

    Ok(())
  }

  async fn query_next_unread_room_to_user_raw_message(
    &self, room_id: RoomId, user_id: UserId, topic: Option<&MessageTopic>,
  ) -> Result<Option<RoomToUserRawMessage>, RoomError> {
    let message = if let Some(topic) = topic {
      sqlx::query_as::<_, RoomToUserRawMessage>(
        "SELECT id, from_room_id AS room_id, to_user_id AS user_id, topic, content
         FROM room_to_user_message
         WHERE from_room_id = $1 AND to_user_id = $2 AND topic = $3 AND read_at IS NULL
         ORDER BY created_at ASC
         LIMIT 1",
      )
      .bind(room_id)
      .bind(user_id)
      .bind(topic)
      .fetch_optional(&self.pool)
      .await?
    } else {
      sqlx::query_as::<_, RoomToUserRawMessage>(
        "SELECT id, from_room_id AS room_id, to_user_id AS user_id, topic, content
         FROM room_to_user_message
         WHERE from_room_id = $1 AND to_user_id = $2 AND read_at IS NULL
         ORDER BY created_at ASC
         LIMIT 1",
      )
      .bind(room_id)
      .bind(user_id)
      .fetch_optional(&self.pool)
      .await?
    };

    Ok(message)
  }

  async fn batch_query_next_unread_room_to_user_raw_messages(
    &self, room_id: RoomId, user_id: UserId, topic: Option<&MessageTopic>, limit: usize,
  ) -> Result<Vec<RoomToUserRawMessage>, RoomError> {
    let messages = if let Some(topic) = topic {
      sqlx::query_as::<_, RoomToUserRawMessage>(
        "SELECT id, from_room_id AS room_id, to_user_id AS user_id, topic, content
         FROM room_to_user_message
         WHERE from_room_id = $1 AND to_user_id = $2 AND topic = $3 AND read_at IS NULL
         ORDER BY created_at ASC
         LIMIT $4",
      )
      .bind(room_id)
      .bind(user_id)
      .bind(topic)
      .bind(limit as i64)
      .fetch_all(&self.pool)
      .await?
    } else {
      sqlx::query_as::<_, RoomToUserRawMessage>(
        "SELECT id, from_room_id AS room_id, to_user_id AS user_id, topic, content
         FROM room_to_user_message
         WHERE from_room_id = $1 AND to_user_id = $2 AND read_at IS NULL
         ORDER BY created_at ASC
         LIMIT $3",
      )
      .bind(room_id)
      .bind(user_id)
      .bind(limit as i64)
      .fetch_all(&self.pool)
      .await?
    };

    Ok(messages)
  }

  async fn mark_room_to_user_raw_message_as_read(
    &self, message_id: RoomToUserMessageId, user_id: UserId,
  ) -> Result<bool, RoomError> {
    let read_at = chrono::Utc::now();
    let rows_affected = sqlx::query(
      "UPDATE room_to_user_message
       SET read_at = $1
       WHERE id = $2 AND to_user_id = $3 AND read_at IS NULL",
    )
    .bind(read_at)
    .bind(message_id)
    .bind(user_id)
    .execute(&self.pool)
    .await?
    .rows_affected();

    Ok(rows_affected > 0)
  }
}
