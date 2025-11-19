use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct RawConfig {
  history_uuid: String,
  ws_agent_uuid: String,
  host: String,
  port: u16,
}

impl RawConfig {
  pub fn load(path: &str) -> anyhow::Result<Self> {
    let file_content = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&file_content)?)
  }
}

pub struct Config {
  pub history_uuid: Uuid,
  pub ws_agent_uuid: Uuid,
  pub host: String,
  pub port: u16,
}

impl Config {
  pub fn load(path: &str) -> anyhow::Result<Self> {
    let raw_config = RawConfig::load(path)?;
    Ok(Self {
      history_uuid: Uuid::parse_str(&raw_config.history_uuid)?,
      ws_agent_uuid: Uuid::parse_str(&raw_config.ws_agent_uuid)?,
      host: raw_config.host,
      port: raw_config.port,
    })
  }
}
