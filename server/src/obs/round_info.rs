use serde::{Deserialize, Serialize};
use valuable::Valuable;

use crate::domain::{OptionRoleOffsetPair, PlayerOffset, PlayerOffsetSet, Role, RoleSet};

#[derive(Clone, Valuable, Serialize, Deserialize, Debug)]
pub struct RoundInfo {
  round: u32,
  crown: PlayerOffset,
  roles_public_dropped: RoleSet,
  players_choose_role_before: PlayerOffsetSet,
  players_choose_role_after: PlayerOffsetSet,
  roles_chosen_before: RoleSet,
  roles_chosen_after: Option<RoleSet>,
  killed: OptionRoleOffsetPair,
  stolen: OptionRoleOffsetPair,
}

impl RoundInfo {
  pub fn new(round: u32, crown: PlayerOffset) -> Self {
    Self {
      round,
      crown,
      roles_public_dropped: RoleSet::empty(),
      players_choose_role_before: PlayerOffsetSet::empty(),
      players_choose_role_after: PlayerOffsetSet::empty(),
      roles_chosen_before: RoleSet::empty(),
      roles_chosen_after: None,
      killed: OptionRoleOffsetPair::none(),
      stolen: OptionRoleOffsetPair::none(),
    }
  }

  pub fn round(&self) -> u32 {
    self.round
  }

  pub fn set_crown(&mut self, crown: PlayerOffset) {
    self.crown = crown;
  }

  pub fn set_killed_offset(&mut self, offset: PlayerOffset) {
    self.killed.set_offset(offset);
  }

  pub fn set_stolen_offset(&mut self, offset: PlayerOffset) {
    self.stolen.set_offset(offset);
  }

  pub fn set_round(&mut self, round: u32) {
    self.round = round;
  }

  pub fn set_roles_public_dropped(&mut self, roles: RoleSet) {
    self.roles_public_dropped = roles;
  }

  pub fn set_players_choose_role_before(&mut self, offsets: PlayerOffsetSet) {
    self.players_choose_role_before = offsets;
  }

  pub fn set_players_choose_role_after(&mut self, offsets: PlayerOffsetSet) {
    self.players_choose_role_after = offsets;
  }

  pub fn set_roles_chosen_before(&mut self, roles: RoleSet) {
    self.roles_chosen_before = roles;
  }

  pub fn set_roles_chosen_after(&mut self, roles: RoleSet) {
    self.roles_chosen_after = Some(roles);
  }

  pub fn set_killed_role(&mut self, role: Role) {
    self.killed.set_role(role);
  }

  pub fn set_stolen_role(&mut self, role: Role) {
    self.stolen.set_role(role);
  }

  pub fn reset(&mut self) {
    self.roles_public_dropped = RoleSet::empty();
    self.players_choose_role_before = PlayerOffsetSet::empty();
    self.players_choose_role_after = PlayerOffsetSet::empty();
    self.roles_chosen_before = RoleSet::empty();
    self.roles_chosen_after = None;
    self.killed = OptionRoleOffsetPair::none();
    self.stolen = OptionRoleOffsetPair::none();
  }
}
