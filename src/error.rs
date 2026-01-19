use thiserror::Error;

#[derive(Error, Debug)]
pub enum PrForgeError {
    #[error("Git error: {0}")]
    GitError(String),

    #[error("Repository not found at {0}")]
    RepoNotFound(String),

    #[error("Branch not found: {0}")]
    BranchNotFound(String),

    #[error("Invalid branch comparison: {0}")]
    #[allow(dead_code)]
    InvalidBranchComparison(String),

    #[error("Failed to analyze commits: {0}")]
    CommitAnalysisError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Internal error: {0}")]
    #[allow(dead_code)]
    Internal(String),

    // Network and API errors
    #[error("API error: HTTP {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Request timeout after {duration_ms}ms")]
    Timeout { duration_ms: u64 },

    #[error("Rate limited. Retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("API response parsing failed: {0}")]
    ApiParseError(String),

    #[error("API key not configured")]
    MissingApiKey,
}

pub type Result<T> = std::result::Result<T, PrForgeError>;
