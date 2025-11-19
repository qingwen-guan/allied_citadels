mod account_service;
mod config;
mod domain;
mod error;
mod infra;
mod migrations;

pub use account_service::AccountService;
pub use config::Config;
// Re-export Salt from domain module (which already exports it from valueobjects)
pub use domain::valueobjects::{AccountId, AccountToAccountMessageId, Salt, SessionId};
pub use domain::{
  Account, AccountFactory, AccountManager, AccountRepository, AccountToAccountMessage,
  AccountToAccountMessageDetails, AccountToAccountRawMessage, SessionManager, SessionStatus,
};
pub use error::AccountError;
pub use infra::{PostgresAccountRepository, PostgresSessionRepository};
pub use migrations::{create_account_session_table, create_account_table, drop_table_account_session};
