use user_context::UserId;
use uuid::Uuid;

pub use super::room_to_user_message_details::RoomToUserMessageDetails;
use crate::domain::valueobjects::{MessageContent, MessageTopic, RoomId, RoomToUserMessageId};

/// RoomToUserMessage - entity representing a message from room to user
/// with typed message details
#[derive(Debug, Clone)]
pub struct RoomToUserMessage {
  #[allow(dead_code)]
  id: RoomToUserMessageId,
  from: RoomId,
  to: UserId,
  details: RoomToUserMessageDetails,
}

impl RoomToUserMessage {
  pub fn new(from: RoomId, to: UserId, details: RoomToUserMessageDetails) -> Self {
    let id = RoomToUserMessageId::from(Uuid::new_v4());
    Self { id, from, to, details }
  }

  pub fn room_id(&self) -> RoomId {
    self.from
  }

  pub fn user_id(&self) -> UserId {
    self.to
  }

  pub fn topic(&self) -> MessageTopic {
    self.details.topic()
  }

  pub fn content(&self) -> MessageContent {
    let topic = self.topic();
    let verb = topic.as_str();
    let content_str = match &self.details {
      RoomToUserMessageDetails::RoomStateUpdate {
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
      RoomToUserMessageDetails::RoomDeleted { room_id } => serde_json::json!({
        "verb": verb,
        "room_id": room_id.to_string()
      })
      .to_string(),
      RoomToUserMessageDetails::MaxPlayersUpdated { room_id, from, to } => serde_json::json!({
        "verb": verb,
        "room_id": room_id.to_string(),
        "from": from.value() as u32,
        "to": to.value() as u32
      })
      .to_string(),
      RoomToUserMessageDetails::ForceStandUp {
        room_id,
        user_id,
        reason,
      } => {
        let mut json = serde_json::json!({
          "verb": verb,
          "room_id": room_id.to_string(),
          "user_id": user_id.to_string()
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
