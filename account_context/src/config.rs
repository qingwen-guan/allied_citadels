use std::env;
use std::path::PathBuf;

use serde::Deserialize;

use crate::error::{AccountError, ConfigError};

#[derive(Debug, Clone)]
pub struct Config {
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

impl Config {
  pub fn load() -> Result<Self, AccountError> {
    let config_path = find_config_file().ok_or_else(|| {
      AccountError::Config(ConfigError::FileNotFound(
        "config/account_context.toml not found in workspace root".to_string(),
      ))
    })?;

    let contents = std::fs::read_to_string(&config_path).map_err(|e| {
      AccountError::Config(ConfigError::FileNotFound(format!(
        "Failed to read config file {}: {}",
        config_path.display(),
        e
      )))
    })?;

    #[derive(Deserialize)]
    struct AccountSection {
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
      account: AccountSection,
    }

    let config: ConfigFile =
      toml::from_str(&contents).map_err(|e| AccountError::Config(ConfigError::ParseError(e)))?;

    Ok(Config {
      dsn: config.account.dsn,
      password_salt: config.account.password_salt,
      server_addr: config.account.server_addr,
      max_connections: config.account.max_connections,
      min_connections: config.account.min_connections,
      acquire_timeout_seconds: config.account.acquire_timeout_seconds,
      idle_timeout_seconds: config.account.idle_timeout_seconds,
      max_lifetime_seconds: config.account.max_lifetime_seconds,
      session_duration_hours: config.account.session_duration_hours,
    })
  }
}

fn find_config_file() -> Option<PathBuf> {
  // Try to find config/account_context.toml in workspace root
  if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
    let manifest_path = PathBuf::from(manifest_dir);
    // If we're in a workspace, go up to find the workspace root
    if let Some(parent) = manifest_path.parent() {
      let config_path = parent.join("config").join("account_context.toml");
      if config_path.exists() {
        return Some(config_path);
      }
    }
  }

  // Walk up from current directory looking for config/account_context.toml
  let mut current = env::current_dir().ok()?;

  for _ in 0..10 {
    let config_path = current.join("config").join("account_context.toml");
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
