use common_context::domain::factories::DbConfigFactory;
use common_context::domain::valueobjects::DbConfig;
use serde::Deserialize;
use thiserror::Error;

const DEFAULT_CONFIG_FILE_NAME: &str = "default_session_config.toml";
const CONTEXT_DIR: &str = "session_context";

/// SessionConfig - value object for session service configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
  pub db: DbConfig,
  pub session_duration_hours: u64,
}

#[derive(Debug, Error)]
pub enum SessionConfigError {
  #[error("Config file not found: {0}")]
  FileNotFound(String),
  #[error("Failed to read config file: {0}")]
  ReadError(String),
  #[error("Failed to parse config file: {0}")]
  ParseError(toml::de::Error),
}

impl SessionConfig {
  pub fn load() -> Result<Self, SessionConfigError> {
    let config_path = DbConfigFactory::find_config_file(CONTEXT_DIR, DEFAULT_CONFIG_FILE_NAME).ok_or_else(|| {
      SessionConfigError::FileNotFound(format!("{}/config/{} not found", CONTEXT_DIR, DEFAULT_CONFIG_FILE_NAME))
    })?;

    let contents = std::fs::read_to_string(&config_path).map_err(|e| {
      SessionConfigError::ReadError(format!("Failed to read config file {}: {}", config_path.display(), e))
    })?;

    #[derive(Deserialize)]
    struct SessionSection {
      #[serde(default = "default_session_duration_hours")]
      session_duration_hours: u64,
    }

    const fn default_session_duration_hours() -> u64 {
      24
    }

    #[derive(Deserialize)]
    struct ConfigFile {
      session: SessionSection,
      db: DbConfig,
    }

    let config: ConfigFile = toml::from_str(&contents).map_err(|e| SessionConfigError::ParseError(e))?;

    Ok(SessionConfig {
      db: config.db,
      session_duration_hours: config.session.session_duration_hours,
    })
  }
}
