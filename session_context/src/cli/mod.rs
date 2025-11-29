mod login;
mod room;

use clap::{Parser, Subcommand};
use room_context::RoomService;
use user_context::UserService;

#[derive(Parser)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
  /// Login command
  Login { nickname: String, password: String },
  /// Room management commands
  Room {
    #[command(subcommand)]
    command: RoomCommand,
  },
}

#[derive(Subcommand)]
pub enum RoomCommand {
  /// List all active (non-expired) rooms
  List {
    session_id: String,
    /// Offset for pagination
    #[arg(long)]
    offset: Option<usize>,
    /// Limit for pagination
    #[arg(long)]
    limit: Option<usize>,
  },
  /// Create a new room
  Create {
    /// Session ID of the user creating the room
    session_id: String,
    /// Room name
    name: String,
    /// Maximum number of players (4 or 6)
    max_players: usize,
  },
  /// Enter a room
  Enter {
    /// Session ID of the user entering the room
    session_id: String,
    /// Room ID (UUID)
    room_id: String,
  },
  /// Enter a room and take a random available seat
  #[command(name = "enter-and-take-random-seat")]
  EnterAndTakeRandomSeat {
    /// Session ID of the user entering the room
    session_id: String,
    /// Room ID (UUID)
    room_id: String,
  },
}

pub async fn handle_command(
  command: Command, user_service: UserService, room_service: RoomService,
) -> Result<(), Box<dyn std::error::Error>> {
  match command {
    Command::Login { nickname, password } => login::execute(user_service, nickname, password).await,
    Command::Room { command } => room::handle_room_command(command, user_service, room_service).await,
  }
}
