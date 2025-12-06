use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
  /// Database migration commands
  Migrates {
    #[command(subcommand)]
    command: MigrateCommand,
  },
}

#[derive(Subcommand)]
pub enum MigrateCommand {
  /// Drop all tables from the database
  DropAllTables,
}
