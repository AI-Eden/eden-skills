use thiserror::Error;

#[derive(Debug, Error)]
pub enum EdenError {
    #[error("invalid arguments: {0}")]
    InvalidArguments(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("state conflict: {0}")]
    Conflict(String),
    #[error("runtime error: {0}")]
    Runtime(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
