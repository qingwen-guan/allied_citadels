use crate::domain::valueobjects::AccountId;

/// AccountToAccountMessageDetails - enum representing different types of account-to-account messages
#[derive(Debug, Clone)]
pub enum AccountToAccountMessageDetails {
  /// Direct text message
  TextMessage { content: String },
  /// Friend request
  FriendRequest { sender_id: AccountId },
  /// Friend request accepted
  FriendRequestAccepted { sender_id: AccountId },
  /// Friend request declined
  FriendRequestDeclined { sender_id: AccountId },
  /// Game invitation
  GameInvitation { room_id: String, content: String },
}

impl AccountToAccountMessageDetails {
  pub const fn topic(&self) -> &'static str {
    match self {
      AccountToAccountMessageDetails::TextMessage { .. } => "text_message",
      AccountToAccountMessageDetails::FriendRequest { .. } => "friend_request",
      AccountToAccountMessageDetails::FriendRequestAccepted { .. } => "friend_request_accepted",
      AccountToAccountMessageDetails::FriendRequestDeclined { .. } => "friend_request_declined",
      AccountToAccountMessageDetails::GameInvitation { .. } => "game_invitation",
    }
  }
}
