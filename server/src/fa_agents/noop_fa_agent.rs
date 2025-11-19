use async_trait::async_trait;

use crate::abstract_fa_agent::AbstractFAAgent;
use crate::domain::{Card, DestroyTarget, MagicianSkill, Oper, Role, RoleSet};
use crate::obs::Obs;

pub struct NoopFAAgent {}

impl NoopFAAgent {
  pub fn new() -> Self {
    Self {}
  }
}

impl Default for NoopFAAgent {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl AbstractFAAgent for NoopFAAgent {
  fn name(&self) -> &str {
    "NoopAgent"
  }

  async fn wait_for_ready(&mut self) {
    // NoopAgent does not need to be ready
  }

  async fn choose_init_card(&mut self, _obs: &Obs, _c0: Card, _c1: Card) -> Card {
    panic!("NoopAgent should not be used for choosing init card");
  }

  async fn choose_role(&mut self, _obs: &Obs, _roles: RoleSet) -> Role {
    panic!("NoopAgent should not be used for choosing role");
  }

  async fn choose_kill_target(&mut self, _obs: &Obs, _choices: RoleSet) -> Role {
    panic!("NoopAgent should not be used for choosing assassination target");
  }

  async fn choose_steal_target(&mut self, _obs: &Obs, _choices: RoleSet) -> Role {
    panic!("NoopAgent should not be used for choosing thief target");
  }

  async fn choose_swap_target(&mut self, _obs: &Obs) -> MagicianSkill {
    panic!("NoopAgent should not be used for choosing magician skill");
  }

  async fn choose_destory_target(&mut self, _obs: &Obs, _choices: &[DestroyTarget]) -> Option<DestroyTarget> {
    panic!("NoopAgent should not be used for choosing destroy target");
  }

  async fn choose_tomb(&mut self, _obs: &Obs, _c: Card) -> bool {
    panic!("NoopAgent should not be used for choosing tomb");
  }

  async fn choose_oper(&mut self, _obs: &Obs, _choices: &[Oper]) -> Oper {
    panic!("NoopAgent should not be used for choosing operation");
  }

  async fn choose_from_2(&mut self, _obs: &Obs, _c0: Card, _c1: Card) -> Card {
    panic!("NoOpAgent should not be used for choosing from 2 cards");
  }

  async fn choose_from_3(&mut self, _obs: &Obs, _c0: Card, _c1: Card, _c2: Card) -> Card {
    panic!("NoOpAgent should not be used for choosing from 3 cards");
  }
}
