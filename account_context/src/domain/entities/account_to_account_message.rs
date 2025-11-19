pub use super::account_to_account_message_details::AccountToAccountMessageDetails;
use crate::domain::valueobjects::{AccountId, AccountToAccountMessageId};

/// AccountToAccountMessage - entity representing a message from one account to another
/// with typed message details
#[derive(Debug, Clone)]
pub struct AccountToAccountMessage {
  id: AccountToAccountMessageId,
  from: AccountId,
  to: AccountId,
  details: AccountToAccountMessageDetails,
}

impl AccountToAccountMessage {
  pub fn new(
    id: AccountToAccountMessageId, from: AccountId, to: AccountId, details: AccountToAccountMessageDetails,
  ) -> Self {
    Self { id, from, to, details }
  }

  pub fn id(&self) -> AccountToAccountMessageId {
    self.id
  }

  pub fn from(&self) -> AccountId {
    self.from
  }

  pub fn to(&self) -> AccountId {
    self.to
  }

  pub fn details(&self) -> &AccountToAccountMessageDetails {
    &self.details
  }

  pub fn into_details(self) -> AccountToAccountMessageDetails {
    self.details
  }

  pub fn topic(&self) -> &'static str {
    self.details.topic()
  }

  pub fn content(&self) -> String {
    let verb = self.topic();
    match &self.details {
      AccountToAccountMessageDetails::TextMessage { content } => serde_json::json!({
        "message_id": self.id.to_string(),
        "verb": verb,
        "sender_id": self.from.to_string(),
        "receiver_id": self.to.to_string(),
        "content": content
      })
      .to_string(),
      AccountToAccountMessageDetails::FriendRequest {
        sender_id: details_sender_id,
      } => serde_json::json!({
        "message_id": self.id.to_string(),
        "verb": verb,
        "sender_id": self.from.to_string(),
        "receiver_id": self.to.to_string(),
        "intro": details_sender_id.to_string()
      })
      .to_string(),
      AccountToAccountMessageDetails::FriendRequestAccepted { .. } => serde_json::json!({
        "message_id": self.id.to_string(),
        "verb": verb,
        "sender_id": self.from.to_string(),
        "receiver_id": self.to.to_string()
      })
      .to_string(),
      AccountToAccountMessageDetails::FriendRequestDeclined { .. } => serde_json::json!({
        "message_id": self.id.to_string(),
        "verb": verb,
        "sender_id": self.from.to_string(),
        "receiver_id": self.to.to_string()
      })
      .to_string(),
      AccountToAccountMessageDetails::GameInvitation { room_id, content } => serde_json::json!({
        "message_id": self.id.to_string(),
        "verb": verb,
        "sender_id": self.from.to_string(),
        "receiver_id": self.to.to_string(),
        "room_id": room_id,
        "content": content
      })
      .to_string(),
    }
  }
}
