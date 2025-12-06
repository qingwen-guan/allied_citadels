mod create;
mod enter;
mod enter_and_take_random_seat;
mod leave;
mod list;

use room_context::services::RoomService;
use user_context::services::UserService;

use crate::cli::RoomCommand;

pub async fn handle_room_command(
  command: RoomCommand, user_service: UserService, room_service: RoomService,
) -> Result<(), Box<dyn std::error::Error>> {
  match command {
    RoomCommand::List {
      session_id,
      offset,
      limit,
    } => list::execute(user_service, room_service, session_id, offset, limit).await,
    RoomCommand::Create {
      session_id,
      name,
      max_players,
    } => create::execute(user_service, room_service, session_id, name, max_players).await,
    RoomCommand::Enter { session_id, room_id } => {
      enter::execute(user_service, room_service, session_id, room_id).await
    },
    RoomCommand::EnterAndTakeRandomSeat { session_id, room_id } => {
      enter_and_take_random_seat::execute(user_service, room_service, session_id, room_id).await
    },
    RoomCommand::Leave { session_id, room_id } => {
      leave::execute(user_service, room_service, session_id, room_id).await
    },
  }
}
