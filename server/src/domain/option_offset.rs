use serde::{Deserialize, Serialize};
use valuable::Valuable;

use crate::domain::PlayerOffset;

#[derive(Clone, Debug, Valuable)]
pub struct OptionOffset {
  value: usize,
}

impl From<PlayerOffset> for OptionOffset {
  fn from(offset: PlayerOffset) -> Self {
    Self { value: offset.value() }
  }
}

impl OptionOffset {
  const NONE_VALUE: usize = 0b1111;

  pub fn none() -> Self {
    Self {
      value: Self::NONE_VALUE,
    }
  }
}

impl Serialize for OptionOffset {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    if self.value == Self::NONE_VALUE {
      serializer.serialize_none()
    } else {
      serializer.serialize_u64(self.value as u64)
    }
  }
}

impl<'de> Deserialize<'de> for OptionOffset {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    if let Some(value) = Option::<usize>::deserialize(deserializer)? {
      Ok(Self { value })
    } else {
      Ok(Self::none())
    }
  }
}
