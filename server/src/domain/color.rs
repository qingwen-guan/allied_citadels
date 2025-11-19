use serde::{Deserialize, Serialize};
use valuable::{Valuable, Value};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Color {
  绿 = 0,
  黄 = 1,
  蓝 = 2,
  红 = 3,
  紫 = 4,
}

impl Color {
  pub fn name(&self) -> &'static str {
    match self {
      Color::绿 => "绿",
      Color::黄 => "黄",
      Color::蓝 => "蓝",
      Color::红 => "红",
      Color::紫 => "紫",
    }
  }
}

impl Valuable for Color {
  fn as_value(&self) -> Value<'_> {
    Value::String(self.name())
  }

  fn visit(&self, visit: &mut dyn valuable::Visit) {
    visit.visit_value(Value::String(self.name()));
  }
}
