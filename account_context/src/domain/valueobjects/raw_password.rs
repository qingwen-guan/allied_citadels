/// RawPassword - value object for plain text password to provide type safety
#[derive(Debug, Clone)]
pub struct RawPassword(String);

impl RawPassword {
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

impl From<String> for RawPassword {
  fn from(value: String) -> Self {
    Self::new(value)
  }
}

impl From<RawPassword> for String {
  fn from(value: RawPassword) -> Self {
    value.into_string()
  }
}

impl From<&str> for RawPassword {
  fn from(value: &str) -> Self {
    Self::new(value.to_string())
  }
}

impl RawPassword {
  /// Generate a random numeric password with the specified number of digits using the provided RNG
  pub fn generate_random(rng: &mut rand::rngs::StdRng, digits: u32) -> Self {
    use rand::Rng;
    let max = 10_u32.pow(digits) - 1;
    let password = format!("{:0digits$}", rng.random_range(0..=max), digits = digits as usize);
    Self::new(password)
  }

  /// Generate a random numeric password with the specified number of digits using thread-local RNG
  pub fn generate_random_default(digits: u32) -> Self {
    use rand::Rng;
    let max = 10_u32.pow(digits) - 1;
    let password = format!(
      "{:0digits$}",
      rand::rng().random_range(0..=max),
      digits = digits as usize
    );
    Self::new(password)
  }
}
