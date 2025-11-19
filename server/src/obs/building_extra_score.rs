use serde::{Deserialize, Serialize};
use valuable::Valuable;

use crate::player::Player;

#[derive(Debug, Clone, Valuable, Serialize, Deserialize)]
pub struct BuildingExtraScore {
  // TODO: rename to extra score
  all_colors: u32,
  eight_buildings: u32,
  first_eight_buildings: u32,
}

impl BuildingExtraScore {
  pub fn new(player: &Player) -> Self {
    Self {
      all_colors: if player.has_all_colors() { 3 } else { 0 },
      eight_buildings: if player.buildings_len() == 8 { 2 } else { 0 },
      first_eight_buildings: if player.is_first_8_buildings() { 2 } else { 0 },
    }
  }
}
