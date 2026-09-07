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
    /// Whether to respect an integer-seconds `Retry-After` response header.
    #[serde(default = "default_respect_retry_after")]
    pub respect_retry_after: bool,
    /// Maximum positive jitter as a percentage of the exponential delay.
    #[serde(default = "default_jitter_percent")]
    pub jitter_percent: u8,
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

fn default_respect_retry_after() -> bool {
    true
}

fn default_jitter_percent() -> u8 {
    20
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: default_base_delay(),
            max_delay: default_max_delay(),
            retry_status_codes: default_retry_status_codes(),
            retry_on_network_error: default_retry_on_network_error(),
            respect_retry_after: default_respect_retry_after(),
            jitter_percent: default_jitter_percent(),
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
            respect_retry_after: false,
            jitter_percent: 0,
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

    /// Enable or disable support for an integer-seconds `Retry-After` header.
    pub fn with_respect_retry_after(mut self, respect: bool) -> Self {
        self.respect_retry_after = respect;
        self
    }

    /// Set positive delay jitter from 0 through 100 percent.
    pub fn with_jitter_percent(mut self, percent: u8) -> Result<Self, crate::CallerError> {
        if percent > 100 {
            return Err(crate::CallerError::config_error(
                "Retry jitter_percent cannot exceed 100",
            ));
        }
        self.jitter_percent = percent;
        Ok(self)
    }

    /// Validate retry status codes and jitter bounds.
    pub fn validate(&self) -> Result<(), crate::CallerError> {
        if self.jitter_percent > 100 {
            return Err(crate::CallerError::config_error(
                "Retry jitter_percent cannot exceed 100",
            ));
        }
        if self
            .retry_status_codes
            .iter()
            .any(|status| !(100..=599).contains(status))
        {
            return Err(crate::CallerError::config_error(
                "Retry status codes must be between 100 and 599",
            ));
        }
        Ok(())
    }

    /// Calculate the delay for a given retry attempt using exponential backoff
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_millis = u64::try_from(self.base_delay.as_millis()).unwrap_or(u64::MAX);
        let max_millis = u64::try_from(self.max_delay.as_millis()).unwrap_or(u64::MAX);
        let multiplier = 2u64.checked_pow(attempt).unwrap_or(u64::MAX);
        let delay_millis = base_millis.saturating_mul(multiplier).min(max_millis);
        Duration::from_millis(add_jitter(delay_millis, max_millis, self.jitter_percent))
    }

    /// Check if a status code should trigger a retry
    pub fn should_retry_status(&self, status_code: u16) -> bool {
        self.retry_status_codes.contains(&status_code)
    }

    pub(crate) fn calculate_response_delay(
        &self,
        headers: &reqwest::header::HeaderMap,
        attempt: u32,
    ) -> Duration {
        if self.respect_retry_after
            && let Some(seconds) = headers
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.trim().parse::<u64>().ok())
        {
            return Duration::from_secs(seconds).min(self.max_delay);
        }

        self.calculate_delay(attempt)
    }
}

fn add_jitter(delay_millis: u64, max_millis: u64, jitter_percent: u8) -> u64 {
    if delay_millis == 0 || jitter_percent == 0 || delay_millis >= max_millis {
        return delay_millis;
    }

    static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let sequence = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;
    let max_jitter = delay_millis.saturating_mul(jitter_percent as u64) / 100;
    let jitter = (nanos ^ sequence) % max_jitter.saturating_add(1);
    delay_millis.saturating_add(jitter).min(max_millis)
}

/// Serde module for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub(super) fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
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
        assert!(config.respect_retry_after);
        assert_eq!(config.jitter_percent, 20);
    }

    #[test]
    fn test_no_retry_config() {
        let config = RetryConfig::no_retry();
        assert_eq!(config.max_retries, 0);
        assert!(!config.retry_on_network_error);
        assert!(!config.respect_retry_after);
        assert_eq!(config.jitter_percent, 0);
    }

    #[test]
    fn test_builder_pattern() {
        let config = RetryConfig::new()
            .with_max_retries(5)
            .with_base_delay(Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(60))
            .with_retry_status_codes(vec![500, 502])
            .with_retry_on_network_error(false)
            .with_respect_retry_after(false)
            .with_jitter_percent(0)
            .unwrap();

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.base_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(60));
        assert_eq!(config.retry_status_codes, vec![500, 502]);
        assert!(!config.retry_on_network_error);
        assert!(!config.respect_retry_after);
        assert_eq!(config.jitter_percent, 0);
    }

    #[test]
    fn test_calculate_delay_exponential_backoff() {
        let config = RetryConfig::new()
            .with_base_delay(Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(10))
            .with_jitter_percent(0)
            .unwrap();

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
            .with_max_delay(Duration::from_secs(5))
            .with_jitter_percent(0)
            .unwrap();

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
        assert!(config.should_retry_status(504)); // Gateway timeout

        assert!(!config.should_retry_status(200)); // OK
        assert!(!config.should_retry_status(404)); // Not Found
        assert!(!config.should_retry_status(401)); // Unauthorized
    }

    #[test]
    fn response_delay_respects_retry_after_and_maximum() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::RETRY_AFTER, "120".parse().unwrap());
        let config = RetryConfig::new().with_max_delay(Duration::from_secs(10));

        assert_eq!(
            config.calculate_response_delay(&headers, 0),
            Duration::from_secs(10)
        );
    }

    #[test]
    fn jitter_is_bounded_and_invalid_values_fail() {
        let config = RetryConfig::new()
            .with_base_delay(Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(1))
            .with_jitter_percent(20)
            .unwrap();
        let delay = config.calculate_delay(0);
        assert!((Duration::from_millis(100)..=Duration::from_millis(120)).contains(&delay));

        assert!(RetryConfig::new().with_jitter_percent(101).is_err());
        assert!(
            RetryConfig::new()
                .with_retry_status_codes(vec![999])
                .validate()
                .is_err()
        );
    }
}
