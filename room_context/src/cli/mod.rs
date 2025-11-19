mod list;

use account_context::UserId;
use clap::{Parser, Subcommand};
use room_context::{MaxPlayers, RoomId, RoomService};

#[derive(Parser)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
  /// Room management commands
  Room {
    #[command(subcommand)]
    command: RoomCommand,
  },
  Serve,
  /// Database migration commands
  Migrates {
    #[command(subcommand)]
    command: MigrateCommand,
  },
}

#[derive(Subcommand)]
pub enum RoomCommand {
  /// List all rooms
  List,
  /// Create a new room
  Create {
    name: String,
    creator: String,
    max_players: usize,
  },
  /// Get room by name
  Get { name: String },
  /// Get room by UUID
  GetByUuid { uuid: String },
  /// Delete room by UUID
  Delete { uuid: String },
  /// Update room name
  UpdateName { uuid: String, new_name: String },
  /// Update room max players
  UpdateMaxPlayers { uuid: String, max_players: usize },
  // TODO: add chat command
}

#[derive(Subcommand)]
pub enum MigrateCommand {
  /// Create the room table in the database
  CreateRoomTable,
  /// Drop the room table from the database
  DropRoomTable,
  /// Create the room_to_user_message table in the database
  CreateRoomToUserMessageTable,
  /// Create the user_to_room_message table in the database
  CreateUserToRoomMessageTable,
  /// Create the user table in the database (delegated to account_context)
  CreateUserTable,
  /// Drop all tables from the database
  DropAllTables,
}

pub async fn handle_command(command: Command, room_service: RoomService) -> Result<(), Box<dyn std::error::Error>> {
  match command {
    Command::Room { command } => handle_room_command(command, room_service).await,
    Command::Serve => {
      // Serve command is handled separately in main.rs since it needs the router
      unreachable!("Serve command should be handled in main.rs")
    },
    Command::Migrates { .. } => {
      // Migrates commands are handled directly in main.rs since they don't need RoomService
      unreachable!("Migrates commands should be handled in main.rs")
    },
  }
}

async fn handle_room_command(
  command: RoomCommand, room_service: RoomService,
) -> Result<(), Box<dyn std::error::Error>> {
  match command {
    RoomCommand::List => list::execute(room_service).await,
    RoomCommand::Create {
      name,
      creator,
      max_players,
    } => {
      let creator_uuid = creator.parse::<uuid::Uuid>()?;
      let creator_id = UserId::from(creator_uuid);
      let max_players = MaxPlayers::try_from(max_players)?;
      let room = room_service.create_room(&name, creator_id, max_players).await?;
      println!(
        "Room created: uuid={}, number={}, name={}, creator={}, max_players={}",
        room.id(),
        room.number().value(),
        room.name().as_str(),
        room.creator(),
        room.max_players().value()
      );
      Ok(())
    },
    RoomCommand::Get { name } => {
      match room_service.get_room_by_name(&name).await? {
        Some(room) => {
          println!(
            "Room found: uuid={}, number={}, name={}, creator={}, max_players={}, created_at={}",
            room.id(),
            room.number().value(),
            room.name().as_str(),
            room.creator(),
            room.max_players().value(),
            room.created_at()
          );
        },
        None => {
          println!("Room not found with name: {}", name);
        },
      }
      Ok(())
    },
    RoomCommand::GetByUuid { uuid } => {
      let uuid = uuid.parse::<uuid::Uuid>()?;
      let room_id = RoomId::from(uuid);
      match room_service.get_room_by_id(room_id).await? {
        Some(room) => {
          println!(
            "Room found: uuid={}, number={}, name={}, creator={}, max_players={}, created_at={}",
            room.id(),
            room.number().value(),
            room.name().as_str(),
            room.creator(),
            room.max_players().value(),
            room.created_at()
          );
        },
        None => {
          println!("Room not found with UUID: {}", uuid);
        },
      }
      Ok(())
    },
    RoomCommand::Delete { uuid } => {
      let uuid = uuid.parse::<uuid::Uuid>()?;
      let room_id = RoomId::from(uuid);
      room_service.delete_room(room_id).await?;
      println!("Deleted room with UUID: {}", uuid);
      Ok(())
    },
    RoomCommand::UpdateName { uuid, new_name } => {
      let uuid = uuid.parse::<uuid::Uuid>()?;
      room_service.update_room_name(uuid, &new_name).await?;
      println!("Updated room name for UUID: {}", uuid);
      Ok(())
    },
    RoomCommand::UpdateMaxPlayers { uuid, max_players } => {
      let uuid = uuid.parse::<uuid::Uuid>()?;
      let room_id = RoomId::from(uuid);
      room_service.update_room_max_players(room_id, max_players).await?;
      println!("Updated max players for UUID: {}", uuid);
      Ok(())
    },
  }
}
