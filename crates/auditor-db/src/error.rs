use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("database error: {0}")]
    Db(String),

    #[error("migration error: {0}")]
    Migration(String),
}

pub type Result<T> = std::result::Result<T, auditor_core::error::CcaError>;
