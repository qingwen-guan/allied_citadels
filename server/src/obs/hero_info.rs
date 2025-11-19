use serde::{Deserialize, Serialize};
use valuable::Valuable;

use super::common_player_info::CommonPlayerInfo;
use crate::domain::{Camp, Card, Role};
use crate::player::Player;

#[derive(Debug, Clone, Valuable, Serialize, Deserialize)]
pub struct HeroInfo {
  #[serde(flatten)]
  common: CommonPlayerInfo,

  cards: Vec<Card>,
}

impl From<&Player> for HeroInfo {
  fn from(player: &Player) -> Self {
    Self {
      common: CommonPlayerInfo::from(player),
      cards: player.cards().clone(),
    }
  }
}

impl HeroInfo {
  pub fn set_role(&mut self, role: Role) {
    self.common.set_role(role);
  }

  pub fn unset_role(&mut self) {
    self.common.unset_role();
  }

  pub fn update_info(&mut self, actor: &Player) {
    self.common.update_info(actor);
    self.cards = actor.cards().clone();
  }

  pub fn num_cards(&self) -> usize {
    self.cards.len()
  }

  pub fn card_at(&self, index: usize) -> Card {
    self.cards[index]
  }

  pub fn camp(&self) -> Camp {
    self.common.camp()
  }
}
