use thiserror::Error;

#[derive(Error, Debug)]
pub enum ComplianceError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Integration error: {0}")]
    Integration(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ComplianceError>;