use std::fmt;

/// RoomName - value object for room name to provide type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoomName(String);

impl RoomName {
  pub fn new(value: String) -> Self {
    Self(value)
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }

  pub fn into_string(self) -> String {
    self.0
  }
}

impl fmt::Display for RoomName {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<String> for RoomName {
  fn from(value: String) -> Self {
    Self::new(value)
  }
}

impl From<RoomName> for String {
  fn from(value: RoomName) -> Self {
    value.into_string()
  }
}

impl From<&str> for RoomName {
  fn from(value: &str) -> Self {
    Self::new(value.to_string())
  }
}
