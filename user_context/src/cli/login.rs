use user_context::UserService;

pub async fn execute(
  user_service: UserService, nickname: String, password: String,
) -> Result<(), Box<dyn std::error::Error>> {
  let (session_id, _user_id) = user_service.login(&nickname, &password).await?;

  // Get user info for display
  let user = user_service.get_user_by_nickname(&nickname).await?;
  match user {
    Some(user) => {
      println!(
        "Login successful! uuid: {}, nickname: {}, session_id: {}",
        // TODO: print user_id here
        user.uuid(),
        user.nickname().as_str(),
        session_id
      );
    },
    None => {
      return Err("User not found after successful login - this should not happen".into());
    },
  }
  Ok(())
}
