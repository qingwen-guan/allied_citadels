use chrono::Local;
use std::collections::HashMap;

use user_context::services::UserService;

struct SessionWithExpiry {
  session_id: String,
  expires_at: String,
}

pub async fn execute(user_service: UserService) -> Result<(), Box<dyn std::error::Error>> {
  // Fetch all users
  let users = user_service.list_users().await?;

  // Fetch all active sessions and build a map from user_id -> Vec<SessionWithExpiry>
  let sessions = user_service.list_active_sessions().await?;
  let mut user_to_sessions: HashMap<String, Vec<SessionWithExpiry>> = HashMap::new();
  for session in sessions {
    // Parse and format expires_at timestamp
    let expires_utc = chrono::DateTime::parse_from_rfc3339(&session.expires_at)
      .map_err(|e| format!("Failed to parse expires_at: {}", e))?;
    let expires_local = expires_utc.with_timezone(&Local);
    let expires_formatted = expires_local.format("%Y-%m-%d %H:%M:%S").to_string();

    user_to_sessions
      .entry(session.user_id.clone())
      .or_default()
      .push(SessionWithExpiry {
        session_id: session.session_id,
        expires_at: expires_formatted,
      });
  }

  // Print users, including active session_id(s) and expire time if any exist
  for user in users {
    if let Some(sessions) = user_to_sessions.get(&user.user_id) {
      match sessions.as_slice() {
        [] => {
          println!("user_id: {}, nickname: {}", user.user_id, user.nickname);
        },
        [session] => {
          println!(
            "user_id: {}, nickname: {}, session_id: {}, expires_at: {}",
            user.user_id, user.nickname, session.session_id, session.expires_at
          );
        },
        many => {
          let session_parts: Vec<String> = many
            .iter()
            .map(|s| format!("{} (expires: {})", s.session_id, s.expires_at))
            .collect();
          let joined = session_parts.join(", ");
          println!(
            "user_id: {}, nickname: {}, session_ids: [{}]",
            user.user_id, user.nickname, joined
          );
        },
      }
    } else {
      println!("user_id: {}, nickname: {}", user.user_id, user.nickname);
    }
  }

  Ok(())
}
