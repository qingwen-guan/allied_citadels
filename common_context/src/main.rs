mod cli;

use clap::Parser;

const DEFAULT_CONFIG_FILE_NAME: &str = "default_common_config.toml";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let cli = cli::Cli::parse();

  match cli.command {
    cli::Command::Migrates { command } => match command {
      cli::MigrateCommand::DropAllTables => {
        let config_path =
          common_context::domain::factories::DbConfigFactory::find_config_file(DEFAULT_CONFIG_FILE_NAME)
            .ok_or_else(|| format!("common_context/config/{} not found", DEFAULT_CONFIG_FILE_NAME))?;
        let config = common_context::domain::factories::DbConfigFactory::new(config_path).load()?;
        common_context::drop_all_tables(&config.dsn).await?;
      },
    },
  }

  Ok(())
}
