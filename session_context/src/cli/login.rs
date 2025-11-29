use user_context::UserService;

pub async fn execute(
  user_service: UserService, nickname: String, password: String,
) -> Result<(), Box<dyn std::error::Error>> {
  let login_result = user_service.login(&nickname, &password).await?;

  // Get user info for display
  let user = user_service.get_user_by_nickname(&nickname).await?;
  println!(
    "Login successful! user_id: {}, nickname: {}, session_id: {}",
    login_result.user_id, user.nickname, login_result.session_id
  );
  Ok(())
}
