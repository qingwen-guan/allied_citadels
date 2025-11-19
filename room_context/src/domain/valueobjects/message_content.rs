use std::fmt;

/// MessageContent - value object for message content to provide type safety
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageContent(String);

impl MessageContent {
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

impl fmt::Display for MessageContent {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<String> for MessageContent {
  fn from(value: String) -> Self {
    Self::new(value)
  }
}

impl From<MessageContent> for String {
  fn from(value: MessageContent) -> Self {
    value.into_string()
  }
}

impl From<&str> for MessageContent {
  fn from(value: &str) -> Self {
    Self::new(value.to_string())
  }
}

// Implement sqlx traits to allow direct binding
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for MessageContent {
  fn encode_by_ref(
    &self, buf: &mut sqlx::postgres::PgArgumentBuffer,
  ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
    <String as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&self.0, buf)
  }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for MessageContent {
  fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
    let s = <String as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
    Ok(MessageContent::from(s))
  }
}

impl sqlx::Type<sqlx::Postgres> for MessageContent {
  fn type_info() -> sqlx::postgres::PgTypeInfo {
    <String as sqlx::Type<sqlx::Postgres>>::type_info()
  }
}
