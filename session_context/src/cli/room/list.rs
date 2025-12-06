use crate::services::SessionService;
use room_context::services::RoomService;

pub async fn execute(
  session_service: SessionService, room_service: RoomService, session_id_str: String, offset: Option<usize>,
  limit: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
  // Verify session exists (for authentication/authorization)
  session_service
    .verify_session(&session_id_str)
    .await
    .map_err(|e| format!("Session verification failed: {}", e))?;

  // List all active (non-expired) rooms with detailed information
  let rooms = room_service.list_active_rooms_detailed(offset, limit).await?;

  if rooms.is_empty() {
    println!("No active rooms found.");
    return Ok(());
  }

  println!("Found {} active room(s):", rooms.len());
  println!();
  for room in rooms {
    let created_at_local = room.created_at.with_timezone(&chrono::Local);
    let expires_at_local = room.expires_at.with_timezone(&chrono::Local);
    println!(
      "  UUID: {}, Number: {:06}, Name: {}, Creator: {} ({}), Max Players: {}, Seated: {}, Created: {}, Expires: {}",
      room.id,
      room.number,
      room.name,
      room.creator_name,
      room.creator_id,
      room.max_players,
      room.seated_players,
      created_at_local.format("%Y-%m-%d %H:%M:%S"),
      expires_at_local.format("%Y-%m-%d %H:%M:%S")
    );
  }

  Ok(())
}
