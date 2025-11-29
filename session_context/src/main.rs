mod cli;

use clap::Parser;
use common_context::database::create_db_pool;
use room_context::{Config as RoomConfig, PostgresMessageRepository, PostgresRoomRepository, RoomService};
use sqlx::postgres::PgPoolOptions;
use user_context::UserService;
use user_context::domain::valueobjects::Salt;
use user_context::infra::{PostgresSessionRepository, PostgresUserRepository};
use user_context::{Config, UserError, UserFactory};

async fn create_pool(config: &Config) -> Result<sqlx::PgPool, UserError> {
  let pool = PgPoolOptions::new()
    .max_connections(config.max_connections)
    .min_connections(config.min_connections)
    .acquire_timeout(std::time::Duration::from_secs(config.acquire_timeout_seconds))
    .idle_timeout(Some(std::time::Duration::from_secs(config.idle_timeout_seconds)))
    .max_lifetime(Some(std::time::Duration::from_secs(config.max_lifetime_seconds)))
    .connect(&config.dsn)
    .await?;

  // Test the connection
  sqlx::query("SELECT 1")
    .execute(&pool)
    .await
    .map_err(UserError::Database)?;

  Ok(pool)
}

async fn create_user_service(config: &Config) -> Result<UserService, UserError> {
  let pool = create_pool(config).await?;
  let repository = Box::new(PostgresUserRepository::new(pool.clone()));
  let session_repository = Box::new(PostgresSessionRepository::new(pool));
  let user_factory = UserFactory::new(Salt::from(config.password_salt.as_str()));
  Ok(UserService::new(config, repository, session_repository, user_factory))
}

async fn create_room_service(config: &RoomConfig) -> Result<RoomService, room_context::RoomError> {
  let pool = create_db_pool(&config.db)
    .await
    .map_err(room_context::RoomError::Database)?;
  let room_repository = Box::new(PostgresRoomRepository::new(pool.clone()));
  let user_repository = Box::new(PostgresUserRepository::new(pool.clone()));
  let message_repository = Box::new(PostgresMessageRepository::new(pool));
  Ok(RoomService::new(room_repository, user_repository, message_repository))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Initialize tracing subscriber
  tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();

  let cli = cli::Cli::parse();
  let config = Config::load()?;
  let user_service = create_user_service(&config).await?;

  // Load room config - room_context uses the same config file structure
  let room_config = RoomConfig::load()?;
  let room_service = create_room_service(&room_config).await?;

  cli::handle_command(cli.command, user_service, room_service).await
}
