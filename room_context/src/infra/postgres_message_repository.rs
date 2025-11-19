use sqlx::PgPool;
use user_context::UserId;

use crate::domain::repositories::RawMessageRepository;
use crate::domain::valueobjects::{MessageContent, MessageTopic, RoomId, RoomToUserMessageId, UserToRoomMessageId};
use crate::domain::{RoomToUserRawMessage, UserToRoomRawMessage};
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
  async fn insert_room_to_user_raw_message(
    &self, message: RoomToUserRawMessage,
  ) -> Result<RoomToUserRawMessage, RoomError> {
    let id = RoomToUserMessageId::from(uuid::Uuid::new_v4());

    sqlx::query(
      "INSERT INTO room_to_user_message (id, room_id, user_id, topic, content, created_at)
       VALUES ($1, $2, $3, $4, $5, NOW())",
    )
    .bind(id) // TODO: use message.id()
    .bind(message.room_id())
    .bind(message.user_id())
    .bind(message.topic())
    .bind(message.content())
    .execute(&self.pool)
    .await?;

    Ok(RoomToUserRawMessage::new(
      id,
      message.room_id(),
      message.user_id(),
      message.topic().clone(),
      message.content().clone(),
    ))
  }

  async fn batch_insert_room_to_user_raw_messages(
    &self, messages: Vec<RoomToUserRawMessage>,
  ) -> Result<Vec<RoomToUserRawMessage>, RoomError> {
    if messages.is_empty() {
      return Ok(Vec::new());
    }

    let mut results = Vec::with_capacity(messages.len());
    let mut query_builder =
      sqlx::QueryBuilder::new("INSERT INTO room_to_user_message (id, room_id, user_id, topic, content, created_at) ");

    // Collect all data first to avoid borrowing issues
    let data: Vec<(RoomToUserMessageId, RoomId, UserId, MessageTopic, MessageContent)> = messages
      .into_iter()
      .map(|message| {
        let id = RoomToUserMessageId::from(uuid::Uuid::new_v4());
        let room_id = message.room_id();
        let user_id = message.user_id();
        let topic = message.topic().clone();
        let content = message.content().clone();
        results.push(RoomToUserRawMessage::new(
          id,
          room_id,
          user_id,
          topic.clone(),
          content.clone(),
        ));
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

    Ok(results)
  }

  async fn query_next_unread_room_to_user_raw_message(
    &self, room_id: RoomId, user_id: UserId, topic: Option<&MessageTopic>,
  ) -> Result<Option<RoomToUserRawMessage>, RoomError> {
    let message = if let Some(topic) = topic {
      sqlx::query_as::<_, RoomToUserRawMessage>(
        "SELECT id, room_id, user_id, topic, content
         FROM room_to_user_message
         WHERE room_id = $1 AND user_id = $2 AND topic = $3 AND read_at IS NULL
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
        "SELECT id, room_id, user_id, topic, content
         FROM room_to_user_message
         WHERE room_id = $1 AND user_id = $2 AND read_at IS NULL
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
        "SELECT id, room_id, user_id, topic, content
         FROM room_to_user_message
         WHERE room_id = $1 AND user_id = $2 AND topic = $3 AND read_at IS NULL
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
        "SELECT id, room_id, user_id, topic, content
         FROM room_to_user_message
         WHERE room_id = $1 AND user_id = $2 AND read_at IS NULL
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
       WHERE id = $2 AND user_id = $3 AND read_at IS NULL",
    )
    .bind(read_at)
    .bind(message_id)
    .bind(user_id)
    .execute(&self.pool)
    .await?
    .rows_affected();

    Ok(rows_affected > 0)
  }

  async fn insert_user_to_room_raw_message(
    &self, message: UserToRoomRawMessage,
  ) -> Result<UserToRoomRawMessage, RoomError> {
    // TODO: return ()
    let id = UserToRoomMessageId::from(uuid::Uuid::new_v4());

    sqlx::query(
      "INSERT INTO user_to_room_message (id, room_id, user_id, topic, content, created_at)
       VALUES ($1, $2, $3, $4, $5, NOW())",
    )
    .bind(id) // TODO: use message.id()
    .bind(message.room_id())
    .bind(message.user_id())
    .bind(message.topic())
    .bind(message.content())
    .execute(&self.pool)
    .await?;

    Ok(UserToRoomRawMessage::new(
      id,
      message.room_id(),
      message.user_id(),
      message.topic().clone(),
      message.content().clone(),
    ))
  }

  async fn batch_insert_user_to_room_raw_messages(
    &self,
    messages: Vec<UserToRoomRawMessage>, // TODO: take [UserToRoomRawMessage] as paramater
  ) -> Result<Vec<UserToRoomRawMessage>, RoomError> {
    if messages.is_empty() {
      return Ok(Vec::new());
    }

    let mut results = Vec::with_capacity(messages.len());
    let mut query_builder =
      sqlx::QueryBuilder::new("INSERT INTO user_to_room_message (id, room_id, user_id, topic, content, created_at) ");

    // Collect all data first to avoid borrowing issues
    let data: Vec<(UserToRoomMessageId, RoomId, UserId, MessageTopic, MessageContent)> = messages
      .into_iter()
      .map(|message| {
        let id = UserToRoomMessageId::from(uuid::Uuid::new_v4()); // TODO: use message.id()
        let room_id = message.room_id();
        let user_id = message.user_id();
        let topic = message.topic().clone();
        let content = message.content().clone();
        results.push(UserToRoomRawMessage::new(
          id,
          room_id,
          user_id,
          topic.clone(),
          content.clone(),
        ));
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

    Ok(results)
  }

  async fn query_next_unread_user_to_room_raw_message(
    &self, room_id: RoomId, topic: Option<&MessageTopic>,
  ) -> Result<Option<UserToRoomRawMessage>, RoomError> {
    let message = if let Some(topic) = topic {
      sqlx::query_as::<_, UserToRoomRawMessage>(
        "SELECT id, room_id, user_id, topic, content
         FROM user_to_room_message
         WHERE room_id = $1 AND topic = $2 AND read_at IS NULL
         ORDER BY created_at ASC
         LIMIT 1",
      )
      .bind(room_id)
      .bind(topic)
      .fetch_optional(&self.pool)
      .await?
    } else {
      sqlx::query_as::<_, UserToRoomRawMessage>(
        "SELECT id, room_id, user_id, topic, content
         FROM user_to_room_message
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

  async fn batch_query_next_unread_user_to_room_raw_messages(
    &self, room_id: RoomId, topic: Option<&MessageTopic>, limit: usize,
  ) -> Result<Vec<UserToRoomRawMessage>, RoomError> {
    let messages = if let Some(topic) = topic {
      sqlx::query_as::<_, UserToRoomRawMessage>(
        "SELECT id, room_id, user_id, topic, content
         FROM user_to_room_message
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
      sqlx::query_as::<_, UserToRoomRawMessage>(
        "SELECT id, room_id, user_id, topic, content
         FROM user_to_room_message
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

  async fn mark_user_to_room_raw_message_as_read(
    &self, message_id: UserToRoomMessageId, room_id: RoomId,
  ) -> Result<bool, RoomError> {
    let read_at = chrono::Utc::now();
    let rows_affected = sqlx::query(
      "UPDATE user_to_room_message
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
