use chrono::Local;
use user_context::UserService;

pub async fn execute(user_service: UserService, session_id_str: String) -> Result<(), Box<dyn std::error::Error>> {
  match user_service.get_session(&session_id_str).await? {
    Some(session) => {
      // Parse RFC3339 timestamps and convert to local time
      let created_utc = chrono::DateTime::parse_from_rfc3339(&session.created_at)
        .map_err(|e| format!("Failed to parse created_at: {}", e))?;
      let expires_utc = chrono::DateTime::parse_from_rfc3339(&session.expires_at)
        .map_err(|e| format!("Failed to parse expires_at: {}", e))?;
      let created_local = created_utc.with_timezone(&Local);
      let expires_local = expires_utc.with_timezone(&Local);

      println!("Session ID: {}", session.session_id);
      println!("  UserID: {}", session.user_id);
      println!("  Created: {}", created_local.format("%Y-%m-%d %H:%M:%S"));
      println!("  Expires: {}", expires_local.format("%Y-%m-%d %H:%M:%S"));
      println!("  Status: {}", session.status);
    },
    None => {
      println!("Session not found with ID: {}", session_id_str);
    },
  }

  Ok(())
}
