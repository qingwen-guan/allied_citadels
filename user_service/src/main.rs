use clap::{Parser, Subcommand};
use user_service::PersonService;

#[derive(Parser)]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand)]
enum Command {
  List,
  Create { nickname: String },
  ResetPassword { nickname: String },
  Delete { nickname: String },
  Get { nickname: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let person_service = PersonService::new().await?;

  let cli = Cli::parse();
  match cli.command {
    Command::List => {
      let users = person_service.list_users().await?;
      println!("users: {:?}", users);
    },
    Command::Create { nickname } => {
      let password = person_service.create_user(&nickname).await?;
      println!("nickname: {}, password: {}", nickname, password);
    },
    Command::ResetPassword { nickname } => {
      let password = person_service.reset_password_by_name(&nickname).await?;
      println!("nickname: {}, new password: {}", nickname, password);
    },
    Command::Delete { nickname } => {
      person_service.delete_user_by_nickname(&nickname).await?;
      println!("Deleted user with nickname: {}", nickname);
    },
    Command::Get { nickname } => match person_service.get_user_by_nickname(&nickname).await? {
      Some(user) => {
        println!("User found: uuid={}, nickname={}", user.uuid(), user.nickname());
      },
      None => {
        println!("User not found with nickname: {}", nickname);
      },
    },
  }
  Ok(())
}
