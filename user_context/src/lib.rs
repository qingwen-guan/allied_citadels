mod config;
mod domain;
mod error;
mod infra;
pub mod migrations;
mod user_service;

pub use config::Config;
pub use user_service::UserService;
// Re-export Salt from domain module (which already exports it from valueobjects)
pub use domain::valueobjects::{Salt, SessionId, UserId, UserToUserMessageId};
pub use domain::{
  SessionInfo, SessionManager, SessionStatus, User, UserFactory, UserManager, UserRepository, UserToUserMessage,
  UserToUserMessageDetails, UserToUserRawMessage,
};
pub use error::UserError;
pub use infra::{PostgresSessionRepository, PostgresUserRepository};
