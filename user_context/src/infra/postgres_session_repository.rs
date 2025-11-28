use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::valueobjects::{SessionId, UserId};
use crate::domain::{SessionInfo, SessionRepository};
use crate::error::UserError;

/// PostgreSQL implementation of SessionRepository
pub struct PostgresSessionRepository {
  pool: PgPool,
}

impl PostgresSessionRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait::async_trait]
impl SessionRepository for PostgresSessionRepository {
  async fn create(
    &self, session_id: SessionId, user_id: UserId, expires_at: chrono::DateTime<chrono::Utc>,
  ) -> Result<(), UserError> {
    sqlx::query("INSERT INTO user_session (id, user_id, expires_at) VALUES ($1, $2, $3)")
      .bind(session_id)
      .bind(user_id)
      .bind(expires_at)
      .execute(&self.pool)
      .await?;

    Ok(())
  }

  async fn find_by_session_id(
    &self, session_id: SessionId,
  ) -> Result<Option<(UserId, chrono::DateTime<chrono::Utc>)>, UserError> {
    #[derive(sqlx::FromRow)]
    struct SessionRow {
      user_id: Uuid,
      expires_at: chrono::DateTime<chrono::Utc>,
    }

    let result: Option<SessionRow> = sqlx::query_as("SELECT user_id, expires_at FROM user_session WHERE id = $1")
      .bind(session_id)
      .fetch_optional(&self.pool)
      .await?;

    Ok(result.map(|row| (UserId::from(row.user_id), row.expires_at)))
  }

  async fn delete(&self, session_id: SessionId) -> Result<bool, UserError> {
    let rows_affected = sqlx::query("DELETE FROM user_session WHERE id = $1")
      .bind(session_id)
      .execute(&self.pool)
      .await?
      .rows_affected();

    Ok(rows_affected > 0)
  }

  async fn delete_expired(&self) -> Result<u64, UserError> {
    let rows_affected = sqlx::query("DELETE FROM user_session WHERE expires_at < NOW()")
      .execute(&self.pool)
      .await?
      .rows_affected();

    Ok(rows_affected)
  }

  async fn delete_by_user_id(&self, user_id: UserId) -> Result<u64, UserError> {
    let rows_affected = sqlx::query("DELETE FROM user_session WHERE user_id = $1")
      .bind(user_id)
      .execute(&self.pool)
      .await?
      .rows_affected();

    Ok(rows_affected)
  }

  async fn update_expiration_by_session_ids(
    &self, session_ids: &[SessionId], expires_at: chrono::DateTime<chrono::Utc>,
  ) -> Result<u64, UserError> {
    // Nothing to do if no sessions are provided
    if session_ids.is_empty() {
      return Ok(0);
    }

    // Convert SessionId newtypes to raw Uuid values for use with ANY($2)
    let ids: Vec<Uuid> = session_ids.iter().copied().map(Into::into).collect();

    let rows_affected = sqlx::query(
      "UPDATE user_session \
       SET expires_at = LEAST(expires_at, $1) \
       WHERE id = ANY($2) AND expires_at > NOW()",
    )
    .bind(expires_at)
    .bind(&ids)
    .execute(&self.pool)
    .await?
    .rows_affected();

    Ok(rows_affected)
  }

  async fn list_all(&self) -> Result<Vec<SessionInfo>, UserError> {
    #[derive(sqlx::FromRow)]
    struct SessionRow {
      id: Uuid,
      user_id: Uuid,
      created_at: chrono::DateTime<chrono::Utc>,
      expires_at: chrono::DateTime<chrono::Utc>,
      is_expired: bool,
      status: String,
    }

    let rows: Vec<SessionRow> = sqlx::query_as(
      r#"
SELECT 
    id,
    user_id,
    created_at,
    expires_at,
    CASE WHEN expires_at < NOW() THEN true ELSE false END as is_expired,
    CASE 
        WHEN expires_at < NOW() THEN 'expired'
        ELSE 'active'
    END as status
FROM user_session
ORDER BY created_at DESC
"#,
    )
    .fetch_all(&self.pool)
    .await?;

    use crate::domain::SessionStatus;
    Ok(
      rows
        .into_iter()
        .map(|row| {
          let status = match row.status.as_str() {
            "active" => SessionStatus::Active,
            "expired" => SessionStatus::Expired,
            _ => {
              if row.is_expired {
                SessionStatus::Expired
              } else {
                SessionStatus::Active
              }
            },
          };
          SessionInfo {
            session_id: SessionId::from(row.id),
            user_id: UserId::from(row.user_id),
            created_at: row.created_at,
            expires_at: row.expires_at,
            is_expired: row.is_expired,
            status,
          }
        })
        .collect(),
    )
  }

  async fn list_active(&self) -> Result<Vec<SessionInfo>, UserError> {
    #[derive(sqlx::FromRow)]
    struct SessionRow {
      id: Uuid,
      user_id: Uuid,
      created_at: chrono::DateTime<chrono::Utc>,
      expires_at: chrono::DateTime<chrono::Utc>,
      is_expired: bool,
      status: String,
    }

    let rows: Vec<SessionRow> = sqlx::query_as(
      r#"
SELECT 
    id,
    user_id,
    created_at,
    expires_at,
    CASE WHEN expires_at < NOW() THEN true ELSE false END as is_expired,
    CASE 
        WHEN expires_at < NOW() THEN 'expired'
        ELSE 'active'
    END as status
FROM user_session
WHERE expires_at >= NOW()
ORDER BY created_at DESC
"#,
    )
    .fetch_all(&self.pool)
    .await?;

    use crate::domain::SessionStatus;
    Ok(
      rows
        .into_iter()
        .map(|row| {
          let status = match row.status.as_str() {
            "active" => SessionStatus::Active,
            "expired" => SessionStatus::Expired,
            _ => {
              if row.is_expired {
                SessionStatus::Expired
              } else {
                SessionStatus::Active
              }
            },
          };
          SessionInfo {
            session_id: SessionId::from(row.id),
            user_id: UserId::from(row.user_id),
            created_at: row.created_at,
            expires_at: row.expires_at,
            is_expired: row.is_expired,
            status,
          }
        })
        .collect(),
    )
  }

  async fn list_active_by_user_id(&self, user_id: UserId) -> Result<Vec<SessionInfo>, UserError> {
    #[derive(sqlx::FromRow)]
    struct SessionRow {
      id: Uuid,
      user_id: Uuid,
      created_at: chrono::DateTime<chrono::Utc>,
      expires_at: chrono::DateTime<chrono::Utc>,
      is_expired: bool,
      status: String,
    }

    let rows: Vec<SessionRow> = sqlx::query_as(
      r#"
SELECT 
    id,
    user_id,
    created_at,
    expires_at,
    CASE WHEN expires_at < NOW() THEN true ELSE false END as is_expired,
    CASE 
        WHEN expires_at < NOW() THEN 'expired'
        ELSE 'active'
    END as status
FROM user_session
WHERE expires_at >= NOW() AND user_id = $1
ORDER BY created_at DESC
"#,
    )
    .bind(user_id)
    .fetch_all(&self.pool)
    .await?;

    use crate::domain::SessionStatus;
    Ok(
      rows
        .into_iter()
        .map(|row| {
          let status = match row.status.as_str() {
            "active" => SessionStatus::Active,
            "expired" => SessionStatus::Expired,
            _ => {
              if row.is_expired {
                SessionStatus::Expired
              } else {
                SessionStatus::Active
              }
            },
          };
          SessionInfo {
            session_id: SessionId::from(row.id),
            user_id: UserId::from(row.user_id),
            created_at: row.created_at,
            expires_at: row.expires_at,
            is_expired: row.is_expired,
            status,
          }
        })
        .collect(),
    )
  }

  async fn get_by_session_id(&self, session_id: SessionId) -> Result<Option<SessionInfo>, UserError> {
    #[derive(sqlx::FromRow)]
    struct SessionRow {
      id: Uuid,
      user_id: Uuid,
      created_at: chrono::DateTime<chrono::Utc>,
      expires_at: chrono::DateTime<chrono::Utc>,
      is_expired: bool,
      status: String,
    }

    let row: Option<SessionRow> = sqlx::query_as(
      r#"
SELECT 
    id,
    user_id,
    created_at,
    expires_at,
    CASE WHEN expires_at < NOW() THEN true ELSE false END as is_expired,
    CASE 
        WHEN expires_at < NOW() THEN 'expired'
        ELSE 'active'
    END as status
FROM user_session
WHERE id = $1
"#,
    )
    .bind(session_id)
    .fetch_optional(&self.pool)
    .await?;

    use crate::domain::SessionStatus;
    Ok(row.map(|row| {
      let status = match row.status.as_str() {
        "active" => SessionStatus::Active,
        "expired" => SessionStatus::Expired,
        _ => {
          if row.is_expired {
            SessionStatus::Expired
          } else {
            SessionStatus::Active
          }
        },
      };
      SessionInfo {
        session_id: SessionId::from(row.id),
        user_id: UserId::from(row.user_id),
        created_at: row.created_at,
        expires_at: row.expires_at,
        is_expired: row.is_expired,
        status,
      }
    }))
  }
}
