use async_trait::async_trait;

use crate::abstract_fyi_agent::AbstractFYIAgent;
use crate::domain::{PlayerOffset, Role};
use crate::obs::Obs;

pub struct NoopFYIAgent {}

impl NoopFYIAgent {
  pub fn new() -> Self {
    Self {}
  }
}

impl Default for NoopFYIAgent {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl AbstractFYIAgent for NoopFYIAgent {
  async fn obs_changed(&mut self, _obs: &Obs) {}

  async fn first_role_dropped(&mut self) {}

  async fn last_role_dropped(&mut self) {}

  async fn villain_choose_role_reqed(&mut self, _villain: PlayerOffset, _num_choices: usize) {}

  async fn villain_choose_role_resped(&mut self, _villain: PlayerOffset, _role: Role) {}
}
