use user_context::domain::valueobjects::UserId;

use crate::domain::valueobjects::{MaxPlayers, MessageTopic, RoomId, RoomState};

/// RoomToUserMessageDetails - enum representing different types of room to user messages
#[derive(Debug, Clone)]
pub enum RoomToUserMessageDetails {
  /// Room state update
  #[allow(dead_code)]
  RoomStateUpdate {
    room_id: RoomId,
    from: RoomState,
    to: RoomState,
    content: String,
  },
  /// Room deleted notification
  RoomDeleted { room_id: RoomId },
  /// Max players updated
  MaxPlayersUpdated {
    room_id: RoomId,
    from: MaxPlayers,
    to: MaxPlayers,
  },
  /// Force stand up notification
  ForceStandUp {
    room_id: RoomId,
    user_id: UserId,
    reason: Option<String>,
  },
}

impl RoomToUserMessageDetails {
  pub fn topic(&self) -> MessageTopic {
    let topic_str = match self {
      RoomToUserMessageDetails::RoomStateUpdate { .. } => "room_state_update",
      RoomToUserMessageDetails::RoomDeleted { .. } => "delete_room",
      RoomToUserMessageDetails::MaxPlayersUpdated { .. } => "update_max_players",
      RoomToUserMessageDetails::ForceStandUp { .. } => "force_stand_up",
    };
    MessageTopic::from(topic_str)
  }
}
