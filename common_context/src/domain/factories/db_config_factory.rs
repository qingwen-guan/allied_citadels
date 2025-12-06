use std::env;
use std::path::PathBuf;

use serde::Deserialize;

use crate::domain::valueobjects::DbConfig;
use crate::migrations::MigrationError;

const DEFAULT_CONFIG_FILE_NAME: &str = "default_common_config.toml";

/// Factory for creating DbConfig instances from configuration files
pub struct DbConfigFactory {
  config_path: PathBuf,
}

impl DbConfigFactory {
  /// Creates a new instance of DbConfigFactory with the specified config file path
  pub fn new(config_path: PathBuf) -> Self {
    Self { config_path }
  }

  /// Loads the database configuration from the config file
  pub fn load(&self) -> Result<DbConfig, MigrationError> {
    let contents = std::fs::read_to_string(&self.config_path).map_err(|e| {
      MigrationError::Config(format!(
        "Failed to read config file {}: {}",
        self.config_path.display(),
        e
      ))
    })?;

    #[derive(Deserialize)]
    struct DbSection {
      dsn: String,
      max_connections: u32,
      min_connections: u32,
      #[serde(default = "default_acquire_timeout")]
      acquire_timeout_seconds: u64,
      #[serde(default = "default_idle_timeout")]
      idle_timeout_seconds: u64,
      #[serde(default = "default_max_lifetime")]
      max_lifetime_seconds: u64,
    }

    const fn default_acquire_timeout() -> u64 {
      30
    }

    const fn default_idle_timeout() -> u64 {
      600
    }

    const fn default_max_lifetime() -> u64 {
      1800
    }

    #[derive(Deserialize)]
    struct ConfigFile {
      db: DbSection,
    }

    let config: ConfigFile =
      toml::from_str(&contents).map_err(|e| MigrationError::Config(format!("Failed to parse config file: {}", e)))?;

    Ok(DbConfig {
      dsn: config.db.dsn,
      max_connections: config.db.max_connections,
      min_connections: config.db.min_connections,
      acquire_timeout_seconds: config.db.acquire_timeout_seconds,
      idle_timeout_seconds: config.db.idle_timeout_seconds,
      max_lifetime_seconds: config.db.max_lifetime_seconds,
    })
  }

  /// Finds the default config file path
  pub fn find_config_file(default_config_file_name: &str) -> Option<PathBuf> {
    // Try to find config/default_common_config.toml in the common_context directory
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
      let manifest_path = PathBuf::from(manifest_dir);
      let config_path = manifest_path.join("config").join(default_config_file_name);
      if config_path.exists() {
        return Some(config_path);
      }
    }

    // Walk up from current directory looking for common_context/config/default_common_config.toml
    let mut current = env::current_dir().ok()?;

    for _ in 0..10 {
      let config_path = current
        .join("common_context")
        .join("config")
        .join(default_config_file_name);
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
}

impl Default for DbConfigFactory {
  fn default() -> Self {
    Self::new(Self::find_config_file(DEFAULT_CONFIG_FILE_NAME).expect("Failed to find default config file"))
  }
}
