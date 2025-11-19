mod config;
mod docker;
mod error;
mod postgres;

use std::io;

use clap::{Parser, Subcommand};
use error::PostgresError;

#[derive(Parser)]
#[command(name = "postgres_proxy")]
#[command(about = "PostgreSQL Docker management tool")]
#[command(version = "0.1.0")]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  /// Initialize/start PostgreSQL in Docker
  Init,
  /// Clean up PostgreSQL container and volume
  Clean,
  /// Create the allied_citadels database
  CreateDb,
}

fn main() {
  let cli = Cli::parse();

  match run_command(cli.command) {
    Ok(()) => {
      // On Windows, pause at the end (like the .bat file)
      #[cfg(windows)]
      {
        println!("\nPress Enter to continue...");
        let _ = io::stdin().read_line(&mut String::new());
      }
    },
    Err(e) => {
      eprintln!("ERROR: {}", e);
      std::process::exit(1);
    },
  }
}

fn run_command(command: Commands) -> Result<(), PostgresError> {
  let config = config::Config::load()?;

  match command {
    Commands::Init => docker::init_postgres(&config),
    Commands::Clean => docker::clean_postgres(&config),
    Commands::CreateDb => postgres::create_database(&config),
  }
}
