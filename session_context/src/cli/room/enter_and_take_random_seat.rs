use room_context::{RoomService, room_service};
use user_context::UserService;

pub async fn execute(
  user_service: UserService, room_service: RoomService, session_id_str: String, room_id_str: String,
) -> Result<(), Box<dyn std::error::Error>> {
  // Verify session exists and get user_id
  let session_info = user_service
    .get_session(&session_id_str)
    .await?
    .ok_or_else(|| format!("Session not found: {}", session_id_str))?;

  // Enter the room and take a random seat
  match room_service
    .enter_room_and_take_random_seat(&session_info.user_id, &room_id_str)
    .await?
  {
    room_service::EnterRoomRandomSeatOutcome::Success(seat_number) => {
      println!(
        "User {} entered room {} and took seat {}",
        session_info.user_id,
        room_id_str,
        seat_number.value()
      );
    },
    room_service::EnterRoomRandomSeatOutcome::AlreadyInRoom => {
      println!(
        "User {} is already in room {} - no action required",
        session_info.user_id, room_id_str
      );
    },
    room_service::EnterRoomRandomSeatOutcome::NoSeatsAvailable => {
      println!(
        "User {} entered room {} but no seats are available",
        session_info.user_id, room_id_str
      );
    },
  }

  Ok(())
}
