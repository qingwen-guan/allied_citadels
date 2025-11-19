use account_context::{AccountService, SessionId};
use chrono::Local;
use uuid::Uuid;

pub async fn execute(
  account_service: AccountService, session_id_str: String,
) -> Result<(), Box<dyn std::error::Error>> {
  let uuid = session_id_str
    .parse::<Uuid>()
    .map_err(|e| format!("Invalid session ID format: {}", e))?;
  let session_id = SessionId::from(uuid);

  match account_service.get_session(session_id).await? {
    Some(session) => {
      // Convert UTC timestamps to local time
      let created_local = session.created_at.with_timezone(&Local);
      let expires_local = session.expires_at.with_timezone(&Local);

      println!("Session ID: {}", session.session_id);
      println!("  AccountID: {}", session.account_id);
      println!("  Created: {}", created_local.format("%Y-%m-%d %H:%M:%S"));
      println!("  Expires: {}", expires_local.format("%Y-%m-%d %H:%M:%S"));
      println!("  Status: {}", session.status);
    },
    None => {
      println!("Session not found with ID: {}", session_id);
    },
  }

  Ok(())
}
