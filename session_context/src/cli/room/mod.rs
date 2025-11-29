mod create;
mod enter;
mod list;

use room_context::RoomService;
use user_context::UserService;

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
    RoomCommand::Enter { session_id, room_id } => enter::execute(user_service, room_service, session_id, room_id).await,
  }
}
