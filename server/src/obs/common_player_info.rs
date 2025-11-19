use serde::{Deserialize, Serialize};
use valuable::Valuable;

use super::building_extra_score::BuildingExtraScore;
use super::building_info::BuildingInfo;
use crate::domain::{Camp, Role};
use crate::player::Player;

#[derive(Debug, Clone, Valuable, Serialize, Deserialize)]
pub struct CommonPlayerInfo {
  camp: Camp,
  gold: u32,
  buildings: Vec<BuildingInfo>,
  building_extra_score: BuildingExtraScore,
  role: Option<Role>,
}

impl From<&Player> for CommonPlayerInfo {
  fn from(player: &Player) -> Self {
    // TODO: 和 update 重复了?
    let mut buildings = Vec::new();
    for b in player.iter_buildings() {
      buildings.push(BuildingInfo::new(b, player.building_destroy_fee(b)));
    }

    let building_extra_score = BuildingExtraScore::new(player);

    Self {
      camp: player.camp(),
      gold: player.gold(),
      buildings,
      building_extra_score,
      role: None,
    }
  }
}

impl CommonPlayerInfo {
  pub fn set_role(&mut self, role: Role) {
    assert!(self.role.is_none());
    self.role = Some(role);
  }

  pub fn unset_role(&mut self) {
    assert!(self.role.is_some());
    self.role = None;
  }

  pub fn update_info(&mut self, player: &Player) {
    self.gold = player.gold();

    let mut buildings = Vec::new();
    for b in player.iter_buildings() {
      buildings.push(BuildingInfo::new(b, player.building_destroy_fee(b)));
    }
    self.buildings = buildings;
    self.building_extra_score = BuildingExtraScore::new(player);
  }

  pub fn camp(&self) -> Camp {
    self.camp
  }
}
