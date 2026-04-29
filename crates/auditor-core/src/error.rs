use thiserror::Error;

#[derive(Error, Debug)]
pub enum CcaError {
    #[error("database error: {0}")]
    Database(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("process error: {0}")]
    Process(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("unknown error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, CcaError>;
