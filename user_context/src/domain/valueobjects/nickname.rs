use std::fmt;

/// NickName - value object for nickname to provide type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NickName(String);

impl NickName {
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

impl fmt::Display for NickName {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<String> for NickName {
  fn from(value: String) -> Self {
    Self::new(value)
  }
}

impl From<NickName> for String {
  fn from(value: NickName) -> Self {
    value.into_string()
  }
}

impl From<&str> for NickName {
  fn from(value: &str) -> Self {
    Self::new(value.to_string())
  }
}
