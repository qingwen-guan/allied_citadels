use std::collections::HashMap;

use user_context::UserService;

pub async fn execute(user_service: UserService) -> Result<(), Box<dyn std::error::Error>> {
  // Fetch all users
  let users = user_service.list_users().await?;

  // Fetch all active sessions and build a map from user_id -> Vec<session_id>
  let sessions = user_service.list_active_sessions().await?;
  let mut user_to_sessions: HashMap<String, Vec<String>> = HashMap::new();
  for session in sessions {
    user_to_sessions
      .entry(session.user_id.clone())
      .or_default()
      .push(session.session_id);
  }

  // Print users, including active session_id(s) if any exist
  for user in users {
    if let Some(session_ids) = user_to_sessions.get(&user.user_id) {
      match session_ids.as_slice() {
        [] => {
          println!("user_id: {}, nickname: {}", user.user_id, user.nickname);
        },
        [only] => {
          println!(
            "user_id: {}, nickname: {}, session_id: {}",
            user.user_id, user.nickname, only
          );
        },
        many => {
          let joined = many.join(", ");
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
