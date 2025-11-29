/// Database configuration
#[derive(Debug, Clone)]
pub struct DbConfig {
  pub dsn: String,
  pub max_connections: u32,
  pub min_connections: u32,
  pub acquire_timeout_seconds: u64,
  pub idle_timeout_seconds: u64,
  pub max_lifetime_seconds: u64,
}
