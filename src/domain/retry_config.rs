use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for retry behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (0 = no retries)
    pub max_retries: u32,
    /// Base delay between retries (exponential backoff base)
    #[serde(with = "duration_serde", default = "default_base_delay")]
    pub base_delay: Duration,
    /// Maximum delay between retries
    #[serde(with = "duration_serde", default = "default_max_delay")]
    pub max_delay: Duration,
    /// HTTP status codes that should trigger a retry
    #[serde(default = "default_retry_status_codes")]
    pub retry_status_codes: Vec<u16>,
    /// Whether to retry on network errors (timeouts, connection failures)
    #[serde(default = "default_retry_on_network_error")]
    pub retry_on_network_error: bool,
}

fn default_base_delay() -> Duration {
    Duration::from_millis(500)
}

fn default_max_delay() -> Duration {
    Duration::from_secs(30)
}

fn default_retry_status_codes() -> Vec<u16> {
    vec![429, 500, 502, 503, 504]
}

fn default_retry_on_network_error() -> bool {
    true
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: default_base_delay(),
            max_delay: default_max_delay(),
            retry_status_codes: default_retry_status_codes(),
            retry_on_network_error: default_retry_on_network_error(),
        }
    }
}

impl RetryConfig {
    /// Create a new RetryConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a RetryConfig with no retries
    pub fn no_retry() -> Self {
        Self {
            max_retries: 0,
            base_delay: default_base_delay(),
            max_delay: default_max_delay(),
            retry_status_codes: vec![],
            retry_on_network_error: false,
        }
    }

    /// Set the maximum number of retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set the base delay for exponential backoff
    pub fn with_base_delay(mut self, base_delay: Duration) -> Self {
        self.base_delay = base_delay;
        self
    }

    /// Set the maximum delay between retries
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }

    /// Set the HTTP status codes that should trigger a retry
    pub fn with_retry_status_codes(mut self, codes: Vec<u16>) -> Self {
        self.retry_status_codes = codes;
        self
    }

    /// Set whether to retry on network errors
    pub fn with_retry_on_network_error(mut self, retry: bool) -> Self {
        self.retry_on_network_error = retry;
        self
    }

    /// Calculate the delay for a given retry attempt using exponential backoff
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let exponential_delay = self.base_delay.as_millis() as u64 * 2u64.pow(attempt);
        Duration::from_millis(exponential_delay.min(self.max_delay.as_millis() as u64))
    }

    /// Check if a status code should trigger a retry
    pub fn should_retry_status(&self, status_code: u16) -> bool {
        self.retry_status_codes.contains(&status_code)
    }
}

/// Serde module for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_retry_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay, Duration::from_millis(500));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert!(config.retry_status_codes.contains(&429));
        assert!(config.retry_status_codes.contains(&503));
        assert!(config.retry_on_network_error);
    }

    #[test]
    fn test_no_retry_config() {
        let config = RetryConfig::no_retry();
        assert_eq!(config.max_retries, 0);
        assert!(!config.retry_on_network_error);
    }

    #[test]
    fn test_builder_pattern() {
        let config = RetryConfig::new()
            .with_max_retries(5)
            .with_base_delay(Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(60))
            .with_retry_status_codes(vec![500, 502])
            .with_retry_on_network_error(false);

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.base_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(60));
        assert_eq!(config.retry_status_codes, vec![500, 502]);
        assert!(!config.retry_on_network_error);
    }

    #[test]
    fn test_calculate_delay_exponential_backoff() {
        let config = RetryConfig::new()
            .with_base_delay(Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(10));

        // First retry: 100ms * 2^0 = 100ms
        assert_eq!(config.calculate_delay(0), Duration::from_millis(100));
        
        // Second retry: 100ms * 2^1 = 200ms
        assert_eq!(config.calculate_delay(1), Duration::from_millis(200));
        
        // Third retry: 100ms * 2^2 = 400ms
        assert_eq!(config.calculate_delay(2), Duration::from_millis(400));
        
        // Fourth retry: 100ms * 2^3 = 800ms
        assert_eq!(config.calculate_delay(3), Duration::from_millis(800));
    }

    #[test]
    fn test_calculate_delay_respects_max() {
        let config = RetryConfig::new()
            .with_base_delay(Duration::from_secs(1))
            .with_max_delay(Duration::from_secs(5));

        // 1s * 2^10 = 1024s, but capped at 5s
        assert_eq!(config.calculate_delay(10), Duration::from_secs(5));
    }

    #[test]
    fn test_should_retry_status() {
        let config = RetryConfig::default();
        
        assert!(config.should_retry_status(429)); // Too Many Requests
        assert!(config.should_retry_status(500)); // Internal Server Error
        assert!(config.should_retry_status(502)); // Bad Gateway
        assert!(config.should_retry_status(503)); // Service Unavailable
        assert!(config.should_retry_status(504)); // Gateway Timeout
        
        assert!(!config.should_retry_status(200)); // OK
        assert!(!config.should_retry_status(404)); // Not Found
        assert!(!config.should_retry_status(401)); // Unauthorized
    }
}
