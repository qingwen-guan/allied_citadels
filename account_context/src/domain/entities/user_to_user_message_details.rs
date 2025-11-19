use crate::domain::valueobjects::UserId;

/// UserToUserMessageDetails - enum representing different types of user-to-user messages
#[derive(Debug, Clone)]
pub enum UserToUserMessageDetails {
  /// Direct text message
  TextMessage { content: String },
  /// Friend request
  FriendRequest { sender_id: UserId },
  /// Friend request accepted
  FriendRequestAccepted { sender_id: UserId },
  /// Friend request declined
  FriendRequestDeclined { sender_id: UserId },
  /// Game invitation
  GameInvitation { room_id: String, content: String },
}

impl UserToUserMessageDetails {
  pub const fn topic(&self) -> &'static str {
    match self {
      UserToUserMessageDetails::TextMessage { .. } => "text_message",
      UserToUserMessageDetails::FriendRequest { .. } => "friend_request",
      UserToUserMessageDetails::FriendRequestAccepted { .. } => "friend_request_accepted",
      UserToUserMessageDetails::FriendRequestDeclined { .. } => "friend_request_declined",
      UserToUserMessageDetails::GameInvitation { .. } => "game_invitation",
    }
  }
}
