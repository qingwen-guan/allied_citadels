use thiserror::Error;

use super::config_error::ConfigError;

#[derive(Error, Debug)]
pub enum RoomError {
  #[error("Configuration error: {0}")]
  Config(#[from] ConfigError),

  #[error("Database error: {0}")]
  Database(#[from] sqlx::Error),

  #[error("Room not found")]
  NotFound,

  #[error("Room name already exists")]
  RoomNameExists,

  #[error("Invalid operation: {0}")]
  InvalidOperation(String),

  #[error("Data integrity error: multiple rooms found with the same name")]
  DuplicateRoomName,

  #[error("Invalid max players: must be 4 or 6")]
  InvalidMaxPlayers,
}
