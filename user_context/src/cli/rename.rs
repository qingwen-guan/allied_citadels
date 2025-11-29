use user_context::services::UserService;

pub async fn execute(
  user_service: UserService, uuid_or_nickname: String, new_nickname: String,
) -> Result<(), Box<dyn std::error::Error>> {
  let result = user_service.rename_user(&uuid_or_nickname, &new_nickname).await?;

  match result {
    Some(info) => {
      println!(
        "Updated user (user_id: {}, old nickname: {}) nickname to: {}",
        info.user_id, info.old_nickname, info.new_nickname
      );
    },
    None => {
      println!("User not found for identifier: {}", uuid_or_nickname);
    },
  }

  Ok(())
}
