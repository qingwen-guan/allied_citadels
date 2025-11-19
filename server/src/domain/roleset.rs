use std::ops::{BitOr, BitOrAssign, Sub, SubAssign};

use rand::Rng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use valuable::Valuable;

use crate::bit;
use crate::domain::{OptionRole, Role};

#[derive(Clone, Copy, Valuable, Debug)]
pub struct RoleSet {
  value: isize,
}

impl BitOr<Role> for RoleSet {
  type Output = Self;

  fn bitor(self, r: Role) -> Self::Output {
    Self {
      value: self.value | (r as isize),
    }
  }
}

impl BitOrAssign<Role> for RoleSet {
  fn bitor_assign(&mut self, r: Role) {
    self.value |= r as isize;
  }
}

impl BitOr<OptionRole> for RoleSet {
  type Output = Self;

  fn bitor(self, r: OptionRole) -> Self::Output {
    Self {
      value: self.value | (r as isize),
    }
  }
}

impl BitOr for RoleSet {
  type Output = Self;

  fn bitor(self, r: Self) -> Self::Output {
    Self {
      value: self.value | r.value,
    }
  }
}

impl From<RoleSet> for Role {
  fn from(val: RoleSet) -> Self {
    Role::from(val.value)
  }
}

impl RoleSet {
  pub fn empty() -> Self {
    Self { value: 0 }
  }

  pub fn from_pair(r0: Role, r1: Role) -> Self {
    Self {
      value: (r0 as isize) | (r1 as isize),
    }
  }

  pub fn contains(self, role: Role) -> bool {
    (self.value & (role as isize)) != 0
  }

  pub fn len(&self) -> usize {
    bit::popcnt(self.value) as usize
  }

  pub fn is_empty(&self) -> bool {
    self.value == 0
  }

  pub fn universal() -> Self {
    Self { value: (1 << 8) - 1 }
  }

  pub fn random_choose(&self, rng: &mut StdRng) -> Role {
    let cnt = self.len();
    let index = rng.random_range(0..cnt);

    let mut s = self.value;
    for _ in 0..index {
      s = bit::clear_lowbit(s);
    }

    Role::from(bit::lowbit(s))
  }
}

impl Sub for RoleSet {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self::Output {
    Self::Output {
      value: (self.value & !(rhs.value)),
    }
  }
}

impl SubAssign for RoleSet {
  fn sub_assign(&mut self, rhs: Self) {
    self.value &= !(rhs.value)
  }
}

impl Sub<Role> for RoleSet {
  type Output = Self;

  fn sub(self, rhs: Role) -> Self::Output {
    Self::Output {
      value: self.value & !(rhs as isize),
    }
  }
}

impl SubAssign<Role> for RoleSet {
  fn sub_assign(&mut self, rhs: Role) {
    self.value &= !(rhs as isize);
  }
}

impl Serialize for RoleSet {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_i64(self.value as i64)
  }
}

impl<'de> Deserialize<'de> for RoleSet {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let value = isize::deserialize(deserializer)?;
    Ok(Self { value })
  }
}
