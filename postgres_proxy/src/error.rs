use thiserror::Error;

#[derive(Error, Debug)]
pub enum PostgresError {
  #[error("Configuration error: {0}")]
  Config(#[from] ConfigError),

  #[error("Docker error: {0}")]
  Docker(#[from] DockerError),
  // #[error("PostgreSQL error: {0}")]
  // Postgres(String),
}

#[derive(Error, Debug)]
pub enum ConfigError {
  #[error("Configuration file not found: {0}\nPlease create config.toml in the project root.")]
  FileNotFound(String),

  #[error("Could not parse config file: {0}")]
  ParseError(#[from] toml::de::Error),

  #[error("Missing required configuration: {0}")]
  MissingConfig(String),
}

#[derive(Error, Debug)]
pub enum DockerError {
  #[error("Docker is not running or not installed.\nPlease start Docker Desktop and try again.")]
  NotRunning,

  #[error("Command failed: {0}")]
  CommandFailed(String),

  #[error("{0}")]
  Other(String),
}
