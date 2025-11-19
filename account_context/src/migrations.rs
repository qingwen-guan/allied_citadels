use sqlx::postgres::PgPoolOptions;

use crate::error::AccountError;

pub async fn create_account_table(dsn: &str) -> Result<(), AccountError> {
  println!("Creating account table...");

  let pool = PgPoolOptions::new()
    .max_connections(1)
    .connect(dsn)
    .await
    .map_err(AccountError::Database)?;

  // Check if table exists
  let table_exists: bool = sqlx::query_scalar(
    r#"
    SELECT EXISTS (
      SELECT FROM information_schema.tables 
      WHERE table_schema = 'public' 
      AND table_name = 'account'
    )
    "#,
  )
  .fetch_one(&pool)
  .await
  .map_err(AccountError::Database)?;

  if table_exists {
    println!("Table account already exists.");
    println!();
    return Ok(());
  }

  sqlx::query(
    r#"
    CREATE TABLE IF NOT EXISTS account (
        id UUID PRIMARY KEY,
        nickname VARCHAR(255) UNIQUE NOT NULL,
        salted_password VARCHAR(255) NOT NULL
    )
    "#,
  )
  .execute(&pool)
  .await
  .map_err(AccountError::Database)?;

  println!();
  println!("{}", "=".repeat(40));
  println!("Account table created successfully!");
  println!("{}", "=".repeat(40));
  println!("Table: account");
  println!("Columns: uuid, nickname, salted_password");
  println!();

  Ok(())
}

pub async fn create_account_session_table(dsn: &str) -> Result<(), AccountError> {
  println!("Creating account_session table...");

  let pool = PgPoolOptions::new()
    .max_connections(1)
    .connect(dsn)
    .await
    .map_err(AccountError::Database)?;

  // Check if table exists
  let table_exists: bool = sqlx::query_scalar(
    r#"
    SELECT EXISTS (
      SELECT FROM information_schema.tables 
      WHERE table_schema = 'public' 
      AND table_name = 'account_session'
    )
    "#,
  )
  .fetch_one(&pool)
  .await
  .map_err(AccountError::Database)?;

  if table_exists {
    println!("Table account_session already exists.");
    println!();
    return Ok(());
  }

  sqlx::query(
    r#"
    CREATE TABLE IF NOT EXISTS account_session (
        id UUID PRIMARY KEY,
        account_id UUID NOT NULL REFERENCES account(id) ON DELETE CASCADE,
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
        expires_at TIMESTAMP WITH TIME ZONE NOT NULL
    )
    "#,
  )
  .execute(&pool)
  .await
  .map_err(AccountError::Database)?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_account_session_account_id ON account_session(account_id)")
    .execute(&pool)
    .await
    .map_err(AccountError::Database)?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_account_session_expires_at ON account_session(expires_at)")
    .execute(&pool)
    .await
    .map_err(AccountError::Database)?;

  println!();
  println!("{}", "=".repeat(40));
  println!("Account session table created successfully!");
  println!("{}", "=".repeat(40));
  println!("Table: account_session");
  println!("Columns: session_id, account_id, created_at, expires_at");
  println!();

  Ok(())
}

pub async fn drop_table_account_session(dsn: &str) -> Result<(), AccountError> {
  println!("Dropping account_session table...");

  let pool = PgPoolOptions::new()
    .max_connections(1)
    .connect(dsn)
    .await
    .map_err(AccountError::Database)?;

  // Check if table exists
  let table_exists: bool = sqlx::query_scalar(
    r#"
    SELECT EXISTS (
      SELECT FROM information_schema.tables 
      WHERE table_schema = 'public' 
      AND table_name = 'account_session'
    )
    "#,
  )
  .fetch_one(&pool)
  .await
  .map_err(AccountError::Database)?;

  if !table_exists {
    println!("Table account_session does not exist.");
    return Ok(());
  }

  // Drop the table with CASCADE to handle foreign key constraints
  sqlx::query("DROP TABLE IF EXISTS account_session CASCADE")
    .execute(&pool)
    .await
    .map_err(AccountError::Database)?;

  println!();
  println!("{}", "=".repeat(40));
  println!("Account session table dropped successfully!");
  println!("{}", "=".repeat(40));
  println!("Dropped table: account_session");
  println!();

  Ok(())
}
