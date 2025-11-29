mod config;
mod domain;
mod error;
mod infra;
mod migrations;
pub mod room_service;

pub use config::Config;
pub use domain::valueobjects::{MaxPlayers, RoomId, RoomName, RoomNumber, SeatNumber};
pub use domain::{RawMessageRepository, Room, RoomManager, RoomParticipant, RoomRepository};
pub use error::RoomError;
pub use infra::{PostgresMessageRepository, PostgresRoomRepository};
pub use migrations::{
  create_all_tables, create_room_participant_table, create_room_table, create_room_to_user_message_table,
  drop_room_table,
};
pub use room_service::RoomService;
