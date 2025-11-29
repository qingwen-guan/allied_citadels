use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
  #[error("Configuration file not found: {0}")]
  FileNotFound(String),

  #[error("Could not parse config file: {0}")]
  ParseError(#[from] toml::de::Error),

  #[error("Missing required configuration: {0}")]
  MissingConfig(String),
}
