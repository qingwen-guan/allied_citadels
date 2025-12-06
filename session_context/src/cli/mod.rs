mod login;
mod room;

use clap::{Parser, Subcommand};
use room_context::services::RoomService;
use user_context::services::UserService;

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
  /// Start HTTP and WebSocket server
  Serve,
}

#[derive(Subcommand)]
pub enum RoomCommand {
  /// List all active (non-expired) rooms
  List {
    /// Session ID of the user
    #[arg(short = 's', long = "session-id")]
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
    #[arg(short = 's', long = "session-id")]
    session_id: String,
    /// Room name
    name: String,
    /// Maximum number of players (4 or 6)
    max_players: usize,
  },
  /// Enter a room
  Enter {
    /// Session ID of the user entering the room
    #[arg(short = 's', long = "session-id")]
    session_id: String,
    /// Room ID (UUID)
    room_id: String,
  },
  /// Enter a room and take a random available seat
  #[command(name = "enter-and-take-random-seat")]
  EnterAndTakeRandomSeat {
    /// Session ID of the user entering the room
    #[arg(short = 's', long = "session-id")]
    session_id: String,
    /// Room ID (UUID)
    room_id: String,
  },
  /// Leave a room
  Leave {
    /// Session ID of the user leaving the room
    #[arg(short = 's', long = "session-id")]
    session_id: String,
    /// Room ID (UUID)
    room_id: String,
  },
}

pub async fn handle_command(
  command: Command, user_service: UserService, room_service: RoomService, server_addr: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  match command {
    Command::Login { nickname, password } => login::execute(user_service, nickname, password).await,
    Command::Room { command } => room::handle_room_command(command, user_service, room_service).await,
    Command::Serve => crate::server::start_server(server_addr, user_service, room_service).await,
  }
}
