use serde::{Deserialize, Serialize};
use valuable::{Valuable, Value};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Role {
  刺客 = 1 << 0,
  小偷 = 1 << 1,
  魔术师 = 1 << 2,
  国王 = 1 << 3,
  主教 = 1 << 4,
  商人 = 1 << 5,
  建筑师 = 1 << 6,
  军阀 = 1 << 7,
}

impl From<isize> for Role {
  fn from(value: isize) -> Self {
    match value {
      0b00000001 => Role::刺客,
      0b00000010 => Role::小偷,
      0b00000100 => Role::魔术师,
      0b00001000 => Role::国王,
      0b00010000 => Role::主教,
      0b00100000 => Role::商人,
      0b01000000 => Role::建筑师,
      0b10000000 => Role::军阀,
      _ => panic!("Invalid role value: {}", value),
    }
  }
}

impl Role {
  pub fn population() -> [Role; 8] {
    [
      Role::刺客,
      Role::小偷,
      Role::魔术师,
      Role::国王,
      Role::主教,
      Role::商人,
      Role::建筑师,
      Role::军阀,
    ]
  }

  pub fn name(&self) -> &'static str {
    match self {
      Role::刺客 => "刺客",
      Role::小偷 => "小偷",
      Role::魔术师 => "魔术师",
      Role::国王 => "国王",
      Role::主教 => "主教",
      Role::商人 => "商人",
      Role::建筑师 => "建筑师",
      Role::军阀 => "军阀",
    }
  }
}

impl Valuable for Role {
  fn as_value(&self) -> Value<'_> {
    Value::String(self.name())
  }

  fn visit(&self, visit: &mut dyn valuable::Visit) {
    visit.visit_value(Value::String(self.name()));
  }
}
