use account_context::AccountId;

use crate::domain::valueobjects::{MaxPlayers, MessageTopic, RoomId, RoomState};

/// RoomToAccountMessageDetails - enum representing different types of room to account messages
#[derive(Debug, Clone)]
pub enum RoomToAccountMessageDetails {
  /// Room state update
  #[allow(dead_code)] // TODO: remove later
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
    account_id: AccountId,
    reason: Option<String>,
  },
}

impl RoomToAccountMessageDetails {
  pub fn topic(&self) -> MessageTopic {
    let topic_str = match self {
      RoomToAccountMessageDetails::RoomStateUpdate { .. } => "room_state_update",
      RoomToAccountMessageDetails::RoomDeleted { .. } => "delete_room",
      RoomToAccountMessageDetails::MaxPlayersUpdated { .. } => "update_max_players",
      RoomToAccountMessageDetails::ForceStandUp { .. } => "force_stand_up",
    };
    MessageTopic::from(topic_str)
  }
}
