use room_context::RoomService;
use user_context::UserService;

pub async fn execute(
  user_service: UserService, room_service: RoomService, session_id_str: String, name: String, max_players: usize,
) -> Result<(), Box<dyn std::error::Error>> {
  // Verify session exists and get user_id
  let session_info = user_service
    .get_session(&session_id_str)
    .await?
    .ok_or_else(|| format!("Session not found: {}", session_id_str))?;

  // Create the room (room_service will parse the creator string and validate max_players)
  let room = room_service
    .create_room(&name, &session_info.user_id, max_players)
    .await?;

  println!(
    "Room created successfully:\n  UUID: {}\n  Number: {:06}\n  Name: {}\n  Creator: {}\n  Max Players: {}",
    room.id(),
    room.number().value(),
    room.name().as_str(),
    room.creator(),
    room.max_players().value()
  );

  Ok(())
}
