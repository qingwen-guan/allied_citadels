use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use strum::IntoEnumIterator;

use crate::domain::Card;
use crate::history::History;

pub struct Deck {
  rng: StdRng,
  deck: Vec<Card>,

  drop: Vec<Card>,
}

impl Deck {
  pub async fn new(mut rng: StdRng, history: &mut History) -> Self {
    let mut deck = Vec::new();
    for c in Card::iter() {
      for _ in 0..c.number() {
        deck.push(c);
      }
    }
    deck.shuffle(&mut rng);
    history.shuffle_deck(&deck).await;

    Self {
      rng,
      deck,
      drop: Vec::new(),
    }
  }

  pub async fn take(&mut self, history: &mut History) -> Option<Card> {
    if self.deck.is_empty() {
      std::mem::swap(&mut self.deck, &mut self.drop);
      self.deck.shuffle(&mut self.rng);
      history.shuffle_deck(&self.deck).await;
    }
    self.deck.pop()
  }

  pub fn drop(&mut self, c: Card) {
    self.drop.push(c);
  }

  pub fn peek_deck(&self) -> &[Card] {
    &self.deck
  }

  pub fn peek_drop(&self) -> &[Card] {
    &self.drop
  }

  pub fn rng(&mut self) -> &mut StdRng {
    &mut self.rng
  }
}
