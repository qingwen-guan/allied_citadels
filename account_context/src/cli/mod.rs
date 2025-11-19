mod list;
mod list_sessions;
mod login;
mod reset_password;
mod session;

use account_context::AccountService;
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
  /// Account management commands
  Account {
    #[command(subcommand)]
    command: AccountCommand,
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
pub enum AccountCommand {
  /// List all accounts
  List,
  /// Create a new account
  Create { nickname: String },
  /// Get account by nickname
  Get { nickname: String },
  /// Delete account by nickname
  Delete { nickname: String },
  /// Reset password for an account
  ResetPassword { uuid_or_nickname: String },
}

#[derive(Subcommand)]
pub enum SessionCommand {
  /// List all sessions
  List,
  /// List non-expired sessions (active and expiring)
  ListNonExpired,
  /// Get session by session ID
  Get { session_id: String },
}

#[derive(Subcommand)]
pub enum MigrateCommand {
  /// Create the account table in the database
  CreateAccountTable,
  /// Create the account_session table in the database
  CreateAccountSessionTable,
  /// Drop the account_session table from the database
  DropTableAccountSession,
  /// Drop all tables from the database
  DropAllTables,
}

pub async fn handle_command(
  command: Command, account_service: AccountService,
) -> Result<(), Box<dyn std::error::Error>> {
  match command {
    Command::Account { command } => handle_account_command(command, account_service).await,
    Command::Session { command } => handle_session_command(command, account_service).await,
    Command::Login { nickname, password } => login::execute(account_service, nickname, password).await,
    Command::Serve => {
      // Serve command is handled separately in main.rs since it needs the router
      unreachable!("Serve command should be handled in main.rs")
    },
    Command::Migrates { .. } => {
      // Migrates commands are handled directly in main.rs since they don't need AccountService
      unreachable!("Migrates commands should be handled in main.rs")
    },
  }
}

async fn handle_account_command(
  command: AccountCommand, account_service: AccountService,
) -> Result<(), Box<dyn std::error::Error>> {
  match command {
    AccountCommand::List => list::execute(account_service).await,
    AccountCommand::Create { nickname } => {
      let (uuid, password) = account_service.create_account(&nickname).await?;
      println!("uuid: {}, nickname: {}, password: {}", uuid, nickname, password);
      Ok(())
    },
    AccountCommand::Get { nickname } => {
      match account_service.get_account_by_nickname(&nickname).await? {
        Some(account) => {
          println!(
            "Account found: uuid={}, nickname={}",
            account.uuid(),
            account.nickname().as_str()
          );
        },
        None => {
          println!("Account not found with nickname: {}", nickname);
        },
      }
      Ok(())
    },
    AccountCommand::Delete { nickname } => {
      account_service.delete_account_by_nickname(&nickname).await?;
      println!("Deleted account with nickname: {}", nickname);
      Ok(())
    },
    AccountCommand::ResetPassword { uuid_or_nickname } => {
      reset_password::execute(account_service, uuid_or_nickname).await
    },
  }
}

async fn handle_session_command(
  command: SessionCommand, account_service: AccountService,
) -> Result<(), Box<dyn std::error::Error>> {
  match command {
    SessionCommand::List => list_sessions::execute(account_service).await,
    SessionCommand::ListNonExpired => session::list_non_expired(account_service).await,
    SessionCommand::Get { session_id } => session::get(account_service, session_id).await,
  }
}
