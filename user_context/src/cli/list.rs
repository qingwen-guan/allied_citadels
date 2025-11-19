use user_context::UserService;

pub async fn execute(user_service: UserService) -> Result<(), Box<dyn std::error::Error>> {
  let users = user_service.list_users().await?;
  for user in users {
    println!(
      "uuid: {}, nickname: {}, salted_password: {:?}",
      user.uuid(),
      user.nickname(),
      user.salted_password(),
    );
  }
  Ok(())
}
