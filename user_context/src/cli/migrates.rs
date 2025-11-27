use super::MigrateCommand;

pub async fn execute(dsn: &str, command: MigrateCommand) -> Result<(), Box<dyn std::error::Error>> {
  match command {
    MigrateCommand::CreateUserTable => {
      user_context::migrations::create_user_table(dsn).await?;
    },
    MigrateCommand::CreateUserSessionTable => {
      user_context::migrations::create_user_session_table(dsn).await?;
    },
    MigrateCommand::DropTableUserSession => {
      user_context::migrations::drop_table_user_session(dsn).await?;
    },
    MigrateCommand::DropAllTables => {
      common_context::drop_all_tables(dsn).await?;
    },
    MigrateCommand::CreateAllTables => {
      user_context::migrations::create_all_tables(dsn).await?;
    },
  }
  Ok(())
}
