use std::process::Command;

use crate::config::Config;
use crate::docker;
use crate::error::{DockerError, PostgresError};

const TARGET_DATABASE: &str = "allied_citadels"; // TODO: move to config

pub fn create_database(config: &Config) -> Result<(), PostgresError> {
  println!("Creating {} database...", TARGET_DATABASE);

  docker::ensure_container_ready(config)?;

  // Check if database already exists
  println!("Checking if database already exists...");
  let check_query = format!("SELECT 1 FROM pg_database WHERE datname='{}'", TARGET_DATABASE);
  let check_cmd: Vec<String> = vec![
    "docker".to_string(),
    "exec".to_string(),
    config.docker.container_name.clone(),
    "psql".to_string(),
    "-U".to_string(),
    config.postgres.user.clone(),
    "-tAc".to_string(),
    check_query,
  ];

  let check_cmd_refs: Vec<&str> = check_cmd.iter().map(|s| s.as_str()).collect();
  let result = run_command(&check_cmd_refs, false)?;

  if result.status.success() {
    let stdout = String::from_utf8_lossy(&result.stdout);
    if stdout.trim().contains("1") {
      println!("Database '{}' already exists.", TARGET_DATABASE);
      println!("Skipping creation.");
      return Ok(());
    }
  }

  // Create the database
  println!("Creating database {}...", TARGET_DATABASE);

  // Use embedded SQL file
  const CREATE_DATABASE_SQL: &str = include_str!("../sql/create_database.sql");

  let create_cmd: Vec<String> = vec![
    "docker".to_string(),
    "exec".to_string(),
    config.docker.container_name.clone(),
    "psql".to_string(),
    "-U".to_string(),
    config.postgres.user.clone(),
    "-d".to_string(),
    "postgres".to_string(),
    "-c".to_string(),
    CREATE_DATABASE_SQL.trim().to_string(),
  ];

  let create_cmd_refs: Vec<&str> = create_cmd.iter().map(|s| s.as_str()).collect();
  let result = run_command(&create_cmd_refs, false)?;

  if !result.status.success() {
    let error_msg = format!(
      "Failed to create database.\nError: {}",
      String::from_utf8_lossy(&result.stderr)
    );
    return Err(DockerError::Other(error_msg).into());
  }

  println!();
  println!("{}", "=".repeat(40));
  println!("Database ready!");
  println!("{}", "=".repeat(40));
  println!("Database name: {}", TARGET_DATABASE);
  println!();
  println!("You can now run 'user_context create-account-table' to create the account table.");
  println!();

  Ok(())
}

fn run_command(cmd: &[&str], check: bool) -> Result<std::process::Output, PostgresError> {
  let cmd_str = cmd.join(" ");
  println!("[EXEC] {}", cmd_str);

  let output = Command::new(cmd[0])
    .args(&cmd[1..])
    .output()
    .map_err(|e| DockerError::CommandFailed(format!("{}: {}", cmd_str, e)))?;

  if check && !output.status.success() {
    let error_msg = format!(
      "Command failed: {}\nError: {}",
      cmd_str,
      String::from_utf8_lossy(&output.stderr)
    );
    return Err(DockerError::CommandFailed(error_msg).into());
  }

  Ok(output)
}
