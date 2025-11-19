use user_context::Config;

use super::MigrateCommand;

pub async fn execute(command: MigrateCommand) -> Result<(), Box<dyn std::error::Error>> {
  let config = Config::load()?;
  match command {
    MigrateCommand::CreateUserTable => {
      user_context::create_user_table(&config.dsn).await?;
    },
    MigrateCommand::CreateUserSessionTable => {
      user_context::create_user_session_table(&config.dsn).await?;
    },
    MigrateCommand::DropTableUserSession => {
      user_context::drop_table_user_session(&config.dsn).await?;
    },
    MigrateCommand::DropAllTables => {
      common_context::drop_all_tables(&config.dsn).await?;
    },
    MigrateCommand::CreateAllTables => {
      // TODO: create fn user_context:create_all_tables()
      user_context::create_user_table(&config.dsn).await?;
      user_context::create_user_session_table(&config.dsn).await?;
      println!("All tables created successfully!");
    },
  }
  Ok(())
}
