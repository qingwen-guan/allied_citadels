use sqlx::postgres::PgPoolOptions;

use crate::error::UserError;

pub async fn create_user_table(dsn: &str) -> Result<(), UserError> {
  println!("Creating user table...");

  let pool = PgPoolOptions::new()
    .max_connections(1)
    .connect(dsn)
    .await
    .map_err(UserError::Database)?;

  // Check if table exists
  let table_exists: bool = sqlx::query_scalar(
    r#"
    SELECT EXISTS (
      SELECT FROM information_schema.tables 
      WHERE table_schema = 'public' 
      AND table_name = 'user'
    )
    "#,
  )
  .fetch_one(&pool)
  .await
  .map_err(UserError::Database)?;

  if table_exists {
    println!("Table user already exists.");
    println!();
    return Ok(());
  }

  sqlx::query(
    r#"
    CREATE TABLE IF NOT EXISTS "user" (
        uuid UUID PRIMARY KEY,
        nickname VARCHAR(255) UNIQUE NOT NULL,
        salted_password VARCHAR(255) NOT NULL
    )
    "#,
  )
  .execute(&pool)
  .await
  .map_err(UserError::Database)?;

  println!();
  println!("{}", "=".repeat(40));
  println!("User table created successfully!");
  println!("{}", "=".repeat(40));
  println!("Table: user");
  println!("Columns: uuid, nickname, salted_password");
  println!();

  Ok(())
}

pub async fn create_user_session_table(dsn: &str) -> Result<(), UserError> {
  println!("Creating user_session table...");

  let pool = PgPoolOptions::new()
    .max_connections(1)
    .connect(dsn)
    .await
    .map_err(UserError::Database)?;

  // Check if table exists
  let table_exists: bool = sqlx::query_scalar(
    r#"
    SELECT EXISTS (
      SELECT FROM information_schema.tables 
      WHERE table_schema = 'public' 
      AND table_name = 'user_session'
    )
    "#,
  )
  .fetch_one(&pool)
  .await
  .map_err(UserError::Database)?;

  if table_exists {
    println!("Table user_session already exists.");
    println!();
    return Ok(());
  }

  sqlx::query(
    r#"
    CREATE TABLE IF NOT EXISTS user_session (
        id UUID PRIMARY KEY,
        user_id UUID NOT NULL REFERENCES "user"(uuid) ON DELETE CASCADE,
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
        expires_at TIMESTAMP WITH TIME ZONE NOT NULL
    )
    "#,
  )
  .execute(&pool)
  .await
  .map_err(UserError::Database)?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_session_user_id ON user_session(user_id)")
    .execute(&pool)
    .await
    .map_err(UserError::Database)?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_session_expires_at ON user_session(expires_at)")
    .execute(&pool)
    .await
    .map_err(UserError::Database)?;

  println!();
  println!("{}", "=".repeat(40));
  println!("User session table created successfully!");
  println!("{}", "=".repeat(40));
  println!("Table: user_session");
  println!("Columns: session_id, user_id, created_at, expires_at");
  println!();

  Ok(())
}

pub async fn drop_table_user_session(dsn: &str) -> Result<(), UserError> {
  println!("Dropping user_session table...");

  let pool = PgPoolOptions::new()
    .max_connections(1)
    .connect(dsn)
    .await
    .map_err(UserError::Database)?;

  // Check if table exists
  let table_exists: bool = sqlx::query_scalar(
    r#"
    SELECT EXISTS (
      SELECT FROM information_schema.tables 
      WHERE table_schema = 'public' 
      AND table_name = 'user_session'
    )
    "#,
  )
  .fetch_one(&pool)
  .await
  .map_err(UserError::Database)?;

  if !table_exists {
    println!("Table user_session does not exist.");
    return Ok(());
  }

  // Drop the table with CASCADE to handle foreign key constraints
  sqlx::query("DROP TABLE IF EXISTS user_session CASCADE")
    .execute(&pool)
    .await
    .map_err(UserError::Database)?;

  println!();
  println!("{}", "=".repeat(40));
  println!("User session table dropped successfully!");
  println!("{}", "=".repeat(40));
  println!("Dropped table: user_session");
  println!();

  Ok(())
}
