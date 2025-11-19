use serde::{Deserialize, Serialize};
use valuable::Valuable;

use crate::domain::Card;

#[derive(Copy, Clone, Valuable, Serialize, Deserialize, Debug)]
pub enum Oper {
  EndRound,
  Card3Choose1,
  Card2Choose1,
  Card2Choose2,
  Gold(u32),
  Build(Card),
  SellCard(Card),
  BuyCard,
}
