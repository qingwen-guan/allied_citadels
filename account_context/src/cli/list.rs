use account_context::AccountService;

pub async fn execute(account_service: AccountService) -> Result<(), Box<dyn std::error::Error>> {
  let accounts = account_service.list_accounts().await?;
  for account in accounts {
    println!(
      "uuid: {}, nickname: {}, salted_password: {:?}",
      account.uuid(),
      account.nickname(),
      account.salted_password(),
    );
  }
  Ok(())
}
