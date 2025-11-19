use account_context::UserService;
use chrono::Local;

pub async fn execute(user_service: UserService) -> Result<(), Box<dyn std::error::Error>> {
  let sessions = user_service.list_non_expired_sessions().await?;

  if sessions.is_empty() {
    println!("No non-expired sessions found.");
    return Ok(());
  }

  println!("Found {} non-expired session(s):", sessions.len());
  println!();

  for session in sessions {
    // Convert UTC timestamps to local time
    let created_local = session.created_at.with_timezone(&Local);
    let expires_local = session.expires_at.with_timezone(&Local);

    println!("Session ID: {}", session.session_id);
    println!("  UserID: {}", session.user_id);
    println!("  Created: {}", created_local.format("%Y-%m-%d %H:%M:%S"));
    println!("  Expires: {}", expires_local.format("%Y-%m-%d %H:%M:%S"));
    println!("  Status: {}", session.status);
    println!();
  }

  Ok(())
}
