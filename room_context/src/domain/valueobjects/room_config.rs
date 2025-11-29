use common_context::domain::valueobjects::DbConfig;

/// RoomConfig - value object for room service configuration
#[derive(Debug, Clone)]
pub struct RoomConfig {
  pub db: DbConfig,
  pub server_addr: String,
}
