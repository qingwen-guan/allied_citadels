use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum ConfigError {
  #[error("Configuration file not found: {0}")]
  FileNotFound(String),

  #[error("Could not parse config file: {0}")]
  ParseError(#[from] toml::de::Error),

  #[error("Missing required configuration: {0}")]
  MissingConfig(String),
}
