use async_trait::async_trait;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::abstract_fa_agent::AbstractFAAgent;
use crate::bit;
use crate::domain::{Card, DestroyTarget, MagicianSkill, Oper, PlayerOffset, Role, RoleSet};
use crate::obs::Obs;

pub struct RandomFAAgent {
  rng: StdRng,
}

impl Default for RandomFAAgent {
  fn default() -> Self {
    Self::new()
  }
}

impl RandomFAAgent {
  pub fn new() -> Self {
    Self {
      rng: StdRng::seed_from_u64(rand::random()),
    }
  }
}

#[async_trait]
impl AbstractFAAgent for RandomFAAgent {
  fn name(&self) -> &str {
    "RandomAgent"
  }

  async fn wait_for_ready(&mut self) {
    // RandomAgent does not need to be ready
  }

  async fn choose_init_card(&mut self, _obs: &Obs, c0: Card, c1: Card) -> Card {
    match self.rng.random_range(0..2) {
      0 => c0,
      1 => c1,
      _ => panic!("Invalid random number"),
    }
  }

  async fn choose_role(&mut self, _obs: &Obs, roles: RoleSet) -> Role {
    roles.random_choose(&mut self.rng)
  }

  async fn choose_kill_target(&mut self, _obs: &Obs, choices: RoleSet) -> Role {
    choices.random_choose(&mut self.rng)
  }

  async fn choose_steal_target(&mut self, _obs: &Obs, choices: RoleSet) -> Role {
    choices.random_choose(&mut self.rng)
  }

  async fn choose_swap_target(&mut self, obs: &Obs) -> MagicianSkill {
    let num_choices = (1 << obs.actor_num_cards()) + (obs.num_players() - 1);
    let chosen_index = self.rng.random_range(0..num_choices);
    if chosen_index == 0 {
      MagicianSkill::放弃
    } else if chosen_index < (1 << obs.actor_num_cards()) {
      let mut cards = Vec::new();
      for i in 0..obs.actor_num_cards() {
        if bit::test_bit(chosen_index, i) {
          cards.push(obs.hero_card_at(i));
        }
      }
      MagicianSkill::制衡(cards)
    } else {
      let nth_choice = chosen_index - (1 << obs.actor_num_cards());
      return MagicianSkill::Swap(PlayerOffset::from_usize(nth_choice + 1));
    }
  }

  async fn choose_destory_target(&mut self, _obs: &Obs, choices: &[DestroyTarget]) -> Option<DestroyTarget> {
    let v = self.rng.random_range(0..=choices.len());
    if v == choices.len() {
      return None;
    }
    Some(choices[v])
  }

  async fn choose_tomb(&mut self, _obs: &Obs, _c: Card) -> bool {
    self.rng.random_range(0..2) == 0
  }

  async fn choose_oper(&mut self, _obs: &Obs, choices: &[Oper]) -> Oper {
    choices[self.rng.random_range(0..choices.len())]
  }

  async fn choose_from_2(&mut self, _obs: &Obs, c0: Card, c1: Card) -> Card {
    match self.rng.random_range(0..2) {
      0 => c0,
      1 => c1,
      _ => panic!("Invalid random number"),
    }
  }

  async fn choose_from_3(&mut self, _obs: &Obs, c0: Card, c1: Card, c2: Card) -> Card {
    match self.rng.random_range(0..3) {
      0 => c0,
      1 => c1,
      2 => c2,
      _ => panic!("Invalid random number"),
    }
  }
}
