use std::env;
use std::path::PathBuf;

use serde::Deserialize;

use crate::errors::{ConfigError, UserError};

#[derive(Debug, Clone)]
pub struct UserConfig {
  pub dsn: String,
  pub password_salt: String,
  pub server_addr: String,
  pub max_connections: u32,
  pub min_connections: u32,
  pub acquire_timeout_seconds: u64,
  pub idle_timeout_seconds: u64,
  pub max_lifetime_seconds: u64,
  pub session_duration_hours: u64,
}

impl UserConfig {
  pub fn load() -> Result<Self, UserError> {
    let config_path = find_config_file().ok_or_else(|| {
      UserError::Config(ConfigError::FileNotFound(
        "user_context/config/default_user_context.toml not found".to_string(),
      ))
    })?;

    let contents = std::fs::read_to_string(&config_path).map_err(|e| {
      UserError::Config(ConfigError::FileNotFound(format!(
        "Failed to read config file {}: {}",
        config_path.display(),
        e
      )))
    })?;

    #[derive(Deserialize)]
    struct UserSection {
      dsn: String,
      password_salt: String,
      server_addr: String,
      max_connections: u32,
      min_connections: u32,
      #[serde(default = "default_acquire_timeout")]
      acquire_timeout_seconds: u64,
      #[serde(default = "default_idle_timeout")]
      idle_timeout_seconds: u64,
      #[serde(default = "default_max_lifetime")]
      max_lifetime_seconds: u64,
      #[serde(default = "default_session_duration_hours")]
      session_duration_hours: u64,
    }

    fn default_acquire_timeout() -> u64 {
      30
    }

    fn default_idle_timeout() -> u64 {
      600
    }

    fn default_max_lifetime() -> u64 {
      1800
    }

    fn default_session_duration_hours() -> u64 {
      24
    }

    #[derive(Deserialize)]
    struct ConfigFile {
      user: UserSection,
    }

    let config: ConfigFile = toml::from_str(&contents).map_err(|e| UserError::Config(ConfigError::ParseError(e)))?;

    let section = config.user;

    Ok(UserConfig {
      dsn: section.dsn,
      password_salt: section.password_salt,
      server_addr: section.server_addr,
      max_connections: section.max_connections,
      min_connections: section.min_connections,
      acquire_timeout_seconds: section.acquire_timeout_seconds,
      idle_timeout_seconds: section.idle_timeout_seconds,
      max_lifetime_seconds: section.max_lifetime_seconds,
      session_duration_hours: section.session_duration_hours,
    })
  }
}

fn find_config_file() -> Option<PathBuf> {
  // Try to find config/default_user_context.toml in the user_context directory
  if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
    let manifest_path = PathBuf::from(manifest_dir);
    let config_path = manifest_path.join("config").join("default_user_context.toml");
    if config_path.exists() {
      return Some(config_path);
    }
  }

  // Walk up from current directory looking for user_context/config/default_user_context.toml
  let mut current = env::current_dir().ok()?;

  for _ in 0..10 {
    let config_path = current
      .join("user_context")
      .join("config")
      .join("default_user_context.toml");
    if config_path.exists() {
      return Some(config_path);
    }

    if let Some(parent) = current.parent() {
      current = parent.to_path_buf();
    } else {
      break;
    }
  }

  None
}
