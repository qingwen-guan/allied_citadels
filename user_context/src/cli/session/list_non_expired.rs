use chrono::Local;
use user_context::UserService;

pub async fn execute(user_service: UserService) -> Result<(), Box<dyn std::error::Error>> {
  let sessions = user_service.list_non_expired_sessions().await?;

  if sessions.is_empty() {
    println!("No non-expired sessions found.");
    return Ok(());
  }

  println!("Found {} non-expired session(s):", sessions.len());
  println!();

  for session in sessions {
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
    println!();
  }

  Ok(())
}
