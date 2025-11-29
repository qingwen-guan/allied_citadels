use user_context::services::UserService;

pub async fn execute(user_service: UserService, uuid_or_nickname: String) -> Result<(), Box<dyn std::error::Error>> {
  // Try to parse as UUID first
  if uuid_or_nickname.parse::<uuid::Uuid>().is_ok() {
    let password = user_service.reset_password_by_id(&uuid_or_nickname).await?;
    println!("user_id: {}, new password: {}", uuid_or_nickname, password);
  } else {
    // If not a valid UUID, treat it as a nickname
    let result = user_service.reset_password_by_name(&uuid_or_nickname).await?;
    println!(
      "user_id: {}, nickname: {}, new password: {}",
      result.user_id, uuid_or_nickname, result.password
    );
  }
  Ok(())
}
