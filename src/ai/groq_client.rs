use crate::error::{PrForgeError, Result};
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for Groq API client
#[derive(Debug, Clone)]
pub struct GroqConfig {
    pub api_key: String,
    pub api_endpoint: String,
    pub model: String,
    pub timeout_secs: u64,
    pub max_tokens: usize,
    pub temperature: f32,
}

impl Default for GroqConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_endpoint: "https://api.groq.com/openai/v1/chat/completions".to_string(),
            model: "mixtral-8x7b-32768".to_string(),
            timeout_secs: 30,
            max_tokens: 8000,
            temperature: 0.7,
        }
    }
}

/// Message role for chat completion
#[derive(Debug, Serialize, Deserialize)]
pub enum MessageRole {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

/// Message in chat completion request
#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }
}

/// Groq API request
#[derive(Debug, Serialize)]
pub struct GroqRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: usize,
    pub temperature: f32,
}

/// Choice in response
#[derive(Debug, Deserialize, Clone)]
pub struct Choice {
    pub message: ResponseMessage,
    pub finish_reason: Option<String>,
}

/// Message in response
#[derive(Debug, Deserialize, Clone)]
pub struct ResponseMessage {
    pub content: Option<String>,
}

/// Usage statistics in response
#[derive(Debug, Deserialize, Clone)]
pub struct Usage {
    pub prompt_tokens: Option<usize>,
    pub completion_tokens: Option<usize>,
    pub total_tokens: Option<usize>,
}

/// Error info in response
#[derive(Debug, Deserialize, Clone)]
pub struct ErrorInfo {
    pub message: String,
    pub code: Option<u32>,
}

/// Groq API response
#[derive(Debug, Deserialize, Clone)]
pub struct GroqResponse {
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
    pub error: Option<ErrorInfo>,
}

impl GroqResponse {
    /// Validate and extract content from response
    pub fn validate(&self) -> Result<String> {
        // Check for error in response
        if let Some(error) = &self.error {
            return Err(PrForgeError::ApiError {
                status: error.code.unwrap_or(500) as u16,
                message: error.message.clone(),
            });
        }

        // Check if we have choices
        if self.choices.is_empty() {
            return Err(PrForgeError::ApiParseError(
                "No choices in response".to_string(),
            ));
        }

        // Extract content
        let content = self.choices[0]
            .message
            .content
            .as_ref()
            .ok_or_else(|| PrForgeError::ApiParseError("Empty content in response".to_string()))?;

        if content.is_empty() {
            return Err(PrForgeError::ApiParseError(
                "Content is empty string".to_string(),
            ));
        }

        // Detect truncation
        if content.ends_with("...") || content.ends_with("[truncated]") {
            warn!("Response appears truncated, may have exceeded token limit");
        }

        Ok(content.clone())
    }
}

/// Groq API client with retry logic and timeout
pub struct GroqClient {
    config: GroqConfig,
    client: reqwest::blocking::Client,
}

impl GroqClient {
    /// Create new Groq client
    pub fn new(mut config: GroqConfig) -> Result<Self> {
        // Validate API key
        if config.api_key.is_empty() {
            config.api_key = super::load_api_key()?;
        }
        super::validate_api_key(&config.api_key)?;

        let timeout = Duration::from_secs(config.timeout_secs);
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| {
                PrForgeError::NetworkError(format!("Failed to create HTTP client: {}", e))
            })?;

        debug!("Groq client initialized with model: {}", config.model);

        Ok(Self { config, client })
    }

    /// Send request to Groq API with retry logic
    pub fn chat_completion(
        &self,
        system_prompt: String,
        user_message: String,
    ) -> Result<String> {
        let request = GroqRequest {
            model: self.config.model.clone(),
            messages: vec![
                ChatMessage::system(system_prompt),
                ChatMessage::user(user_message),
            ],
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
        };

        // Retry with exponential backoff
        let max_retries = 3;
        let mut base_delay_ms = 1000u64;
        let max_delay_ms = 10000u64;
        let jitter_factor = 0.2; // +/- 20% jitter

        for attempt in 0..=max_retries {
            match self.send_request(&request) {
                Ok(response) => {
                    debug!("Groq API request succeeded on attempt {}", attempt + 1);
                    return response.validate();
                }
                Err(e) => {
                    if attempt < max_retries {
                        // Classify error as retryable or not
                        if self.is_retryable_error(&e) {
                            let delay = self.calculate_backoff(
                                base_delay_ms,
                                max_delay_ms,
                                jitter_factor,
                            );
                            warn!(
                                "Groq API request failed (attempt {}), retrying after {}ms: {}",
                                attempt + 1,
                                delay,
                                e
                            );
                            std::thread::sleep(Duration::from_millis(delay));
                            base_delay_ms = (base_delay_ms * 2).min(max_delay_ms);
                        } else {
                            // Non-retryable error, fail immediately
                            return Err(e);
                        }
                    } else {
                        warn!(
                            "Groq API request failed after {} attempts: {}",
                            max_retries + 1,
                            e
                        );
                        return Err(e);
                    }
                }
            }
        }

        Err(PrForgeError::Internal(
            "Unexpected state in retry loop".to_string(),
        ))
    }

    /// Send HTTP request to Groq API
    fn send_request(&self, request: &GroqRequest) -> Result<GroqResponse> {
        debug!("Sending request to Groq API");

        let response = self
            .client
            .post(&self.config.api_endpoint)
            .bearer_auth(&self.config.api_key)
            .json(request)
            .send()
            .map_err(|err: reqwest::Error| {
                if err.is_timeout() {
                    PrForgeError::Timeout {
                        duration_ms: self.config.timeout_secs * 1000,
                    }
                } else if err.is_connect() {
                    PrForgeError::NetworkError(format!("Connection error: {}", err))
                } else {
                    PrForgeError::NetworkError(format!("Request failed: {}", err))
                }
            })?;

        let status = response.status();
        let status_code = status.as_u16();

        if status_code == 429 {
            // Rate limited
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v: &reqwest::header::HeaderValue| v.to_str().ok())
                .and_then(|s: &str| s.parse::<u64>().ok())
                .unwrap_or(60);

            return Err(PrForgeError::RateLimited {
                retry_after_secs: retry_after,
            });
        }

        if !status.is_success() {
            return Err(PrForgeError::ApiError {
                status: status_code,
                message: format!("HTTP {}: {}", status_code, status.canonical_reason().unwrap_or("Unknown")),
            });
        }

        let parsed_response: GroqResponse = response.json().map_err(|e| {
            PrForgeError::ApiParseError(format!(
                "Failed to parse response: {}",
                e
            ))
        })?;

        Ok(parsed_response)
    }

    /// Check if error is retryable
    fn is_retryable_error(&self, error: &PrForgeError) -> bool {
        matches!(
            error,
            PrForgeError::Timeout { .. }
                | PrForgeError::RateLimited { .. }
                | PrForgeError::NetworkError(_)
        )
    }

    /// Calculate exponential backoff with jitter
    fn calculate_backoff(&self, base_ms: u64, max_ms: u64, jitter_factor: f64) -> u64 {
        let jitter_range = (base_ms as f64 * jitter_factor) as u64;
        let jitter = rand::random::<u64>() % (jitter_range * 2) - jitter_range;
        ((base_ms as i128 + jitter as i128).max(0) as u64).min(max_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groq_response_validate_success() {
        let response = GroqResponse {
            choices: vec![Choice {
                message: ResponseMessage {
                    content: Some("Test content".to_string()),
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: None,
            error: None,
        };

        assert_eq!(response.validate().unwrap(), "Test content");
    }

    #[test]
    fn test_groq_response_validate_empty_content() {
        let response = GroqResponse {
            choices: vec![Choice {
                message: ResponseMessage { content: None },
                finish_reason: Some("stop".to_string()),
            }],
            usage: None,
            error: None,
        };

        assert!(response.validate().is_err());
    }

    #[test]
    fn test_groq_response_validate_empty_choices() {
        let response = GroqResponse {
            choices: vec![],
            usage: None,
            error: None,
        };

        assert!(response.validate().is_err());
    }

    #[test]
    fn test_groq_response_validate_with_error() {
        let response = GroqResponse {
            choices: vec![],
            usage: None,
            error: Some(ErrorInfo {
                message: "API Error".to_string(),
                code: Some(401),
            }),
        };

        assert!(response.validate().is_err());
    }
}
