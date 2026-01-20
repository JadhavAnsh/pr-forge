pub mod groq_client;
pub mod rate_limiter;

pub use groq_client::{GroqClient, GroqConfig};
pub use rate_limiter::RateLimitConfig;

use crate::error::{PrForgeError, Result};

/// Configuration for AI analysis
#[derive(Debug, Clone)]
// Configuration for AI features (for future extensibility)
#[allow(dead_code)]
pub struct AiConfig {
    pub groq_config: GroqConfig,
    pub rate_limit_config: RateLimitConfig,
    pub max_commits: usize,
    pub max_files: usize,
    pub max_commit_msg_chars: usize,
    pub enable_caching: bool,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            groq_config: GroqConfig::default(),
            rate_limit_config: RateLimitConfig::default(),
            max_commits: 1000,
            max_files: 500,
            max_commit_msg_chars: 500,
            enable_caching: true,
        }
    }
}

/// Changelog entry with categorization
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChangelogEntry {
    pub title: String,
    pub description: String,
    pub files: Vec<String>,
}

/// Categorized changes from Groq analysis
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChangelogSection {
    pub category: String,
    pub emoji: String,
    pub entries: Vec<ChangelogEntry>,
}

/// Complete changelog from AI analysis
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiGeneratedChangelog {
    pub sections: Vec<ChangelogSection>,
    pub summary: String,
    pub breaking_changes: Vec<String>,
}

/// Load API key from environment with validation
pub fn load_api_key() -> Result<String> {
    std::env::var("GROQ_API_KEY")
        .or_else(|_| {
            // Try loading from .env file if it exists
            dotenv::dotenv().ok();
            std::env::var("GROQ_API_KEY")
        })
        .map_err(|_| PrForgeError::MissingApiKey)
        .and_then(|key| {
            if key.trim().is_empty() {
                Err(PrForgeError::MissingApiKey)
            } else {
                Ok(key)
            }
        })
}

/// Validate API key format (Groq keys start with "gsk_")
pub fn validate_api_key(key: &str) -> Result<()> {
    if !key.starts_with("gsk_") {
        return Err(PrForgeError::ConfigError(
            "Invalid API key format. Groq keys should start with 'gsk_'".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_api_key_valid() {
        let key = "gsk_validkey123456789";
        assert!(validate_api_key(key).is_ok());
    }

    #[test]
    fn test_validate_api_key_invalid() {
        let key = "invalid_key_format";
        assert!(validate_api_key(key).is_err());
    }
}
