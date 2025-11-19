use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::valueobjects::{AccountId, SessionId};
use crate::domain::{SessionInfo, SessionRepository};
use crate::error::AccountError;

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
    &self, session_id: SessionId, account_id: AccountId, expires_at: chrono::DateTime<chrono::Utc>,
  ) -> Result<(), AccountError> {
    sqlx::query("INSERT INTO account_session (session_id, account_id, expires_at) VALUES ($1, $2, $3)")
      .bind(session_id)
      .bind(account_id)
      .bind(expires_at)
      .execute(&self.pool)
      .await?;

    Ok(())
  }

  async fn find_by_session_id(
    &self, session_id: SessionId,
  ) -> Result<Option<(AccountId, chrono::DateTime<chrono::Utc>)>, AccountError> {
    #[derive(sqlx::FromRow)]
    struct SessionRow {
      account_id: Uuid,
      expires_at: chrono::DateTime<chrono::Utc>,
    }

    let result: Option<SessionRow> =
      sqlx::query_as("SELECT account_id, expires_at FROM account_session WHERE session_id = $1")
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?;

    Ok(result.map(|row| (AccountId::from(row.account_id), row.expires_at)))
  }

  async fn delete(&self, session_id: SessionId) -> Result<bool, AccountError> {
    let rows_affected = sqlx::query("DELETE FROM account_session WHERE session_id = $1")
      .bind(session_id)
      .execute(&self.pool)
      .await?
      .rows_affected();

    Ok(rows_affected > 0)
  }

  async fn delete_expired(&self) -> Result<u64, AccountError> {
    let rows_affected = sqlx::query("DELETE FROM account_session WHERE expires_at < NOW()")
      .execute(&self.pool)
      .await?
      .rows_affected();

    Ok(rows_affected)
  }

  async fn delete_by_account_id(&self, account_id: AccountId) -> Result<u64, AccountError> {
    let rows_affected = sqlx::query("DELETE FROM account_session WHERE account_id = $1")
      .bind(account_id)
      .execute(&self.pool)
      .await?
      .rows_affected();

    Ok(rows_affected)
  }

  async fn update_expiration_by_account_id(
    &self, account_id: AccountId, expires_at: chrono::DateTime<chrono::Utc>,
  ) -> Result<u64, AccountError> {
    let rows_affected = sqlx::query(
      "UPDATE account_session SET expires_at = LEAST(expires_at, $1) WHERE account_id = $2 AND expires_at > NOW()",
    )
    .bind(expires_at)
    .bind(account_id)
    .execute(&self.pool)
    .await?
    .rows_affected();

    Ok(rows_affected)
  }

  async fn list_all(&self) -> Result<Vec<SessionInfo>, AccountError> {
    #[derive(sqlx::FromRow)]
    struct SessionRow {
      session_id: Uuid,
      account_id: Uuid,
      created_at: chrono::DateTime<chrono::Utc>,
      expires_at: chrono::DateTime<chrono::Utc>,
      is_expired: bool,
      status: String,
    }

    let rows: Vec<SessionRow> = sqlx::query_as(
      r#"
SELECT 
    session_id,
    account_id,
    created_at,
    expires_at,
    CASE WHEN expires_at < NOW() THEN true ELSE false END as is_expired,
    CASE 
        WHEN expires_at < NOW() THEN 'expired'
        WHEN expires_at < NOW() + INTERVAL '1 minute' THEN 'expiring'
        ELSE 'active'
    END as status
FROM account_session
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
            "expiring" => SessionStatus::Expiring,
            "expired" => SessionStatus::Expired,
            _ => {
              // Fallback: calculate status based on is_expired and time remaining
              if row.is_expired {
                SessionStatus::Expired
              } else {
                let now = chrono::Utc::now();
                let time_until_expiry = row.expires_at - now;
                if time_until_expiry < chrono::Duration::minutes(1) {
                  SessionStatus::Expiring
                } else {
                  SessionStatus::Active
                }
              }
            },
          };
          SessionInfo {
            session_id: SessionId::from(row.session_id),
            account_id: AccountId::from(row.account_id),
            created_at: row.created_at,
            expires_at: row.expires_at,
            is_expired: row.is_expired,
            status,
          }
        })
        .collect(),
    )
  }

  async fn list_non_expired(&self) -> Result<Vec<SessionInfo>, AccountError> {
    #[derive(sqlx::FromRow)]
    struct SessionRow {
      session_id: Uuid,
      account_id: Uuid,
      created_at: chrono::DateTime<chrono::Utc>,
      expires_at: chrono::DateTime<chrono::Utc>,
      is_expired: bool,
      status: String,
    }

    let rows: Vec<SessionRow> = sqlx::query_as(
      r#"
SELECT 
    session_id,
    account_id,
    created_at,
    expires_at,
    CASE WHEN expires_at < NOW() THEN true ELSE false END as is_expired,
    CASE 
        WHEN expires_at < NOW() THEN 'expired'
        WHEN expires_at < NOW() + INTERVAL '1 minute' THEN 'expiring'
        ELSE 'active'
    END as status
FROM account_session
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
            "expiring" => SessionStatus::Expiring,
            "expired" => SessionStatus::Expired,
            _ => {
              // Fallback: calculate status based on is_expired and time remaining
              if row.is_expired {
                SessionStatus::Expired
              } else {
                let now = chrono::Utc::now();
                let time_until_expiry = row.expires_at - now;
                if time_until_expiry < chrono::Duration::minutes(1) {
                  SessionStatus::Expiring
                } else {
                  SessionStatus::Active
                }
              }
            },
          };
          SessionInfo {
            session_id: SessionId::from(row.session_id),
            account_id: AccountId::from(row.account_id),
            created_at: row.created_at,
            expires_at: row.expires_at,
            is_expired: row.is_expired,
            status,
          }
        })
        .collect(),
    )
  }

  async fn get_by_session_id(&self, session_id: SessionId) -> Result<Option<SessionInfo>, AccountError> {
    #[derive(sqlx::FromRow)]
    struct SessionRow {
      session_id: Uuid,
      account_id: Uuid,
      created_at: chrono::DateTime<chrono::Utc>,
      expires_at: chrono::DateTime<chrono::Utc>,
      is_expired: bool,
      status: String,
    }

    let row: Option<SessionRow> = sqlx::query_as(
      r#"
SELECT 
    session_id,
    account_id,
    created_at,
    expires_at,
    CASE WHEN expires_at < NOW() THEN true ELSE false END as is_expired,
    CASE 
        WHEN expires_at < NOW() THEN 'expired'
        WHEN expires_at < NOW() + INTERVAL '1 minute' THEN 'expiring'
        ELSE 'active'
    END as status
FROM account_session
WHERE session_id = $1
"#,
    )
    .bind(session_id)
    .fetch_optional(&self.pool)
    .await?;

    use crate::domain::SessionStatus;
    Ok(row.map(|row| {
      let status = match row.status.as_str() {
        "active" => SessionStatus::Active,
        "expiring" => SessionStatus::Expiring,
        "expired" => SessionStatus::Expired,
        _ => {
          // Fallback: calculate status based on is_expired and time remaining
          if row.is_expired {
            SessionStatus::Expired
          } else {
            let now = chrono::Utc::now();
            let time_until_expiry = row.expires_at - now;
            if time_until_expiry < chrono::Duration::minutes(1) {
              SessionStatus::Expiring
            } else {
              SessionStatus::Active
            }
          }
        },
      };
      SessionInfo {
        session_id: SessionId::from(row.session_id),
        account_id: AccountId::from(row.account_id),
        created_at: row.created_at,
        expires_at: row.expires_at,
        is_expired: row.is_expired,
        status,
      }
    }))
  }
}
