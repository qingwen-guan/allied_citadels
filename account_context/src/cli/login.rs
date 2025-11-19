use account_context::UserService;

pub async fn execute(
  user_service: UserService, nickname: String, password: String,
) -> Result<(), Box<dyn std::error::Error>> {
  let (session_id, _user_id) = user_service.login(&nickname, &password).await?;

  // Get user info for display
  let user = user_service.get_user_by_nickname(&nickname).await?;
  if let Some(user) = user {
    // TODO: print user id
    println!(
      "Login successful! uuid: {}, nickname: {}, session_id: {}",
      user.uuid(),
      user.nickname().as_str(),
      session_id
    );
  } else {
    // TODO: error
    println!("Login successful! session_id: {}", session_id);
  }
  Ok(())
}
