use serde::{Deserialize, Serialize};
use valuable::Valuable;

use crate::domain::{OptionOffset, OptionRole, PlayerOffset, Role};

#[derive(Serialize, Deserialize, Clone, Debug, Valuable)]
pub struct OptionRoleOffsetPair {
  offset: OptionOffset,
  role: OptionRole,
}

impl OptionRoleOffsetPair {
  pub fn none() -> Self {
    Self {
      offset: OptionOffset::none(),
      role: OptionRole::None,
    }
  }

  pub fn set_offset(&mut self, offset: PlayerOffset) {
    self.offset = OptionOffset::from(offset);
  }

  pub fn set_role(&mut self, role: Role) {
    self.role = OptionRole::from(role);
  }
}
