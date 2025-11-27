use user_context::UserService;

pub async fn execute(user_service: UserService) -> Result<(), Box<dyn std::error::Error>> {
  let users = user_service.list_users().await?;
  for user in users {
    println!("user_id: {}, nickname: {}", user.user_id, user.nickname,);
  }
  Ok(())
}
