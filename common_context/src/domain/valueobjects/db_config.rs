use serde::Deserialize;

/// Database configuration
#[derive(Debug, Clone, Deserialize)]
pub struct DbConfig {
  pub dsn: String,
  #[serde(default = "default_max_connections")]
  pub max_connections: u32,
  #[serde(default = "default_min_connections")]
  pub min_connections: u32,
  #[serde(default = "default_acquire_timeout")]
  pub acquire_timeout_seconds: u64,
  #[serde(default = "default_idle_timeout")]
  pub idle_timeout_seconds: u64,
  #[serde(default = "default_max_lifetime")]
  pub max_lifetime_seconds: u64,
}

const fn default_max_connections() -> u32 {
  10
}

const fn default_min_connections() -> u32 {
  2
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
