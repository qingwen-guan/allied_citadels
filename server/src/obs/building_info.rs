use serde::{Deserialize, Serialize};
use valuable::Valuable;

use crate::domain::{Card, Color};

#[derive(Debug, Clone, Valuable, Serialize, Deserialize)]
pub struct BuildingInfo {
  card: Card,
  color: Color,
  fee: u32,
  score: u32,
  destroy_fee: Option<u32>,
}

impl BuildingInfo {
  pub fn new(card: Card, destroy_fee: Option<u32>) -> Self {
    Self {
      card,
      color: card.color(),
      fee: card.fee(),
      score: card.score(),
      destroy_fee,
    }
  }
}
