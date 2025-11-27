use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::error::{DockerError, PostgresError};

// Internal PostgreSQL container port (always 5432)
const POSTGRES_CONTAINER_PORT: u16 = 5432;

pub fn check_docker_running() -> Result<(), PostgresError> {
  let output = Command::new("docker")
    .arg("info")
    .output()
    .map_err(|e| DockerError::Other(format!("Failed to execute docker info: {}", e)))?;

  if !output.status.success() {
    return Err(DockerError::NotRunning.into());
  }

  Ok(())
}

pub fn container_exists(container_name: &str) -> bool {
  Command::new("docker")
    .arg("inspect")
    .arg(container_name)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .map(|s| s.success())
    .unwrap_or(false)
}

pub fn container_is_running(container_name: &str) -> bool {
  let output = Command::new("docker")
    .arg("ps")
    .arg("--filter")
    .arg(format!("name={}", container_name))
    .arg("--format")
    .arg("{{.Names}}")
    .output()
    .ok();

  if let Some(output) = output
    && output.status.success()
  {
    let stdout = String::from_utf8_lossy(&output.stdout);
    return stdout.contains(container_name);
  }

  false
}

pub fn volume_exists(volume_name: &str) -> bool {
  Command::new("docker")
    .arg("volume")
    .arg("inspect")
    .arg(volume_name)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .map(|s| s.success())
    .unwrap_or(false)
}

pub fn create_volume(volume_name: &str) -> Result<(), PostgresError> {
  if !volume_exists(volume_name) {
    println!("Creating persistent volume: {}", volume_name);
    run_command(&["docker", "volume", "create", volume_name], true)?;
  } else {
    println!("Volume {} already exists.", volume_name);
  }
  Ok(())
}

pub fn create_container(config: &Config) -> Result<(), PostgresError> {
  println!("Creating new PostgreSQL container...");

  create_volume(&config.docker.volume_name)?;

  let user_env = format!("POSTGRES_USER={}", config.postgres.user);
  let password_env = format!("POSTGRES_PASSWORD={}", config.postgres.password);
  let timezone_env = format!("TZ={}", config.docker.timezone);
  let port_mapping = format!("{}:{}", config.postgres.port, POSTGRES_CONTAINER_PORT);
  let volume_mapping = format!("{}:/var/lib/postgresql/data", config.docker.volume_name);
  let image = format!("postgres:{}", config.postgres.version);

  let docker_cmd = vec![
    "docker",
    "run",
    "-d",
    "--name",
    &config.docker.container_name,
    "-e",
    &user_env,
    "-e",
    &password_env,
    "-e",
    &timezone_env,
    "-p",
    &port_mapping,
    "-v",
    &volume_mapping,
    &image,
  ];

  let result = run_command(&docker_cmd, false)?;
  if !result.status.success() {
    let error_msg = format!(
      "Failed to start PostgreSQL container.\nError: {}",
      String::from_utf8_lossy(&result.stderr)
    );
    return Err(DockerError::Other(error_msg).into());
  }

  println!("\nPostgreSQL container created and started successfully!");
  println!("Waiting for PostgreSQL to be ready...");
  thread::sleep(Duration::from_secs(config.timing.initial_wait_seconds));

  Ok(())
}

pub fn start_existing_container(container_name: &str, config: &Config) -> Result<(), PostgresError> {
  if !container_is_running(container_name) {
    println!("Starting existing container...");
    let result = run_command(&["docker", "start", container_name], false)?;
    if !result.status.success() {
      return Err(DockerError::Other("Failed to start container.".to_string()).into());
    }
    println!("Container started successfully!");
    println!("Waiting for PostgreSQL to be ready...");
    thread::sleep(Duration::from_secs(config.timing.container_start_wait_seconds));
  } else {
    println!("Container is already running.");
  }
  Ok(())
}

pub fn wait_for_postgres(config: &Config) -> Result<(), PostgresError> {
  println!("\nVerifying PostgreSQL is ready...");

  for retry_count in 0..config.timing.max_postgres_retries {
    let cmd = vec![
      "docker",
      "exec",
      &config.docker.container_name,
      "pg_isready",
      "-U",
      &config.postgres.user,
    ];

    let result = run_command(&cmd, false)?;

    if result.status.success() {
      println!("PostgreSQL is ready and accepting connections!");
      return Ok(());
    }

    if retry_count == 0 {
      println!("Waiting for PostgreSQL to be ready...");
    }

    thread::sleep(Duration::from_secs(config.timing.retry_delay_seconds));
  }

  Err(
    DockerError::Other(format!(
      "PostgreSQL failed to become ready after {} attempts.\n\
         Container may be starting up. Please wait and try again.\n\
         You can check container logs with: docker logs {}",
      config.timing.max_postgres_retries, config.docker.container_name
    ))
    .into(),
  )
}

pub fn clean_postgres(config: &Config) -> Result<(), PostgresError> {
  println!("Cleaning up PostgreSQL Docker container and volume...");

  check_docker_running()?;

  // Stop container if it's running
  if container_is_running(&config.docker.container_name) {
    println!("Stopping container {}...", config.docker.container_name);
    let result = run_command(
      &["docker", "stop", "--time", "10", &config.docker.container_name],
      false,
    )?;
    if result.status.success() {
      println!("Container stopped successfully.");
    } else {
      println!("Failed to stop container, attempting force remove...");
      run_command(&["docker", "rm", "-f", &config.docker.container_name], false)?;
    }
  } else {
    println!("Container {} is not running.", config.docker.container_name);
  }

  // Remove container
  if container_exists(&config.docker.container_name) {
    println!("Removing container {}...", config.docker.container_name);
    let result = run_command(&["docker", "rm", "-f", &config.docker.container_name], false)?;
    if result.status.success() {
      println!("Container removed successfully.");
    } else {
      println!("Failed to remove container.");
    }
  } else {
    println!("Container {} does not exist.", config.docker.container_name);
  }

  // Remove volume
  if volume_exists(&config.docker.volume_name) {
    println!("Removing volume {}...", config.docker.volume_name);
    let result = run_command(&["docker", "volume", "rm", &config.docker.volume_name], false)?;
    if result.status.success() {
      println!("Volume removed successfully.");
    } else {
      println!("Failed to remove volume. It may be in use.");
    }
  } else {
    println!("Volume {} does not exist.", config.docker.volume_name);
  }

  println!("\n{}", "=".repeat(40));
  println!("Cleanup complete!");
  println!("{}", "=".repeat(40));
  println!("You can now run 'postgres_proxy init' to create a fresh PostgreSQL container.");
  println!();

  Ok(())
}

pub fn ensure_container_ready(config: &Config) -> Result<(), PostgresError> {
  check_docker_running()?;

  if !container_exists(&config.docker.container_name) {
    return Err(
      DockerError::Other(format!(
        "Container '{}' not found.\n\
             Please run 'postgres_proxy init' first to create the container.",
        config.docker.container_name
      ))
      .into(),
    );
  }

  if !container_is_running(&config.docker.container_name) {
    println!("Container exists but is not running. Starting it...");
    start_existing_container(&config.docker.container_name, config)?;
  }

  wait_for_postgres(config)
}

pub fn print_connection_info(config: &Config) {
  println!("\n{}", "=".repeat(40));
  println!("PostgreSQL is running!");
  println!("{}", "=".repeat(40));
  println!("Container Name: {}", config.docker.container_name);
  println!("Host: {}", config.postgres.host);
  println!("Port: {}", config.postgres.port);
  println!("Username: {}", config.postgres.user);
  println!("Password: {}", config.postgres.password);
  println!();
  println!("Connection string:");
  println!();
  println!("Data is persisted in Docker volume: {}", config.docker.volume_name);
  println!();
  println!("To stop the container: docker stop {}", config.docker.container_name);
  println!("To remove the container: docker rm {}", config.docker.container_name);
  println!(
    "To remove the volume (deletes all data): docker volume rm {}",
    config.docker.volume_name
  );
  println!();
}

pub fn init_postgres(config: &Config) -> Result<(), PostgresError> {
  println!("Starting PostgreSQL in Docker with persistent data...");

  check_docker_running()?;

  if !container_exists(&config.docker.container_name) {
    create_container(config)?;
  } else {
    println!("Container {} already exists.", config.docker.container_name);
    println!();

    start_existing_container(&config.docker.container_name, config)?;
  }

  wait_for_postgres(config)?;
  print_connection_info(config);

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
