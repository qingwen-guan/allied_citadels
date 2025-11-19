use async_trait::async_trait;

use crate::domain::{PlayerOffset, Role};
use crate::obs::Obs;

#[async_trait]
pub trait AbstractFYIAgent: Send + Sync {
  async fn obs_changed(&mut self, obs: &Obs);
  async fn first_role_dropped(&mut self);
  async fn last_role_dropped(&mut self);
  async fn villain_choose_role_reqed(&mut self, villain: PlayerOffset, num_choices: usize);
  async fn villain_choose_role_resped(&mut self, villain: PlayerOffset, role: Role);
}
