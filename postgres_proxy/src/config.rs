use std::env;
use std::path::PathBuf;

use crate::error::{ConfigError, PostgresError};

#[derive(Debug, Clone)]
pub struct PostgresConfig {
  pub user: String,
  pub password: String,
  pub host: String,
  pub port: String,
  pub version: String,
}

#[derive(Debug, Clone)]
pub struct DockerConfig {
  pub container_name: String,
  pub volume_name: String,
  pub timezone: String,
}

#[derive(Debug, Clone)]
pub struct TimingConfig {
  pub initial_wait_seconds: u64,
  pub container_start_wait_seconds: u64,
  pub max_postgres_retries: u32,
  pub retry_delay_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct Config {
  pub postgres: PostgresConfig,
  pub docker: DockerConfig,
  pub timing: TimingConfig,
}

impl Config {
  pub fn load() -> Result<Self, PostgresError> {
    let config_path = find_config_file()?;

    let contents = std::fs::read_to_string(&config_path)
      .map_err(|e| ConfigError::FileNotFound(format!("{}: {}", config_path.display(), e)))?;

    let toml_data: toml::Value = toml::from_str(&contents).map_err(ConfigError::ParseError)?;

    // Load PostgreSQL configuration
    let postgres_table = toml_data.get("postgres").and_then(|v| v.as_table()).ok_or_else(|| {
      ConfigError::MissingConfig("Missing [postgres] section in config/postgres_proxy.toml".to_string())
    })?;

    let postgres = PostgresConfig {
      user: get_required_string(postgres_table, "user", "postgres.user")?,
      password: get_required_string(postgres_table, "password", "postgres.password")?,
      host: get_required_string(postgres_table, "host", "postgres.host")?,
      port: get_required_string(postgres_table, "port", "postgres.port")?,
      version: get_required_string(postgres_table, "version", "postgres.version")?,
    };

    // Load Docker configuration
    let docker_table = toml_data.get("docker").and_then(|v| v.as_table()).ok_or_else(|| {
      ConfigError::MissingConfig("Missing [docker] section in config/postgres_proxy.toml".to_string())
    })?;

    let docker = DockerConfig {
      container_name: get_required_string(docker_table, "container_name", "docker.container_name")?,
      volume_name: get_required_string(docker_table, "volume_name", "docker.volume_name")?,
      timezone: get_optional_string(docker_table, "timezone").unwrap_or_else(|| "UTC".to_string()),
    };

    // Load timing configuration (required)
    let timing_table = toml_data.get("timing").and_then(|v| v.as_table()).ok_or_else(|| {
      ConfigError::MissingConfig("Missing [timing] section in config/postgres_proxy.toml".to_string())
    })?;

    let timing = TimingConfig {
      initial_wait_seconds: get_required_integer(timing_table, "initial_wait_seconds", "timing.initial_wait_seconds")?
        as u64,
      container_start_wait_seconds: get_required_integer(
        timing_table,
        "container_start_wait_seconds",
        "timing.container_start_wait_seconds",
      )? as u64,
      max_postgres_retries: get_required_integer(timing_table, "max_postgres_retries", "timing.max_postgres_retries")?
        as u32,
      retry_delay_seconds: get_required_integer(timing_table, "retry_delay_seconds", "timing.retry_delay_seconds")?
        as u64,
    };

    let config = Config {
      postgres,
      docker,
      timing,
    };

    Ok(config)
  }
}

fn get_required_string(
  table: &toml::map::Map<String, toml::Value>, key: &str, full_key: &str,
) -> Result<String, ConfigError> {
  let value = table.get(key).and_then(|v| v.as_str()).ok_or_else(|| {
    ConfigError::MissingConfig(format!(
      "Missing required key '{}' in config/postgres_proxy.toml",
      full_key
    ))
  })?;
  Ok(value.to_string())
}

fn get_optional_string(table: &toml::map::Map<String, toml::Value>, key: &str) -> Option<String> {
  table.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn get_required_integer(
  table: &toml::map::Map<String, toml::Value>, key: &str, full_key: &str,
) -> Result<i64, ConfigError> {
  let value = table.get(key).and_then(|v| v.as_integer()).ok_or_else(|| {
    ConfigError::MissingConfig(format!(
      "Missing required key '{}' in config/postgres_proxy.toml",
      full_key
    ))
  })?;
  Ok(value)
}

fn find_config_file() -> Result<PathBuf, ConfigError> {
  // Find workspace root by looking for Cargo.toml
  let workspace_root = find_workspace_root()?;

  // Look for config/postgres_proxy.toml
  let config_path = workspace_root.join("config").join("postgres_proxy.toml");
  if config_path.exists() {
    return Ok(config_path);
  }

  Err(ConfigError::FileNotFound(format!(
    "config/postgres_proxy.toml not found in workspace root: {}",
    workspace_root.display()
  )))
}

fn find_workspace_root() -> Result<PathBuf, ConfigError> {
  // Try CARGO_MANIFEST_DIR first (set by Cargo at compile time)
  if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
    let manifest_path = PathBuf::from(manifest_dir);
    // If we're in a workspace, go up to find the workspace root
    if let Some(parent) = manifest_path.parent() {
      // Check if parent has Cargo.toml (workspace root)
      let workspace_cargo = parent.join("Cargo.toml");
      if workspace_cargo.exists() {
        return Ok(parent.to_path_buf());
      }
      // Otherwise, the manifest dir itself is the root
      return Ok(manifest_path);
    }
  }

  // Walk up from current directory looking for Cargo.toml
  let mut current =
    env::current_dir().map_err(|e| ConfigError::FileNotFound(format!("Failed to get current directory: {}", e)))?;

  for _ in 0..10 {
    // Limit to 10 levels to avoid infinite loops
    let cargo_toml = current.join("Cargo.toml");
    if cargo_toml.exists() {
      return Ok(current);
    }

    // Also check for workspace Cargo.toml (might be in parent)
    if let Some(parent) = current.parent() {
      let workspace_cargo = parent.join("Cargo.toml");
      if workspace_cargo.exists() {
        // Check if it's a workspace
        if let Ok(contents) = std::fs::read_to_string(&workspace_cargo)
          && contents.contains("[workspace]")
        {
          return Ok(parent.to_path_buf());
        }
      }
      current = parent.to_path_buf();
    } else {
      break;
    }
  }

  Err(ConfigError::FileNotFound(
    "Could not find workspace root (Cargo.toml not found)".to_string(),
  ))
}
