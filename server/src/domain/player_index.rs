use std::ops::{Add, Sub};

use serde::{Deserialize, Serialize};
use valuable::Valuable;

#[derive(Copy, Clone, PartialEq, PartialOrd, Valuable, Debug, Ord, Eq)]
pub struct PlayerIndex {
  value: usize,
}

impl PlayerIndex {
  // 这里故意没impl trait From<usize>
  pub fn from_usize(value: usize) -> Self {
    Self { value }
  }

  pub fn value(&self) -> usize {
    self.value
  }

  pub fn invalid() -> Self {
    Self { value: usize::MAX }
  }
}

impl Sub for PlayerIndex {
  type Output = usize;

  fn sub(self, other: Self) -> Self::Output {
    self.value - other.value
  }
}

impl Add<usize> for PlayerIndex {
  type Output = PlayerIndex;

  fn add(self, other: usize) -> Self::Output {
    Self::from_usize(self.value + other)
  }
}

impl Serialize for PlayerIndex {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_u64(self.value as u64)
  }
}

impl<'de> Deserialize<'de> for PlayerIndex {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let value = usize::deserialize(deserializer)?;
    Ok(Self::from_usize(value))
  }
}
