mod cli;

use std::sync::Arc;

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
use user_context::UserService;
use user_context::domain::valueobjects::Salt;
use user_context::infra::{PostgresSessionRepository, PostgresUserRepository};
use user_context::{Config, UserError, UserFactory};

#[derive(Serialize)]
struct UserResponse {
  user_id: String,
  nickname: String,
}

#[derive(Deserialize, Serialize)]
struct CreateUserRequest {
  nickname: String,
}

#[derive(Serialize)]
struct CreateUserResponse {
  user_id: String,
  nickname: String,
  password: String,
}

#[derive(Serialize)]
struct ErrorResponse {
  error: String,
}

async fn create_pool(config: &Config) -> Result<sqlx::PgPool, UserError> {
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
    .map_err(UserError::Database)?;

  Ok(pool)
}

async fn create_user_service(config: &Config) -> Result<UserService, UserError> {
  let pool = create_pool(config).await?;
  let repository = Box::new(PostgresUserRepository::new(pool.clone()));
  let session_repository = Box::new(PostgresSessionRepository::new(pool));
  let user_factory = UserFactory::new(Salt::from(config.password_salt.as_str()));
  Ok(UserService::new(config, repository, session_repository, user_factory))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Initialize tracing subscriber
  tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();

  let cli = cli::Cli::parse();
  let config = Config::load()?;

  // Handle commands that don't need UserService
  if let cli::Command::Migrates { command } = cli.command {
    return cli::handle_migrate_command(&config.dsn, command).await;
  }
  let user_service = create_user_service(&config).await?;

  match cli.command {
    cli::Command::Serve => {
      let app = create_router(Arc::new(user_service));

      let listener = tokio::net::TcpListener::bind(&config.server_addr).await?;
      println!("🚀 User service HTTP server running on http://{}", config.server_addr);
      println!("📡 API endpoints:");
      println!("  GET  /api/users - List all users");
      println!("  GET  /api/users/{{nickname}} - Get user by nickname");
      println!("  POST /api/users - Create new user");
      println!("  POST /api/users/{{nickname}}/reset-password - Reset password");
      println!("  DELETE /api/users/{{nickname}} - Delete user");

      axum::serve(listener, app).await?;
      Ok(())
    },
    command => cli::handle_command(command, user_service).await,
  }
}

fn create_router(user_service: Arc<UserService>) -> Router {
  Router::new()
    .route("/api/users", get(list_users).post(create_user))
    .route("/api/users/{nickname}", get(get_user).delete(delete_user))
    .route("/api/users/{nickname}/reset-password", post(reset_password))
    .layer(ServiceBuilder::new().layer(CorsLayer::permissive()))
    .with_state(user_service)
}

async fn list_users(State(service): State<Arc<UserService>>) -> Result<Json<Vec<UserResponse>>, AppError> {
  let users = service.list_users().await?;
  let response: Vec<UserResponse> = users
    .into_iter()
    .map(|user| UserResponse {
      user_id: user.user_id,
      nickname: user.nickname,
    })
    .collect();
  Ok(Json(response))
}

async fn get_user(
  State(service): State<Arc<UserService>>, axum::extract::Path(nickname): axum::extract::Path<String>,
) -> Result<Json<UserResponse>, AppError> {
  let user = service.get_user_by_nickname(&nickname).await?;
  Ok(Json(UserResponse {
    user_id: user.user_id,
    nickname: user.nickname,
  }))
}

async fn create_user(
  State(service): State<Arc<UserService>>, Json(payload): Json<CreateUserRequest>,
) -> Result<Json<CreateUserResponse>, AppError> {
  let result = service.create_user(&payload.nickname).await?;
  Ok(Json(CreateUserResponse {
    user_id: result.user_id,
    nickname: payload.nickname,
    password: result.password,
  }))
}

async fn reset_password(
  State(service): State<Arc<UserService>>, axum::extract::Path(nickname): axum::extract::Path<String>,
) -> Result<Json<CreateUserResponse>, AppError> {
  let result = service.reset_password_by_name(&nickname).await?;
  Ok(Json(CreateUserResponse {
    user_id: result.user_id,
    nickname,
    password: result.password,
  }))
}

async fn delete_user(
  State(service): State<Arc<UserService>>, axum::extract::Path(nickname): axum::extract::Path<String>,
) -> Result<StatusCode, AppError> {
  service.delete_user_by_nickname(&nickname).await?;
  Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug)]
enum AppError {
  User(UserError),
  NotFound,
  Conflict,
}

impl From<UserError> for AppError {
  fn from(err: UserError) -> Self {
    match err {
      UserError::NotFound => AppError::NotFound,
      UserError::NicknameExists => AppError::Conflict,
      _ => AppError::User(err),
    }
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    match self {
      AppError::NotFound => (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          error: "User not found".to_string(),
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
      AppError::User(e) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: e.to_string() }),
      )
        .into_response(),
    }
  }
}
