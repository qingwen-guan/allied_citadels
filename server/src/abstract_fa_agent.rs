use async_trait::async_trait;

use crate::domain::{Card, DestroyTarget, MagicianSkill, Oper, Role, RoleSet};
use crate::obs::Obs;

#[async_trait]
pub trait AbstractFAAgent: Send + Sync {
  fn name(&self) -> &str;

  async fn wait_for_ready(&mut self);

  async fn choose_init_card(&mut self, obs: &Obs, c0: Card, c1: Card) -> Card;

  async fn choose_role(&mut self, obs: &Obs, roles: RoleSet) -> Role;

  async fn choose_kill_target(&mut self, obs: &Obs, choices: RoleSet) -> Role;

  async fn choose_steal_target(&mut self, obs: &Obs, choices: RoleSet) -> Role;

  async fn choose_swap_target(&mut self, obs: &Obs) -> MagicianSkill;

  async fn choose_destory_target(&mut self, obs: &Obs, choices: &[DestroyTarget]) -> Option<DestroyTarget>;

  async fn choose_tomb(&mut self, obs: &Obs, c: Card) -> bool;

  async fn choose_oper(&mut self, obs: &Obs, choices: &[Oper]) -> Oper;

  async fn choose_from_2(&mut self, obs: &Obs, c0: Card, c1: Card) -> Card;

  async fn choose_from_3(&mut self, obs: &Obs, c0: Card, c1: Card, c2: Card) -> Card;
}
