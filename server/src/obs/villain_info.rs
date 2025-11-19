use serde::{Deserialize, Serialize};
use valuable::Valuable;

use super::common_player_info::CommonPlayerInfo;
use crate::domain::Role;
use crate::player::Player;

#[derive(Debug, Clone, Valuable, Serialize, Deserialize)]
pub struct VillainInfo {
  #[serde(flatten)]
  common: CommonPlayerInfo,

  num_cards: u32,
}

impl From<&Player> for VillainInfo {
  fn from(player: &Player) -> Self {
    Self {
      common: CommonPlayerInfo::from(player),
      num_cards: player.cards().len() as u32,
    }
  }
}
impl VillainInfo {
  pub fn set_role(&mut self, role: Role) {
    self.common.set_role(role);
  }

  pub fn unset_role(&mut self) {
    self.common.unset_role();
  }

  pub fn update_info(&mut self, player: &Player) {
    self.common.update_info(player);
    self.num_cards = player.cards().len() as u32;
  }
}
