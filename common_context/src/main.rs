mod cli;

use clap::Parser;

use common_context::domain::factories::DbConfigFactory;
use common_context::PACKAGE_DIR;

const DEFAULT_CONFIG_FILE_NAME: &str = "default_common_config.toml";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let cli = cli::Cli::parse();

  match cli.command {
    cli::Command::Migrates { command } => match command {
      cli::MigrateCommand::DropAllTables => {
        let config_path =
          DbConfigFactory::find_config_file(PACKAGE_DIR, DEFAULT_CONFIG_FILE_NAME)
            .ok_or_else(|| format!("{}/config/{} not found", PACKAGE_DIR, DEFAULT_CONFIG_FILE_NAME))?;
        let config = DbConfigFactory::new(config_path).load()?;
        common_context::drop_all_tables(&config.dsn).await?;
      },
    },
  }

  Ok(())
}
