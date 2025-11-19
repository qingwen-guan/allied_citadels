/// Salt - value object for password salt to provide type safety
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Salt(String);

impl Salt {
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

impl From<String> for Salt {
  fn from(value: String) -> Self {
    Self::new(value)
  }
}

impl From<Salt> for String {
  fn from(value: Salt) -> Self {
    value.into_string()
  }
}

impl From<&str> for Salt {
  fn from(value: &str) -> Self {
    Self::new(value.to_string())
  }
}
