use std::env;
use std::path::PathBuf;

use serde::Deserialize;

use common_context::domain::valueobjects::DbConfig;

use crate::domain::valueobjects::RoomConfig;
use crate::errors::{ConfigError, RoomError};

const DEFAULT_CONFIG_FILE_NAME: &str = "default_room_config.toml";

/// Factory for creating RoomConfig instances from configuration files
pub struct RoomConfigFactory;

impl RoomConfigFactory {
  /// Creates a new instance of RoomConfigFactory
  pub fn new() -> Self {
    Self
  }

  /// Loads the room configuration from the default config file
  pub fn load(&self) -> Result<RoomConfig, RoomError> {
    let config_path = self.find_config_file().ok_or_else(|| {
      RoomError::Config(ConfigError::FileNotFound(format!(
        "room_context/config/{} not found",
        DEFAULT_CONFIG_FILE_NAME
      )))
    })?;

    let contents = std::fs::read_to_string(&config_path).map_err(|e| {
      RoomError::Config(ConfigError::FileNotFound(format!(
        "Failed to read config file {}: {}",
        config_path.display(),
        e
      )))
    })?;

    #[derive(Deserialize)]
    struct RoomSection {
      server_addr: String,
    }

    #[derive(Deserialize)]
    struct ConfigFile {
      room: RoomSection,
      db: DbConfig,
    }

    let config: ConfigFile = toml::from_str(&contents).map_err(|e| RoomError::Config(ConfigError::ParseError(e)))?;

    Ok(RoomConfig {
      db: config.db,
      server_addr: config.room.server_addr,
    })
  }

  fn find_config_file(&self) -> Option<PathBuf> {
    // Try to find config/default_room_config.toml in the room_context directory
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
      let manifest_path = PathBuf::from(manifest_dir);
      let config_path = manifest_path.join("config").join(DEFAULT_CONFIG_FILE_NAME);
      if config_path.exists() {
        return Some(config_path);
      }
    }

    // Walk up from current directory looking for room_context/config/default_room_config.toml
    let mut current = env::current_dir().ok()?;

    for _ in 0..10 {
      let config_path = current
        .join("room_context")
        .join("config")
        .join(DEFAULT_CONFIG_FILE_NAME);
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

impl Default for RoomConfigFactory {
  fn default() -> Self {
    Self::new()
  }
}
