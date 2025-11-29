use room_context::RoomService;
use user_context::UserService;

pub async fn execute(
  user_service: UserService, room_service: RoomService, session_id_str: String,
) -> Result<(), Box<dyn std::error::Error>> {
  // Verify session exists (for authentication/authorization)
  let _session_info = user_service
    .get_session(&session_id_str)
    .await?
    .ok_or_else(|| format!("Session not found: {}", session_id_str))?;

  // List all active (non-expired) rooms
  let rooms = room_service.list_active_rooms(None).await?;

  if rooms.is_empty() {
    println!("No active rooms found.");
    return Ok(());
  }

  println!("Found {} active room(s):", rooms.len());
  println!();
  for room in rooms {
    let created_at_local = room.created_at().with_timezone(&chrono::Local);
    let expires_at_local = room.expires_at().with_timezone(&chrono::Local);
    println!(
      "  UUID: {}, Number: {:06}, Name: {}, Creator: {}, Max Players: {}, Created: {}, Expires: {}",
      room.id(),
      room.number().value(),
      room.name().as_str(),
      room.creator(),
      room.max_players().value(),
      created_at_local.format("%Y-%m-%d %H:%M:%S"),
      expires_at_local.format("%Y-%m-%d %H:%M:%S")
    );
  }

  Ok(())
}
