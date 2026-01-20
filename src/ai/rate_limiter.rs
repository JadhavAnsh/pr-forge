#![allow(dead_code)]

use crate::error::{PrForgeError, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Configuration for rate limiting
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: usize,
    pub enable_tracking: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 10, // Conservative default for free tier
            enable_tracking: true,
        }
    }
}

/// Rate limiter with sliding window tracking
#[derive(Debug, Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    request_history: Arc<Mutex<Vec<Instant>>>,
}

impl RateLimiter {
    /// Create new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            request_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Check if request should be allowed, returns wait time if rate limited
    pub fn check_rate_limit(&self) -> Result<Duration> {
        if !self.config.enable_tracking {
            return Ok(Duration::ZERO);
        }

        let mut history = self.request_history.lock().unwrap();
        let now = Instant::now();

        // Remove requests older than 1 minute
        history.retain(|instant| now.duration_since(*instant) < Duration::from_secs(60));

        if history.len() >= self.config.requests_per_minute {
            // Calculate wait time until oldest request expires
            if let Some(&oldest) = history.first() {
                let elapsed = now.duration_since(oldest);
                let wait_time = Duration::from_secs(60).saturating_sub(elapsed);
                return Err(PrForgeError::RateLimited {
                    retry_after_secs: wait_time.as_secs(),
                });
            }
        }

        // Record this request
        history.push(now);

        Ok(Duration::ZERO)
    }

    /// Get current request count in the window
    pub fn current_requests(&self) -> usize {
        let history = self.request_history.lock().unwrap();
        let now = Instant::now();
        history
            .iter()
            .filter(|instant| now.duration_since(**instant) < Duration::from_secs(60))
            .count()
    }

    /// Reset rate limiter
    pub fn reset(&self) {
        let mut history = self.request_history.lock().unwrap();
        history.clear();
    }
}

/// Usage statistics tracker
#[derive(Debug, Clone)]
pub struct UsageStats {
    pub requests_today: usize,
    pub tokens_used: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

/// Cache for Groq responses based on PR branch pair
#[derive(Debug, Clone)]
pub struct ResponseCache {
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    content: String,
    timestamp: Instant,
    ttl_secs: u64,
}

impl ResponseCache {
    /// Create new cache
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Generate cache key from PR analysis parameters
    pub fn generate_key(branch: &str, base_branch: &str, commit_hash: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("{}:{}", branch, base_branch).hash(&mut hasher);
        commit_hash.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Try to get cached response
    pub fn get(&self, key: &str) -> Option<String> {
        let mut cache = self.cache.lock().unwrap();

        if let Some(entry) = cache.get(key) {
            let elapsed = entry.timestamp.elapsed();
            if elapsed < Duration::from_secs(entry.ttl_secs) {
                return Some(entry.content.clone());
            } else {
                // Remove expired entry
                cache.remove(key);
            }
        }

        None
    }

    /// Store response in cache
    pub fn set(&self, key: String, content: String, ttl_secs: u64) {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(
            key,
            CacheEntry {
                content,
                timestamp: Instant::now(),
                ttl_secs,
            },
        );
    }

    /// Clear entire cache
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_requests_within_limit() {
        let config = RateLimitConfig {
            requests_per_minute: 5,
            enable_tracking: true,
        };
        let limiter = RateLimiter::new(config);

        for i in 0..5 {
            assert!(
                limiter.check_rate_limit().is_ok(),
                "Request {} should be allowed",
                i + 1
            );
        }
    }

    #[test]
    fn test_rate_limiter_rejects_excess_requests() {
        let config = RateLimitConfig {
            requests_per_minute: 2,
            enable_tracking: true,
        };
        let limiter = RateLimiter::new(config);

        assert!(limiter.check_rate_limit().is_ok());
        assert!(limiter.check_rate_limit().is_ok());
        assert!(limiter.check_rate_limit().is_err()); // Should be rate limited
    }

    #[test]
    fn test_cache_stores_and_retrieves() {
        let cache = ResponseCache::new();
        let key = "test_key".to_string();
        let content = "test_content".to_string();

        cache.set(key.clone(), content.clone(), 60);
        assert_eq!(cache.get(&key), Some(content));
    }

    #[test]
    fn test_cache_expires_old_entries() {
        let cache = ResponseCache::new();
        let key = "test_key".to_string();

        cache.set(key.clone(), "content".to_string(), 0); // 0 second TTL
        std::thread::sleep(Duration::from_millis(10));

        // Entry should be expired
        assert_eq!(cache.get(&key), None);
    }
}
