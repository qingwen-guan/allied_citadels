mod cli;

use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use axum::routing::{get, put};
use clap::Parser;
use common_context::database::create_postgres_pool;
use room_context::domain::factories::RoomConfigFactory;
use room_context::domain::valueobjects::RoomConfig;
use room_context::errors::RoomError;
use room_context::infra::{PostgresMessageRepository, PostgresRoomRepository};
use room_context::migrations;
use room_context::services::RoomService;
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use user_context::infra::PostgresUserRepository;

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

async fn create_room_service(config: &RoomConfig) -> Result<RoomService, RoomError> {
  let pool = create_postgres_pool(&config.db).await.map_err(RoomError::Database)?;
  let room_repository = Box::new(PostgresRoomRepository::new(pool.clone()));
  let user_repository = Box::new(PostgresUserRepository::new(pool.clone()));
  let message_repository = Box::new(PostgresMessageRepository::new(pool));
  Ok(RoomService::new(room_repository, user_repository, message_repository))
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
    let config = RoomConfigFactory::new().load()?;
    match command {
      cli::MigrateCommand::CreateUserTable => {
        user_context::migrations::create_user_table(&config.db.dsn).await?;
      },
      cli::MigrateCommand::CreateRoomTable => {
        migrations::create_room_table(&config.db).await?;
      },
      cli::MigrateCommand::DropRoomTable => {
        migrations::drop_room_table(&config.db).await?;
      },
      cli::MigrateCommand::CreateRoomToUserMessageTable => {
        migrations::create_room_to_user_message_table(&config.db).await?;
      },
      cli::MigrateCommand::CreateAllTables => {
        migrations::create_all_tables(&config.db).await?;
      },
      cli::MigrateCommand::DropAllTables => {
        common_context::drop_all_tables(&config.db.dsn).await?;
      },
    }
    return Ok(());
  }

  let config = RoomConfigFactory::new().load()?;
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
  let offset = None;
  let limit = None;
  let rooms = service.list_rooms(offset, limit).await?;
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
  let room = service.get_room_by_id(&uuid).await?;
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
  let room = service
    .create_room(&payload.creator, &payload.name, payload.max_players)
    .await
    .map_err(AppError::Room)?;
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
  service.update_room_name(&uuid, &payload.name).await?;
  Ok(StatusCode::NO_CONTENT)
}

async fn update_room_max_players(
  State(service): State<Arc<RoomService>>, axum::extract::Path(uuid): axum::extract::Path<String>,
  Json(payload): Json<UpdateRoomMaxPlayersRequest>,
) -> Result<StatusCode, AppError> {
  service.update_room_max_players(&uuid, payload.max_players).await?;
  Ok(StatusCode::NO_CONTENT)
}

async fn delete_room(
  State(service): State<Arc<RoomService>>, axum::extract::Path(uuid): axum::extract::Path<String>,
) -> Result<StatusCode, AppError> {
  service.delete_room(&uuid).await.map_err(AppError::Room)?;
  Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug)]
enum AppError {
  Room(RoomError),
  NotFound,
  Conflict,
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
      AppError::Room(e) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: e.to_string() }),
      )
        .into_response(),
    }
  }
}
