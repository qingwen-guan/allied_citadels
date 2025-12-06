use room_context::services::RoomService;

pub async fn execute(room_service: RoomService) -> Result<(), Box<dyn std::error::Error>> {
  let offset = None;
  let limit = None;
  let rooms = room_service.list_rooms(offset, limit).await?;

  if rooms.is_empty() {
    println!("No rooms found.");
    return Ok(());
  }

  println!("Found {} room(s):", rooms.len());
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

