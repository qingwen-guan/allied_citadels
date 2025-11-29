mod list;
mod list_sessions;
mod login;
mod migrates;
mod rename;
mod reset_password;
mod session;

use clap::{Parser, Subcommand};
use user_context::UserService;

#[derive(Parser)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
  /// User management commands
  User {
    #[command(subcommand)]
    command: UserCommand,
  },
  /// Session management commands
  Session {
    #[command(subcommand)]
    command: SessionCommand,
  },
  Login {
    nickname: String,
    password: String,
  },
  Serve,
  /// Database migration commands
  Migrates {
    #[command(subcommand)]
    command: MigrateCommand,
  },
}

#[derive(Subcommand)]
pub enum UserCommand {
  /// List all users
  List,
  /// Create a new user
  Create { nickname: String },
  /// Get user by nickname
  Get { nickname: String },
  /// Delete user by nickname
  Delete { nickname: String },
  /// Reset password for a user
  ResetPassword { uuid_or_nickname: String },
  /// Rename a user by UUID or current nickname
  Rename {
    /// User UUID or current nickname
    uuid_or_nickname: String,
    /// New nickname
    new_nickname: String,
  },
}

#[derive(Subcommand)]
pub enum SessionCommand {
  /// List all sessions
  List,
  /// List active (non-expired) sessions
  ListActive,
  /// Get session by session ID
  Get { session_id: String },
}

#[derive(Subcommand)]
pub enum MigrateCommand {
  /// Create the user table in the database
  CreateUserTable,
  /// Create the user_session table in the database
  CreateUserSessionTable,
  /// Drop the user_session table from the database
  DropTableUserSession,
  /// Drop all tables from the database
  DropAllTables,
  /// Create all tables in the database
  CreateAllTables,
}

pub async fn handle_command(command: Command, user_service: UserService) -> Result<(), Box<dyn std::error::Error>> {
  match command {
    Command::User { command } => handle_user_command(command, user_service).await,
    Command::Session { command } => handle_session_command(command, user_service).await,
    Command::Login { nickname, password } => login::execute(user_service, nickname, password).await,
    Command::Serve => {
      // Serve command is handled separately in main.rs since it needs the router
      unreachable!("Serve command should be handled in main.rs")
    },
    Command::Migrates { .. } => {
      unreachable!("Migrates commands should be handled via handle_migrate_command")
    },
  }
}

pub async fn handle_migrate_command(dsn: &str, command: MigrateCommand) -> Result<(), Box<dyn std::error::Error>> {
  migrates::execute(dsn, command).await
}

async fn handle_user_command(
  command: UserCommand, user_service: UserService,
) -> Result<(), Box<dyn std::error::Error>> {
  match command {
    UserCommand::List => list::execute(user_service).await,
    UserCommand::Create { nickname } => {
      let response = user_service.create_user(&nickname).await?;
      println!(
        "user_id: {}, nickname: {}, password: {}",
        response.user_id, nickname, response.password
      );
      Ok(())
    },
    UserCommand::Get { nickname } => {
      let user = user_service.get_user_by_nickname(&nickname).await?;
      println!("User found: user_id={}, nickname={}", user.user_id, user.nickname);
      Ok(())
    },
    UserCommand::Delete { nickname } => {
      user_service.delete_user_by_nickname(&nickname).await?;
      println!("Deleted user with nickname: {}", nickname);
      Ok(())
    },
    UserCommand::ResetPassword { uuid_or_nickname } => reset_password::execute(user_service, uuid_or_nickname).await,
    UserCommand::Rename {
      uuid_or_nickname,
      new_nickname,
    } => rename::execute(user_service, uuid_or_nickname, new_nickname).await,
  }
}

async fn handle_session_command(
  command: SessionCommand, user_service: UserService,
) -> Result<(), Box<dyn std::error::Error>> {
  match command {
    SessionCommand::List => list_sessions::execute(user_service).await,
    SessionCommand::ListActive => session::list_active(user_service).await,
    SessionCommand::Get { session_id } => session::get(user_service, session_id).await,
  }
}
