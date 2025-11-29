use thiserror::Error;

#[derive(Error, Debug)]
pub enum SessionContextError {
    #[error("Generic error: {0}")]
    Generic(String),
}
