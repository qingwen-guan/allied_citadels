use serde::{Deserialize, Serialize};
use valuable::{Valuable, Value};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Camp {
  楚 = 0,
  汉 = 1,
}

impl Camp {
  pub fn name(&self) -> &'static str {
    match self {
      Camp::楚 => "楚",
      Camp::汉 => "汉",
    }
  }
}

impl Valuable for Camp {
  fn as_value(&self) -> Value<'_> {
    Value::String(self.name())
  }

  fn visit(&self, visit: &mut dyn valuable::Visit) {
    visit.visit_value(Value::String(self.name()));
  }
}
