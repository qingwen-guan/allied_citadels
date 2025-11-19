use serde::{Deserialize, Serialize};
use valuable::Valuable;

use crate::domain::{Card, PlayerOffset};

#[derive(Clone, Valuable, Debug, Serialize, Deserialize)]
pub enum MagicianSkill {
  Swap(PlayerOffset),
  制衡(Vec<Card>), // TODO: Replace
  放弃,            // TODO: Noop
}
