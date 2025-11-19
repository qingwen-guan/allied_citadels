use account_context::AccountService;

pub async fn execute(
  account_service: AccountService, uuid_or_nickname: String,
) -> Result<(), Box<dyn std::error::Error>> {
  let result = account_service.reset_password(&uuid_or_nickname).await?;
  println!(
    "uuid: {}, nickname: {:?}, new password: {}",
    result.uuid, result.nickname, result.password
  );
  Ok(())
}
