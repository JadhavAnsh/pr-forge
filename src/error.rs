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
}

pub type Result<T> = std::result::Result<T, PrForgeError>;
