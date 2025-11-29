use room_context::RoomService;
use user_context::UserService;

pub async fn execute(
  user_service: UserService, room_service: RoomService, session_id_str: String, room_id_str: String,
) -> Result<(), Box<dyn std::error::Error>> {
  // Verify session exists and get user_id
  let session_info = user_service
    .get_session(&session_id_str)
    .await?
    .ok_or_else(|| format!("Session not found: {}", session_id_str))?;

  // Enter the room (room_service will parse the IDs)
  match room_service.enter_room(&session_info.user_id, &room_id_str).await? {
    room_context::EnterRoomResult::Success => {
      println!(
        "User {} entered room {} and is standing by",
        session_info.user_id, room_id_str
      );
    },
    room_context::EnterRoomResult::AlreadyInRoom => {
      println!(
        "User {} is already in room {} - no action required",
        session_info.user_id, room_id_str
      );
    },
  }

  Ok(())
}
