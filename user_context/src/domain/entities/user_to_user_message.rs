pub use super::user_to_user_message_details::UserToUserMessageDetails;
use crate::domain::valueobjects::{UserId, UserToUserMessageId};

/// UserToUserMessage - entity representing a message from one user to another
/// with typed message details
#[derive(Debug, Clone)]
pub struct UserToUserMessage {
  id: UserToUserMessageId,
  from: UserId,
  to: UserId,
  details: UserToUserMessageDetails,
}

impl UserToUserMessage {
  pub fn new(id: UserToUserMessageId, from: UserId, to: UserId, details: UserToUserMessageDetails) -> Self {
    Self { id, from, to, details }
  }

  pub fn id(&self) -> UserToUserMessageId {
    self.id
  }

  pub fn from(&self) -> UserId {
    self.from
  }

  pub fn to(&self) -> UserId {
    self.to
  }

  pub fn details(&self) -> &UserToUserMessageDetails {
    &self.details
  }

  pub fn into_details(self) -> UserToUserMessageDetails {
    self.details
  }

  pub fn topic(&self) -> &'static str {
    self.details.topic()
  }

  pub fn content(&self) -> String {
    let verb = self.topic();
    match &self.details {
      UserToUserMessageDetails::TextMessage { content } => serde_json::json!({
        "message_id": self.id.to_string(),
        "verb": verb,
        "sender_id": self.from.to_string(),
        "receiver_id": self.to.to_string(),
        "content": content
      })
      .to_string(),
      UserToUserMessageDetails::FriendRequest {
        sender_id: details_sender_id,
      } => serde_json::json!({
        "message_id": self.id.to_string(),
        "verb": verb,
        "sender_id": self.from.to_string(),
        "receiver_id": self.to.to_string(),
        "intro": details_sender_id.to_string()
      })
      .to_string(),
      UserToUserMessageDetails::FriendRequestAccepted { .. } => serde_json::json!({
        "message_id": self.id.to_string(),
        "verb": verb,
        "sender_id": self.from.to_string(),
        "receiver_id": self.to.to_string()
      })
      .to_string(),
      UserToUserMessageDetails::FriendRequestDeclined { .. } => serde_json::json!({
        "message_id": self.id.to_string(),
        "verb": verb,
        "sender_id": self.from.to_string(),
        "receiver_id": self.to.to_string()
      })
      .to_string(),
      UserToUserMessageDetails::GameInvitation { room_id, content } => serde_json::json!({
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
