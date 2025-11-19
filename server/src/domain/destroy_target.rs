use serde::{Deserialize, Serialize};
use valuable::Valuable;

use crate::domain::{Card, PlayerOffset};

#[derive(Copy, Clone, Valuable, Serialize, Deserialize, Debug)]
pub struct DestroyTarget {
  pub player_offset: PlayerOffset,
  pub card: Card,
}
