use account_context::UserService;
use uuid::Uuid;

pub async fn execute(user_service: UserService, uuid_or_nickname: String) -> Result<(), Box<dyn std::error::Error>> {
  // Try to parse as UUID first
  if let Ok(uuid) = uuid_or_nickname.parse::<Uuid>() {
    let password = user_service.reset_password_by_uuid(uuid).await?;
    println!("uuid: {}, new password: {}", uuid, password);
  } else {
    // If not a valid UUID, treat it as a nickname
    let (uuid, password) = user_service.reset_password_by_name(&uuid_or_nickname).await?;
    println!(
      "uuid: {}, nickname: {}, new password: {}",
      uuid, uuid_or_nickname, password
    );
  }
  Ok(())
}
