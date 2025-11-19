use std::ops::{BitOr, BitOrAssign};

use serde::{Deserialize, Serialize};
use valuable::Valuable;

use crate::domain::PlayerOffset;

#[derive(Clone, Debug, Valuable)]
pub struct PlayerOffsetSet {
  value: u32,
}

impl BitOr<PlayerOffset> for PlayerOffsetSet {
  type Output = Self;

  fn bitor(self, rhs: PlayerOffset) -> Self::Output {
    Self {
      value: self.value | (1 << rhs.value()),
    }
  }
}

impl BitOrAssign<PlayerOffset> for PlayerOffsetSet {
  fn bitor_assign(&mut self, rhs: PlayerOffset) {
    self.value |= 1 << rhs.value();
  }
}

impl PlayerOffsetSet {
  pub fn empty() -> Self {
    Self { value: 0 }
  }
}

impl Serialize for PlayerOffsetSet {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_u32(self.value)
  }
}

impl<'de> Deserialize<'de> for PlayerOffsetSet {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let value = u32::deserialize(deserializer)?;
    Ok(Self { value })
  }
}
