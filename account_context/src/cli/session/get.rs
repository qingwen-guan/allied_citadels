use account_context::{SessionId, UserService};
use chrono::Local;
use uuid::Uuid;

pub async fn execute(user_service: UserService, session_id_str: String) -> Result<(), Box<dyn std::error::Error>> {
  let uuid = session_id_str
    .parse::<Uuid>()
    .map_err(|e| format!("Invalid session ID format: {}", e))?;
  let session_id = SessionId::from(uuid);

  match user_service.get_session(session_id).await? {
    Some(session) => {
      // Convert UTC timestamps to local time
      let created_local = session.created_at.with_timezone(&Local);
      let expires_local = session.expires_at.with_timezone(&Local);

      println!("Session ID: {}", session.session_id);
      println!("  UserID: {}", session.user_id);
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
