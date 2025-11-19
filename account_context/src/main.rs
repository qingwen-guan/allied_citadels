mod cli;

use std::sync::Arc;

use account_context::{
  AccountError, AccountFactory, AccountService, Config, PostgresAccountRepository, PostgresSessionRepository, Salt,
};
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use axum::routing::{get, post};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

#[derive(Serialize)]
struct AccountResponse {
  uuid: String,
  nickname: String,
}

#[derive(Deserialize, Serialize)]
struct CreateAccountRequest {
  nickname: String,
}

#[derive(Serialize)]
struct CreateAccountResponse {
  uuid: Uuid,
  nickname: String,
  password: String,
}

#[derive(Serialize)]
struct ErrorResponse {
  error: String,
}

async fn create_pool(config: &Config) -> Result<sqlx::PgPool, AccountError> {
  let pool = PgPoolOptions::new()
    .max_connections(config.max_connections)
    .min_connections(config.min_connections)
    .acquire_timeout(std::time::Duration::from_secs(config.acquire_timeout_seconds))
    .idle_timeout(Some(std::time::Duration::from_secs(config.idle_timeout_seconds)))
    .max_lifetime(Some(std::time::Duration::from_secs(config.max_lifetime_seconds)))
    .connect(&config.dsn)
    .await?;

  // Test the connection
  sqlx::query("SELECT 1")
    .execute(&pool)
    .await
    .map_err(AccountError::Database)?;

  Ok(pool)
}

async fn create_account_service(config: &Config) -> Result<AccountService, AccountError> {
  let pool = create_pool(config).await?;
  let repository = Box::new(PostgresAccountRepository::new(pool.clone()));
  let session_repository = Box::new(PostgresSessionRepository::new(pool));
  let account_factory = AccountFactory::new(Salt::from(config.password_salt.as_str()));
  Ok(AccountService::new(
    config,
    repository,
    session_repository,
    account_factory,
  ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Initialize tracing subscriber
  tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();

  let cli = cli::Cli::parse();

  // Handle commands that don't need AccountService
  // TODO: move the logic to cli/migrates.rs
  if let cli::Command::Migrates { command } = &cli.command {
    let config = Config::load()?;
    match command {
      // TODO: add create_all_tables command
      cli::MigrateCommand::CreateAccountTable => {
        account_context::create_account_table(&config.dsn).await?;
      },
      cli::MigrateCommand::CreateAccountSessionTable => {
        account_context::create_account_session_table(&config.dsn).await?;
      },
      cli::MigrateCommand::DropTableAccountSession => {
        account_context::drop_table_account_session(&config.dsn).await?;
      },
      cli::MigrateCommand::DropAllTables => {
        common_context::drop_all_tables(&config.dsn).await?;
      },
    }
    return Ok(());
  }

  let config = Config::load()?;
  let account_service = create_account_service(&config).await?;

  match cli.command {
    cli::Command::Serve => {
      let app = create_router(Arc::new(account_service));

      let listener = tokio::net::TcpListener::bind(&config.server_addr).await?;
      println!(
        "🚀 Account service HTTP server running on http://{}",
        config.server_addr
      );
      println!("📡 API endpoints:");
      println!("  GET  /api/accounts - List all accounts");
      println!("  GET  /api/accounts/{{nickname}} - Get account by nickname");
      println!("  POST /api/accounts - Create new account");
      println!("  POST /api/accounts/{{nickname}}/reset-password - Reset password");
      println!("  DELETE /api/accounts/{{nickname}} - Delete account");

      axum::serve(listener, app).await?;
      Ok(())
    },
    command => cli::handle_command(command, account_service).await,
  }
}

fn create_router(account_service: Arc<AccountService>) -> Router {
  Router::new()
    .route("/api/accounts", get(list_accounts).post(create_account))
    .route("/api/accounts/{nickname}", get(get_account).delete(delete_account))
    .route("/api/accounts/{nickname}/reset-password", post(reset_password))
    .layer(ServiceBuilder::new().layer(CorsLayer::permissive()))
    .with_state(account_service)
}

async fn list_accounts(State(service): State<Arc<AccountService>>) -> Result<Json<Vec<AccountResponse>>, AppError> {
  let accounts = service.list_accounts().await?;
  let response: Vec<AccountResponse> = accounts
    .into_iter()
    .map(|acc| AccountResponse {
      uuid: acc.uuid().to_string(),
      nickname: acc.nickname().to_string(),
    })
    .collect();
  Ok(Json(response))
}

async fn get_account(
  State(service): State<Arc<AccountService>>, axum::extract::Path(nickname): axum::extract::Path<String>,
) -> Result<Json<AccountResponse>, AppError> {
  let account = service.get_account_by_nickname(&nickname).await?;
  let account = account.ok_or(AppError::NotFound)?;
  Ok(Json(AccountResponse {
    uuid: account.uuid().to_string(),
    nickname: account.nickname().to_string(),
  }))
}

async fn create_account(
  State(service): State<Arc<AccountService>>, Json(payload): Json<CreateAccountRequest>,
) -> Result<Json<CreateAccountResponse>, AppError> {
  let (uuid, password) = service.create_account(&payload.nickname).await?;
  Ok(Json(CreateAccountResponse {
    uuid,
    nickname: payload.nickname,
    password,
  }))
}

async fn reset_password(
  State(service): State<Arc<AccountService>>, axum::extract::Path(nickname): axum::extract::Path<String>,
) -> Result<Json<CreateAccountResponse>, AppError> {
  let (uuid, password) = service.reset_password_by_name(&nickname).await?;
  Ok(Json(CreateAccountResponse {
    uuid,
    nickname,
    password,
  }))
}

async fn delete_account(
  State(service): State<Arc<AccountService>>, axum::extract::Path(nickname): axum::extract::Path<String>,
) -> Result<StatusCode, AppError> {
  service.delete_account_by_nickname(&nickname).await?;
  Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug)]
enum AppError {
  Account(AccountError),
  NotFound,
  Conflict,
}

impl From<AccountError> for AppError {
  fn from(err: AccountError) -> Self {
    match err {
      AccountError::NotFound => AppError::NotFound,
      AccountError::NicknameExists => AppError::Conflict,
      _ => AppError::Account(err),
    }
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    match self {
      AppError::NotFound => (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          error: "Account not found".to_string(),
        }),
      )
        .into_response(),
      AppError::Conflict => (
        StatusCode::CONFLICT,
        Json(ErrorResponse {
          error: "Nickname already exists".to_string(),
        }),
      )
        .into_response(),
      AppError::Account(e) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: e.to_string() }),
      )
        .into_response(),
    }
  }
}
