use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserError {
  #[error("Configuration error: {0}")]
  Config(#[from] ConfigError),

  #[error("Database error: {0}")]
  Database(#[from] sqlx::Error),

  #[error("User not found")]
  NotFound,

  #[error("Nickname already exists")]
  NicknameExists,

  #[error("Invalid operation: {0}")]
  InvalidOperation(String),

  #[error("Data integrity error: multiple users found with the same nickname")]
  DuplicateNickname,

  #[error("Invalid credentials")]
  InvalidCredentials,
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
