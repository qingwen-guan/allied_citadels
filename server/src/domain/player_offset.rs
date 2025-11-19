use serde::{Deserialize, Serialize};
use valuable::{Valuable, Value};

use crate::domain::PlayerIndex;

// 0: 自己
// 1: 下家
// ...
#[derive(Debug, Clone, Copy)]
pub struct PlayerOffset {
  value: usize,
}

impl PlayerOffset {
  pub const ZERO: Self = Self { value: 0 };

  // 这里故意没impl trait From<usize>
  pub fn from_usize(value: usize) -> Self {
    Self { value }
  }

  pub fn from_index(player: PlayerIndex, observer: PlayerIndex, n: usize) -> Self {
    if player >= observer {
      Self {
        value: player - observer,
      }
    } else {
      Self {
        value: player + n - observer,
      }
    }
  }

  pub fn to_index(self, observer: PlayerIndex, n: usize) -> PlayerIndex {
    let value = observer.value() + self.value();
    if value >= n {
      PlayerIndex::from_usize(value - n)
    } else {
      PlayerIndex::from_usize(value)
    }
  }

  pub fn is_zero(&self) -> bool {
    self.value == 0
  }

  pub fn value(&self) -> usize {
    self.value
  }
}

impl Valuable for PlayerOffset {
  fn as_value(&self) -> Value<'_> {
    Value::Usize(self.value)
  }

  fn visit(&self, visit: &mut dyn valuable::Visit) {
    visit.visit_value(Value::Usize(self.value));
  }
}

impl Serialize for PlayerOffset {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_u64(self.value as u64)
  }
}

impl<'de> Deserialize<'de> for PlayerOffset {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let value = u64::deserialize(deserializer)?;
    Ok(Self { value: value as usize })
  }
}
