use account_context::AccountId;
use uuid::Uuid;

pub use super::room_to_account_message_details::RoomToAccountMessageDetails;
use crate::domain::valueobjects::{MessageContent, MessageTopic, RoomId, RoomToAccountMessageId};

/// RoomToAccountMessage - entity representing a message from room to account
/// with typed message details
#[derive(Debug, Clone)]
pub struct RoomToAccountMessage {
  #[allow(dead_code)] // TODO: remove later
  id: RoomToAccountMessageId,
  from: RoomId,
  to: AccountId,
  details: RoomToAccountMessageDetails,
}

impl RoomToAccountMessage {
  pub fn new(from: RoomId, to: AccountId, details: RoomToAccountMessageDetails) -> Self {
    let id = RoomToAccountMessageId::from(Uuid::new_v4());
    Self { id, from, to, details }
  }

  #[allow(dead_code)] // TODO: remove later
  pub fn with_id(
    id: RoomToAccountMessageId, from: RoomId, to: AccountId, details: RoomToAccountMessageDetails,
  ) -> Self {
    Self { id, from, to, details }
  }

  #[allow(dead_code)] // TODO: remove later
  pub fn id(&self) -> RoomToAccountMessageId {
    self.id
  }

  #[allow(dead_code)] // TODO: remove later
  pub fn from(&self) -> RoomId {
    self.from
  }

  #[allow(dead_code)] // TODO: remove later
  pub fn to(&self) -> AccountId {
    self.to
  }

  pub fn room_id(&self) -> RoomId {
    self.from
  }

  pub fn account_id(&self) -> AccountId {
    self.to
  }

  #[allow(dead_code)] // TODO: remove later
  pub fn details(&self) -> &RoomToAccountMessageDetails {
    &self.details
  }

  #[allow(dead_code)] // TODO: remove later
  pub fn into_details(self) -> RoomToAccountMessageDetails {
    self.details
  }

  pub fn topic(&self) -> MessageTopic {
    self.details.topic()
  }

  pub fn content(&self) -> MessageContent {
    let topic = self.topic();
    let verb = topic.as_str();
    let content_str = match &self.details {
      RoomToAccountMessageDetails::RoomStateUpdate {
        room_id,
        from,
        to,
        content,
      } => serde_json::json!({
        "verb": verb,
        "room_id": room_id.to_string(),
        "from": from.as_str(),
        "to": to.as_str(),
        "content": content
      })
      .to_string(),
      RoomToAccountMessageDetails::RoomDeleted { room_id } => serde_json::json!({
        "verb": verb,
        "room_id": room_id.to_string()
      })
      .to_string(),
      RoomToAccountMessageDetails::MaxPlayersUpdated { room_id, from, to } => serde_json::json!({
        "verb": verb,
        "room_id": room_id.to_string(),
        "from": from.value() as u32,
        "to": to.value() as u32
      })
      .to_string(),
      RoomToAccountMessageDetails::ForceStandUp {
        room_id,
        account_id,
        reason,
      } => {
        let mut json = serde_json::json!({
          "verb": verb,
          "room_id": room_id.to_string(),
          "account_id": account_id.to_string()
        });
        if let Some(reason) = reason {
          json["reason"] = serde_json::json!(reason);
        }
        json.to_string()
      },
    };
    MessageContent::from(content_str)
  }
}
