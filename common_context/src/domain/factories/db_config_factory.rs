use std::env;
use std::path::PathBuf;

use serde::Deserialize;

use crate::constants::PACKAGE_DIR;
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
    struct ConfigFile {
      db: DbConfig,
    }

    let config: ConfigFile =
      toml::from_str(&contents).map_err(|e| MigrationError::Config(format!("Failed to parse config file: {}", e)))?;

    Ok(config.db)
  }

  /// Finds a config file in a specific context directory
  ///
  /// # Arguments
  /// * `context_dir` - The directory name (e.g., "session_context", "room_context", "common_context")
  /// * `config_file_name` - The name of the config file (e.g., "default_session_config.toml")
  ///
  /// # Returns
  /// `Some(PathBuf)` if the config file is found, `None` otherwise
  pub fn find_config_file(context_dir: &str, config_file_name: &str) -> Option<PathBuf> {
    // Try to find config file in the context directory using CARGO_MANIFEST_DIR
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
      let manifest_path = PathBuf::from(manifest_dir);
      let config_path = manifest_path.join("config").join(config_file_name);
      if config_path.exists() {
        return Some(config_path);
      }
    }

    // Walk up from current directory looking for context_dir/config/config_file_name
    let mut current = env::current_dir().ok()?;

    for _ in 0..10 {
      let config_path = current.join(context_dir).join("config").join(config_file_name);
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
    Self::new(
      Self::find_config_file(PACKAGE_DIR, DEFAULT_CONFIG_FILE_NAME).expect("Failed to find default config file"),
    )
  }
}
