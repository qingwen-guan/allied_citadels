use crate::domain::valueobjects::AccountToRoomMessageId;

/// AccountToRoomMessage - entity representing a message from account to room
/// with typed message details
#[allow(dead_code)] // TODO: remove later
#[derive(Debug, Clone)]
pub struct AccountToRoomMessage {
  id: AccountToRoomMessageId,
  details: AccountToRoomMessageDetails,
}

impl AccountToRoomMessage {
  #[allow(dead_code)] // TODO: remove later
  pub fn new(id: AccountToRoomMessageId, details: AccountToRoomMessageDetails) -> Self {
    Self { id, details }
  }

  #[allow(dead_code)] // TODO: remove later
  pub fn id(&self) -> AccountToRoomMessageId {
    self.id
  }

  #[allow(dead_code)] // TODO: remove later
  pub fn details(&self) -> &AccountToRoomMessageDetails {
    &self.details
  }

  #[allow(dead_code)] // TODO: remove later
  pub fn into_details(self) -> AccountToRoomMessageDetails {
    self.details
  }

  #[allow(dead_code)] // TODO: remove later
  pub fn topic(&self) -> &'static str {
    self.details.topic()
  }

  #[allow(dead_code)] // TODO: remove later
  pub fn content(&self) -> String {
    let verb = self.topic();
    match &self.details {
      AccountToRoomMessageDetails::Chat { content } => serde_json::json!({
        "verb": verb,
        "content": content
      })
      .to_string(),
    }
  }
}

/// AccountToRoomMessageDetails - enum representing different types of account to room messages
#[allow(dead_code)] // TODO: remove later
#[derive(Debug, Clone)]
pub enum AccountToRoomMessageDetails {
  /// Chat message
  Chat { content: String },
}

impl AccountToRoomMessageDetails {
  #[allow(dead_code)] // TODO: remove later
  pub const fn topic(&self) -> &'static str {
    match self {
      AccountToRoomMessageDetails::Chat { .. } => "chat",
    }
  }
}
