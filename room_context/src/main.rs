mod cli;

use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use axum::routing::{get, put};
use clap::Parser;
use room_context::{
  AccountId, Config, MaxPlayers, PostgresMessageRepository, PostgresRoomRepository, RoomError, RoomId, RoomService,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

#[derive(Serialize)]
struct RoomResponse {
  uuid: String,
  number: u32,
  name: String,
  creator: String,
  max_players: usize,
  created_at: String,
}

#[derive(Deserialize)]
struct CreateRoomRequest {
  name: String,
  creator: String,
  max_players: usize,
}

#[derive(Deserialize)]
struct UpdateRoomNameRequest {
  name: String,
}

#[derive(Deserialize)]
struct UpdateRoomMaxPlayersRequest {
  max_players: usize,
}

#[derive(Serialize)]
struct ErrorResponse {
  error: String,
}

async fn create_pool(config: &Config) -> Result<sqlx::PgPool, RoomError> {
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
    .map_err(RoomError::Database)?;

  Ok(pool)
}

async fn create_room_service(config: &Config) -> Result<RoomService, RoomError> {
  let pool = create_pool(config).await?;
  let room_repository = Box::new(PostgresRoomRepository::new(pool.clone()));
  let message_repository = Box::new(PostgresMessageRepository::new(pool));
  Ok(RoomService::new(room_repository, message_repository))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Initialize tracing subscriber
  tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();

  let cli = cli::Cli::parse();

  // Handle commands that don't need RoomService
  if let cli::Command::Migrates { command } = &cli.command {
    match command {
      cli::MigrateCommand::CreateAccountTable => {
        // TODO: load Config, rather than account_context::Config
        let account_config = account_context::Config::load()?;
        account_context::create_account_table(&account_config.dsn).await?;
      },
      cli::MigrateCommand::CreateRoomTable => {
        let config = Config::load()?;
        room_context::create_room_table(&config).await?;
      },
      cli::MigrateCommand::DropRoomTable => {
        let config = Config::load()?;
        room_context::drop_room_table(&config).await?;
      },
      cli::MigrateCommand::CreateRoomToAccountMessageTable => {
        let config = Config::load()?;
        room_context::create_room_to_account_message_table(&config).await?;
      },
      cli::MigrateCommand::CreateAccountToRoomMessageTable => {
        let config = Config::load()?;
        room_context::create_account_to_room_message_table(&config).await?;
      },
      cli::MigrateCommand::DropAllTables => {
        let config = Config::load()?;
        common_context::drop_all_tables(&config.dsn).await?;
      },
    }
    return Ok(());
  }

  let config = Config::load()?;
  let room_service = create_room_service(&config).await?;

  match cli.command {
    cli::Command::Serve => {
      let app = create_router(Arc::new(room_service));

      let listener = tokio::net::TcpListener::bind(&config.server_addr).await?;
      println!("🚀 Room service HTTP server running on http://{}", config.server_addr);
      println!("📡 API endpoints:");
      println!("  GET    /api/rooms - List all rooms");
      println!("  GET    /api/rooms/{{uuid}} - Get room by UUID");
      println!("  POST   /api/rooms - Create new room");
      println!("  PUT    /api/rooms/{{uuid}}/name - Update room name");
      println!("  PUT    /api/rooms/{{uuid}}/max-players - Update room max players");
      println!("  DELETE /api/rooms/{{uuid}} - Delete room");

      axum::serve(listener, app).await?;
      Ok(())
    },
    command => cli::handle_command(command, room_service).await,
  }
}

fn create_router(room_service: Arc<RoomService>) -> Router {
  Router::new()
    .route("/api/rooms", get(list_rooms).post(create_room))
    .route("/api/rooms/:uuid", get(get_room).delete(delete_room))
    .route("/api/rooms/:uuid/name", put(update_room_name))
    .route("/api/rooms/:uuid/max-players", put(update_room_max_players))
    .layer(ServiceBuilder::new().layer(CorsLayer::permissive()))
    .with_state(room_service)
}

async fn list_rooms(State(service): State<Arc<RoomService>>) -> Result<Json<Vec<RoomResponse>>, AppError> {
  let rooms = service.list_rooms(None).await?;
  let response: Vec<RoomResponse> = rooms
    .into_iter()
    .map(|room| RoomResponse {
      uuid: room.id().to_string(),
      number: room.number().value(),
      name: room.name().as_str().to_string(),
      creator: room.creator().to_string(),
      max_players: room.max_players().value(),
      created_at: room.created_at().to_rfc3339(),
    })
    .collect();
  Ok(Json(response))
}

async fn get_room(
  State(service): State<Arc<RoomService>>, axum::extract::Path(uuid): axum::extract::Path<String>,
) -> Result<Json<RoomResponse>, AppError> {
  let uuid = uuid.parse::<Uuid>().map_err(|_| AppError::InvalidUuid)?;
  let room_id = RoomId::from(uuid);
  let room = service.get_room_by_id(room_id).await?;
  let room = room.ok_or(AppError::NotFound)?;
  Ok(Json(RoomResponse {
    uuid: room.id().to_string(),
    number: room.number().value(),
    name: room.name().as_str().to_string(),
    creator: room.creator().to_string(),
    max_players: room.max_players().value(),
    created_at: room.created_at().to_rfc3339(),
  }))
}

async fn create_room(
  State(service): State<Arc<RoomService>>, Json(payload): Json<CreateRoomRequest>,
) -> Result<Json<RoomResponse>, AppError> {
  let creator_uuid = payload.creator.parse::<Uuid>().map_err(|_| AppError::InvalidUuid)?;
  let creator = AccountId::from(creator_uuid);
  let max_players =
    MaxPlayers::try_from(payload.max_players).map_err(|_| AppError::Room(RoomError::InvalidMaxPlayers))?;
  let room = service.create_room(&payload.name, creator, max_players).await?;
  Ok(Json(RoomResponse {
    uuid: room.id().to_string(),
    number: room.number().value(),
    name: room.name().as_str().to_string(),
    creator: room.creator().to_string(),
    max_players: room.max_players().value(),
    created_at: room.created_at().to_rfc3339(),
  }))
}

async fn update_room_name(
  State(service): State<Arc<RoomService>>, axum::extract::Path(uuid): axum::extract::Path<String>,
  Json(payload): Json<UpdateRoomNameRequest>,
) -> Result<StatusCode, AppError> {
  let uuid = uuid.parse::<Uuid>().map_err(|_| AppError::InvalidUuid)?;
  service.update_room_name(uuid, &payload.name).await?;
  Ok(StatusCode::NO_CONTENT)
}

async fn update_room_max_players(
  State(service): State<Arc<RoomService>>, axum::extract::Path(uuid): axum::extract::Path<String>,
  Json(payload): Json<UpdateRoomMaxPlayersRequest>,
) -> Result<StatusCode, AppError> {
  let uuid = uuid.parse::<Uuid>().map_err(|_| AppError::InvalidUuid)?;
  let room_id = RoomId::from(uuid);
  service.update_room_max_players(room_id, payload.max_players).await?;
  Ok(StatusCode::NO_CONTENT)
}

async fn delete_room(
  State(service): State<Arc<RoomService>>, axum::extract::Path(uuid): axum::extract::Path<String>,
) -> Result<StatusCode, AppError> {
  let uuid = uuid.parse::<Uuid>().map_err(|_| AppError::InvalidUuid)?;
  let room_id = RoomId::from(uuid);
  service.delete_room(room_id).await?;
  Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug)]
enum AppError {
  Room(RoomError),
  NotFound,
  Conflict,
  InvalidUuid,
}

impl From<RoomError> for AppError {
  fn from(err: RoomError) -> Self {
    match err {
      RoomError::NotFound => AppError::NotFound,
      RoomError::RoomNameExists => AppError::Conflict,
      _ => AppError::Room(err),
    }
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    match self {
      AppError::NotFound => (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          error: "Room not found".to_string(),
        }),
      )
        .into_response(),
      AppError::Conflict => (
        StatusCode::CONFLICT,
        Json(ErrorResponse {
          error: "Room name already exists".to_string(),
        }),
      )
        .into_response(),
      AppError::InvalidUuid => (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
          error: "Invalid UUID format".to_string(),
        }),
      )
        .into_response(),
      AppError::Room(e) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: e.to_string() }),
      )
        .into_response(),
    }
  }
}
