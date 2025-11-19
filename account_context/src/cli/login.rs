use account_context::AccountService;

pub async fn execute(
  account_service: AccountService, nickname: String, password: String,
) -> Result<(), Box<dyn std::error::Error>> {
  let session_id = account_service.login(&nickname, &password).await?;

  // Get account info for display
  let account = account_service.get_account_by_nickname(&nickname).await?;
  if let Some(account) = account {
    println!(
      "Login successful! uuid: {}, nickname: {}, session_id: {}",
      account.uuid(),
      account.nickname().as_str(),
      session_id
    );
  } else {
    println!("Login successful! session_id: {}", session_id);
  }
  Ok(())
}
