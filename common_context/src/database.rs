use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::domain::valueobjects::DbConfig;

/// Create a PostgreSQL connection pool from database configuration
pub async fn create_db_pool(config: &DbConfig) -> Result<PgPool, sqlx::Error> {
  let pool = PgPoolOptions::new()
    .max_connections(config.max_connections)
    .min_connections(config.min_connections)
    .acquire_timeout(std::time::Duration::from_secs(config.acquire_timeout_seconds))
    .idle_timeout(Some(std::time::Duration::from_secs(config.idle_timeout_seconds)))
    .max_lifetime(Some(std::time::Duration::from_secs(config.max_lifetime_seconds)))
    .connect(&config.dsn)
    .await?;

  // Test the connection
  sqlx::query("SELECT 1").execute(&pool).await?;

  Ok(pool)
}
