use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("could not read task file: {0}")]
    Io(#[from] std::io::Error),

    #[error("could not parse task file: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("no task with id {0}")]
    NotFound(u32),

    #[error("invalid operation: {0}")]
    InvalidOperation(String),

    #[error("error: {0}")]
    Other(String),
}
