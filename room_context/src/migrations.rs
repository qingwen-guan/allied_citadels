use sqlx::postgres::PgPoolOptions;

use crate::config::Config;
use crate::error::RoomError;

pub async fn create_room_table(config: &Config) -> Result<(), RoomError> {
  println!("Creating room table...");

  let pool = PgPoolOptions::new()
    .max_connections(1)
    .connect(&config.dsn)
    .await
    .map_err(RoomError::Database)?;

  // Check if table exists
  let table_exists: bool = sqlx::query_scalar(
    r#"
    SELECT EXISTS (
      SELECT FROM information_schema.tables 
      WHERE table_schema = 'public' 
      AND table_name = 'room'
    )
    "#,
  )
  .fetch_one(&pool)
  .await
  .map_err(RoomError::Database)?;

  if table_exists {
    println!("Table room already exists.");
    println!();
    return Ok(());
  }

  sqlx::query(
    r#"
    CREATE TABLE IF NOT EXISTS room (
        id UUID PRIMARY KEY,
        room_number INTEGER UNIQUE NOT NULL,
        room_name VARCHAR(255) UNIQUE NOT NULL,
        creator UUID NOT NULL,
        max_players INTEGER NOT NULL CHECK (max_players IN (4, 6)),
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
        expires_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW() + INTERVAL '1 hour'
    )
    "#,
  )
  .execute(&pool)
  .await
  .map_err(RoomError::Database)?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_room_name ON room(room_name)")
    .execute(&pool)
    .await
    .map_err(RoomError::Database)?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_room_creator ON room(creator)")
    .execute(&pool)
    .await
    .map_err(RoomError::Database)?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_room_number ON room(room_number)")
    .execute(&pool)
    .await
    .map_err(RoomError::Database)?;

  println!();
  println!("{}", "=".repeat(40));
  println!("Room table created successfully!");
  println!("{}", "=".repeat(40));
  println!("Table: room");
  println!("Columns: id, room_number, room_name, creator, max_players, created_at, expires_at");
  println!();

  Ok(())
}

pub async fn create_room_participant_table(config: &Config) -> Result<(), RoomError> {
  println!("Creating room_participant table...");

  let pool = PgPoolOptions::new()
    .max_connections(1)
    .connect(&config.dsn)
    .await
    .map_err(RoomError::Database)?;

  // Check if table exists
  let table_exists: bool = sqlx::query_scalar(
    r#"
    SELECT EXISTS (
      SELECT FROM information_schema.tables 
      WHERE table_schema = 'public' 
      AND table_name = 'room_participant'
    )
    "#,
  )
  .fetch_one(&pool)
  .await
  .map_err(RoomError::Database)?;

  if table_exists {
    println!("Table room_participant already exists.");
    println!();
    return Ok(());
  }

  sqlx::query(
    r#"
    CREATE TABLE IF NOT EXISTS room_participant (
        room_id UUID NOT NULL REFERENCES room(id) ON DELETE CASCADE,
        user_id UUID NOT NULL,
        seat_number SMALLINT CHECK (seat_number IS NULL OR (seat_number >= 0 AND seat_number <= 5)),
        viewing_seat_number SMALLINT CHECK (viewing_seat_number IS NULL OR (viewing_seat_number >= 0 AND viewing_seat_number <= 5)),
        joined_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
        PRIMARY KEY (room_id, user_id),
        UNIQUE (room_id, seat_number) WHERE seat_number IS NOT NULL
    )
    "#,
  )
  .execute(&pool)
  .await
  .map_err(RoomError::Database)?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_room_participant_room_id ON room_participant(room_id)")
    .execute(&pool)
    .await
    .map_err(RoomError::Database)?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_room_participant_user_id ON room_participant(user_id)")
    .execute(&pool)
    .await
    .map_err(RoomError::Database)?;

  sqlx::query(
    "CREATE INDEX IF NOT EXISTS idx_room_participant_seat_number ON room_participant(room_id, seat_number)",
  )
  .execute(&pool)
  .await
  .map_err(RoomError::Database)?;

  println!();
  println!("{}", "=".repeat(40));
  println!("Room participant table created successfully!");
  println!("{}", "=".repeat(40));
  println!("Table: room_participant");
  println!("Columns: room_id, user_id, seat_number, joined_at");
  println!();

  Ok(())
}

pub async fn create_room_to_user_message_table(config: &Config) -> Result<(), RoomError> {
  println!("Creating room_to_user_message table...");

  let pool = PgPoolOptions::new()
    .max_connections(1)
    .connect(&config.dsn)
    .await
    .map_err(RoomError::Database)?;

  // Check if table exists
  let table_exists: bool = sqlx::query_scalar(
    r#"
    SELECT EXISTS (
      SELECT FROM information_schema.tables 
      WHERE table_schema = 'public' 
      AND table_name = 'room_to_user_message'
    )
    "#,
  )
  .fetch_one(&pool)
  .await
  .map_err(RoomError::Database)?;

  if table_exists {
    println!("Table room_to_user_message already exists.");
    println!();
    return Ok(());
  }

  sqlx::query(
    r#"
    CREATE TABLE IF NOT EXISTS room_to_user_message (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        room_id UUID NOT NULL REFERENCES room(id) ON DELETE CASCADE,
        user_id UUID NOT NULL,
        topic VARCHAR(255) NOT NULL,
        content TEXT NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
        read_at TIMESTAMP WITH TIME ZONE
    )
    "#,
  )
  .execute(&pool)
  .await
  .map_err(RoomError::Database)?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_room_to_user_message_room_id ON room_to_user_message(room_id)")
    .execute(&pool)
    .await
    .map_err(RoomError::Database)?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_room_to_user_message_user_id ON room_to_user_message(user_id)")
    .execute(&pool)
    .await
    .map_err(RoomError::Database)?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_room_to_user_message_topic ON room_to_user_message(topic)")
    .execute(&pool)
    .await
    .map_err(RoomError::Database)?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_room_to_user_message_created_at ON room_to_user_message(created_at)")
    .execute(&pool)
    .await
    .map_err(RoomError::Database)?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_room_to_user_message_read_at ON room_to_user_message(read_at)")
    .execute(&pool)
    .await
    .map_err(RoomError::Database)?;

  println!();
  println!("{}", "=".repeat(40));
  println!("Room to user message table created successfully!");
  println!("{}", "=".repeat(40));
  println!("Table: room_to_user_message");
  println!("Columns: id, room_id, user_id, topic, content, created_at, read_at");
  println!();

  Ok(())
}

pub async fn create_user_to_room_message_table(config: &Config) -> Result<(), RoomError> {
  println!("Creating user_to_room_message table...");

  let pool = PgPoolOptions::new()
    .max_connections(1)
    .connect(&config.dsn)
    .await
    .map_err(RoomError::Database)?;

  // Check if table exists
  let table_exists: bool = sqlx::query_scalar(
    r#"
    SELECT EXISTS (
      SELECT FROM information_schema.tables 
      WHERE table_schema = 'public' 
      AND table_name = 'user_to_room_message'
    )
    "#,
  )
  .fetch_one(&pool)
  .await
  .map_err(RoomError::Database)?;

  if table_exists {
    println!("Table user_to_room_message already exists.");
    println!();
    return Ok(());
  }

  sqlx::query(
    r#"
    CREATE TABLE IF NOT EXISTS user_to_room_message (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        room_id UUID NOT NULL REFERENCES room(id) ON DELETE CASCADE,
        user_id UUID NOT NULL,
        topic VARCHAR(255) NOT NULL,
        content TEXT NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
        read_at TIMESTAMP WITH TIME ZONE
    )
    "#,
  )
  .execute(&pool)
  .await
  .map_err(RoomError::Database)?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_to_room_message_room_id ON user_to_room_message(room_id)")
    .execute(&pool)
    .await
    .map_err(RoomError::Database)?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_to_room_message_user_id ON user_to_room_message(user_id)")
    .execute(&pool)
    .await
    .map_err(RoomError::Database)?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_to_room_message_topic ON user_to_room_message(topic)")
    .execute(&pool)
    .await
    .map_err(RoomError::Database)?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_to_room_message_created_at ON user_to_room_message(created_at)")
    .execute(&pool)
    .await
    .map_err(RoomError::Database)?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_to_room_message_read_at ON user_to_room_message(read_at)")
    .execute(&pool)
    .await
    .map_err(RoomError::Database)?;

  println!();
  println!("{}", "=".repeat(40));
  println!("User to room message table created successfully!");
  println!("{}", "=".repeat(40));
  println!("Table: user_to_room_message");
  println!("Columns: id, room_id, user_id, topic, content, created_at, read_at");
  println!();

  Ok(())
}

pub async fn drop_room_table(config: &Config) -> Result<(), RoomError> {
  println!("Dropping room table...");

  let pool = PgPoolOptions::new()
    .max_connections(1)
    .connect(&config.dsn)
    .await
    .map_err(RoomError::Database)?;

  // Check if table exists
  let table_exists: bool = sqlx::query_scalar(
    r#"
    SELECT EXISTS (
      SELECT FROM information_schema.tables 
      WHERE table_schema = 'public' 
      AND table_name = 'room'
    )
    "#,
  )
  .fetch_one(&pool)
  .await
  .map_err(RoomError::Database)?;

  if !table_exists {
    println!("Table room does not exist.");
    return Ok(());
  }

  // Drop the table with CASCADE to handle foreign key constraints
  sqlx::query("DROP TABLE IF EXISTS room CASCADE")
    .execute(&pool)
    .await
    .map_err(RoomError::Database)?;

  println!();
  println!("{}", "=".repeat(40));
  println!("Room table dropped successfully!");
  println!("{}", "=".repeat(40));
  println!("Dropped table: room");
  println!();

  Ok(())
}
