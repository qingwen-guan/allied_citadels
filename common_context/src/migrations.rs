use sqlx::postgres::PgPoolOptions;

#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
  #[error("Database error: {0}")]
  Database(#[from] sqlx::Error),
  #[error("Config error: {0}")]
  Config(String),
}

pub async fn drop_all_tables(dsn: &str) -> Result<(), MigrationError> {
  println!("Dropping all tables from database...");

  let pool = PgPoolOptions::new().max_connections(1).connect(dsn).await?;

  // Get all table names from the public schema
  let tables: Vec<String> = sqlx::query_scalar(
    r#"
    SELECT tablename 
    FROM pg_tables 
    WHERE schemaname = 'public'
    ORDER BY tablename
    "#,
  )
  .fetch_all(&pool)
  .await?;

  if tables.is_empty() {
    println!("No tables found in the database.");
    return Ok(());
  }

  println!("Found {} table(s):", tables.len());
  for table in &tables {
    println!("  - {}", table);
  }

  // Drop all tables with CASCADE to handle foreign key constraints
  for table in &tables {
    // Quote table name to handle any special characters
    let drop_query = format!(r#"DROP TABLE IF EXISTS "{}" CASCADE"#, table);
    println!("Dropping table: {}", table);
    sqlx::query(&drop_query).execute(&pool).await?;
  }

  println!();
  println!("{}", "=".repeat(40));
  println!("All tables dropped successfully!");
  println!("{}", "=".repeat(40));
  println!("Dropped {} table(s)", tables.len());
  println!();

  Ok(())
}
