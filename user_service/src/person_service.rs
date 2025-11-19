use tokio_postgres::{Client, NoTls};
use uuid::Uuid;

use crate::person::User;

pub struct PersonService {
  client: Client,
}

impl PersonService {
  pub async fn new() -> anyhow::Result<Self> {
    // Connection string matches postgres_config.bat settings
    // URL-encode the password: ! becomes %21
    let database_url = "postgresql://postgres:Pwd123@127.0.0.1:5432/allied_citadels";
    let (client, connection) = tokio_postgres::connect(database_url, NoTls).await?;

    // Spawn the connection task
    tokio::spawn(async move {
      if let Err(e) = connection.await {
        eprintln!("connection error: {}", e);
      }
    });

    Ok(Self { client })
  }

  // Create - Insert a new user
  pub async fn create_user(&self, nickname: &str) -> anyhow::Result<String> {
    // Check if nickname already exists
    let existing: Option<Uuid> = self
      .client
      .query_opt("SELECT uuid FROM person WHERE nickname = $1", &[&nickname])
      .await?
      .map(|row| row.get(0));

    if existing.is_some() {
      return Err(anyhow::anyhow!("Nickname already exists"));
    }

    let user_uuid = Uuid::new_v4();
    let password = format!("{:06}", rand::random_range(0..=999999));
    let user = User::new(user_uuid, nickname, &password);

    self
      .client
      .execute(
        "INSERT INTO person (uuid, nickname, salted_password) VALUES ($1, $2, $3)",
        &[&user.uuid(), &user.nickname(), &user.salted_password()],
      )
      .await?;

    Ok(password)
  }

  // Read - Get user by UUID
  pub async fn get_user_by_uuid(&self, uuid: Uuid) -> anyhow::Result<Option<User>> {
    let row = self
      .client
      .query_opt(
        "SELECT uuid, nickname, salted_password FROM person WHERE uuid = $1",
        &[&uuid],
      )
      .await?;

    Ok(row.map(User::from))
  }

  // Read - Get user by nickname
  pub async fn get_user_by_nickname(&self, nickname: &str) -> anyhow::Result<Option<User>> {
    let row = self
      .client
      .query_opt(
        "SELECT uuid, nickname, salted_password FROM person WHERE nickname = $1",
        &[&nickname],
      )
      .await?;

    Ok(row.map(User::from))
  }

  // Read - List all users
  pub async fn list_users(&self) -> anyhow::Result<Vec<User>> {
    let rows = self
      .client
      .query(
        "SELECT uuid, nickname, salted_password FROM person ORDER BY nickname",
        &[],
      )
      .await?;

    Ok(rows.into_iter().map(User::from).collect())
  }

  // Update - Update user nickname
  pub async fn update_user_nickname(&self, uuid: Uuid, new_nickname: &str) -> anyhow::Result<()> {
    // Check if new nickname already exists
    let existing: Option<Uuid> = self
      .client
      .query_opt(
        "SELECT uuid FROM person WHERE nickname = $1 AND uuid != $2",
        &[&new_nickname, &uuid],
      )
      .await?
      .map(|row| row.get(0));

    if existing.is_some() {
      return Err(anyhow::anyhow!("Nickname already exists"));
    }

    let rows_affected = self
      .client
      .execute(
        "UPDATE person SET nickname = $1 WHERE uuid = $2",
        &[&new_nickname, &uuid],
      )
      .await?;

    if rows_affected == 0 {
      return Err(anyhow::anyhow!("User not found"));
    }

    Ok(())
  }

  // Update - Reset password
  pub async fn reset_password(&self, uuid: Uuid) -> anyhow::Result<String> {
    let user = self.get_user_by_uuid(uuid).await?;
    if user.is_none() {
      return Err(anyhow::anyhow!("User not found"));
    }

    let password = format!("{:06}", rand::random_range(0..=999999));
    let new_user = User::new(uuid, user.as_ref().unwrap().nickname(), &password);

    let rows_affected = self
      .client
      .execute(
        "UPDATE person SET salted_password = $1 WHERE uuid = $2",
        &[&new_user.salted_password(), &uuid],
      )
      .await?;

    if rows_affected == 0 {
      return Err(anyhow::anyhow!("User not found"));
    }

    Ok(password)
  }

  // Update - Reset password by nickname
  pub async fn reset_password_by_name(&self, nickname: &str) -> anyhow::Result<String> {
    let user = self.get_user_by_nickname(nickname).await?;
    if user.is_none() {
      return Err(anyhow::anyhow!("User not found"));
    }

    self.reset_password(user.unwrap().uuid()).await
  }

  // Delete - Remove user by UUID
  pub async fn delete_user(&self, uuid: Uuid) -> anyhow::Result<()> {
    let rows_affected = self
      .client
      .execute("DELETE FROM person WHERE uuid = $1", &[&uuid])
      .await?;

    if rows_affected == 0 {
      return Err(anyhow::anyhow!("User not found"));
    }

    Ok(())
  }

  // Delete - Remove user by nickname
  pub async fn delete_user_by_nickname(&self, nickname: &str) -> anyhow::Result<()> {
    let user = self.get_user_by_nickname(nickname).await?;
    if user.is_none() {
      return Err(anyhow::anyhow!("User not found"));
    }

    self.delete_user(user.unwrap().uuid()).await
  }
}
