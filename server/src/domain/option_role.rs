use serde::{Deserialize, Serialize};
use valuable::{Valuable, Value};

use crate::domain::Role;

#[derive(Copy, Clone, Debug)]
pub enum OptionRole {
  None = 0,
  刺客 = (Role::刺客 as isize),
  小偷 = (Role::小偷 as isize),
  魔术师 = (Role::魔术师 as isize),
  国王 = (Role::国王 as isize),
  主教 = (Role::主教 as isize),
  商人 = (Role::商人 as isize),
  建筑师 = (Role::建筑师 as isize),
  军阀 = (Role::军阀 as isize),
}

impl PartialEq<Role> for OptionRole {
  fn eq(&self, other: &Role) -> bool {
    (*self as isize) == (*other as isize)
  }
}

impl From<Role> for OptionRole {
  fn from(value: Role) -> Self {
    match value {
      Role::刺客 => OptionRole::刺客,
      Role::小偷 => OptionRole::小偷,
      Role::魔术师 => OptionRole::魔术师,
      Role::国王 => OptionRole::国王,
      Role::主教 => OptionRole::主教,
      Role::商人 => OptionRole::商人,
      Role::建筑师 => OptionRole::建筑师,
      Role::军阀 => OptionRole::军阀,
    }
  }
}

impl Valuable for OptionRole {
  fn as_value(&self) -> Value<'_> {
    match self {
      OptionRole::None => Value::Unit,
      OptionRole::刺客 => Value::String("刺客"),
      OptionRole::小偷 => Value::String("小偷"),
      OptionRole::魔术师 => Value::String("换牌师"),
      OptionRole::国王 => Value::String("国王"),
      OptionRole::主教 => Value::String("主教"),
      OptionRole::商人 => Value::String("商人"),
      OptionRole::建筑师 => Value::String("建筑师"),
      OptionRole::军阀 => Value::String("军阀"),
    }
  }

  fn visit(&self, _visit: &mut dyn valuable::Visit) {}
}

impl Serialize for OptionRole {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    match self {
      OptionRole::None => serializer.serialize_none(),
      OptionRole::刺客 => serializer.serialize_str("刺客"),
      OptionRole::小偷 => serializer.serialize_str("小偷"),
      OptionRole::魔术师 => serializer.serialize_str("换牌师"),
      OptionRole::国王 => serializer.serialize_str("国王"),
      OptionRole::主教 => serializer.serialize_str("主教"),
      OptionRole::商人 => serializer.serialize_str("商人"),
      OptionRole::建筑师 => serializer.serialize_str("建筑师"),
      OptionRole::军阀 => serializer.serialize_str("军阀"),
    }
  }
}

impl<'de> Deserialize<'de> for OptionRole {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    match Option::<String>::deserialize(deserializer)? {
      Some(value) if value == "刺客" => Ok(OptionRole::刺客),
      Some(value) if value == "小偷" => Ok(OptionRole::小偷),
      Some(value) if value == "换牌师" => Ok(OptionRole::魔术师),
      Some(value) if value == "国王" => Ok(OptionRole::国王),
      Some(value) if value == "主教" => Ok(OptionRole::主教),
      Some(value) if value == "商人" => Ok(OptionRole::商人),
      Some(value) if value == "建筑师" => Ok(OptionRole::建筑师),
      Some(value) if value == "军阀" => Ok(OptionRole::军阀),
      Some(value) => Err(serde::de::Error::custom(format!("Invalid role: {}", value))),
      None => Ok(OptionRole::None),
    }
  }
}
