use crate::shared::error::CallerError;
use async_trait::async_trait;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Request context containing all information about an outgoing request
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// The HTTP method (GET, POST, etc.)
    pub method: String,
    /// The full URL being requested
    pub url: String,
    /// Request headers
    pub headers: HeaderMap,
    /// Request parameters (if any)
    pub params: Option<HashMap<String, String>>,
    /// Request body (if any, as JSON string)
    pub body: Option<String>,
    /// Custom metadata that can be set by middleware
    pub metadata: HashMap<String, String>,
}

impl RequestContext {
    pub fn new(method: &str, url: &str) -> Self {
        Self {
            method: method.to_string(),
            url: url.to_string(),
            headers: HeaderMap::new(),
            params: None,
            body: None,
            metadata: HashMap::new(),
        }
    }

    /// Add a header to the request
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(key.as_bytes())
            && let Ok(header_value) = reqwest::header::HeaderValue::from_str(value)
        {
            self.headers.insert(header_name, header_value);
        }
        self
    }

    /// Set the request body
    pub fn with_body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    /// Set request parameters
    pub fn with_params(mut self, params: HashMap<String, String>) -> Self {
        self.params = Some(params);
        self
    }

    /// Add custom metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Response context containing all information about a received response
#[derive(Debug, Clone)]
pub struct ResponseContext {
    /// HTTP status code
    pub status_code: u16,
    /// Response headers
    pub headers: HeaderMap,
    /// Response body as string
    pub body: String,
    /// Request that generated this response
    pub request: RequestContext,
    /// Time taken for the request (in milliseconds)
    pub duration_ms: u64,
}

impl ResponseContext {
    pub fn new(status_code: u16, body: String, request: RequestContext, duration_ms: u64) -> Self {
        Self {
            status_code,
            headers: HeaderMap::new(),
            body,
            request,
            duration_ms,
        }
    }

    /// Check if the response was successful (2xx status code)
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }

    /// Check if the response was a client error (4xx status code)
    pub fn is_client_error(&self) -> bool {
        self.status_code >= 400 && self.status_code < 500
    }

    /// Check if the response was a server error (5xx status code)
    pub fn is_server_error(&self) -> bool {
        self.status_code >= 500 && self.status_code < 600
    }
}

/// Trait for implementing request/response middleware
///
/// Middleware can intercept and modify requests before they are sent
/// and responses after they are received.
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Called before the request is sent
    /// Can modify the request context or return an error to abort
    async fn before_request(&self, ctx: &mut RequestContext) -> Result<(), CallerError> {
        let _ = ctx; // Default: do nothing
        Ok(())
    }

    /// Called after the response is received
    /// Can modify the response context or return an error
    async fn after_response(&self, ctx: &mut ResponseContext) -> Result<(), CallerError> {
        let _ = ctx; // Default: do nothing
        Ok(())
    }

    /// Called when an error occurs during the request
    async fn on_error(&self, error: &CallerError, ctx: &RequestContext) {
        let _ = (error, ctx); // Default: do nothing
    }

    /// Get the name of this middleware (for debugging/logging)
    fn name(&self) -> &str {
        "unnamed"
    }
}

/// A collection of middleware that are executed in order
#[derive(Default, Clone)]
pub struct MiddlewareChain {
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl MiddlewareChain {
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    /// Add a middleware to the chain (builder pattern)
    pub fn with<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middlewares.push(Arc::new(middleware));
        self
    }

    /// Add a middleware wrapped in Arc (builder pattern)
    pub fn with_arc(mut self, middleware: Arc<dyn Middleware>) -> Self {
        self.middlewares.push(middleware);
        self
    }

    /// Execute all before_request middleware in order
    pub async fn before_request(&self, ctx: &mut RequestContext) -> Result<(), CallerError> {
        for middleware in &self.middlewares {
            middleware.before_request(ctx).await?;
        }
        Ok(())
    }

    /// Execute all after_response middleware in order
    pub async fn after_response(&self, ctx: &mut ResponseContext) -> Result<(), CallerError> {
        for middleware in &self.middlewares {
            middleware.after_response(ctx).await?;
        }
        Ok(())
    }

    /// Execute all on_error middleware in order
    pub async fn on_error(&self, error: &CallerError, ctx: &RequestContext) {
        for middleware in &self.middlewares {
            middleware.on_error(error, ctx).await;
        }
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.middlewares.is_empty()
    }

    /// Get the number of middleware in the chain
    pub fn len(&self) -> usize {
        self.middlewares.len()
    }
}

// ============================================================================
// Built-in Middleware Implementations
// ============================================================================

/// Middleware that adds custom headers to all requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderMiddleware {
    headers: HashMap<String, String>,
}

impl HeaderMiddleware {
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
}

impl Default for HeaderMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for HeaderMiddleware {
    async fn before_request(&self, ctx: &mut RequestContext) -> Result<(), CallerError> {
        for (key, value) in &self.headers {
            if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                && let Ok(header_value) = reqwest::header::HeaderValue::from_str(value)
            {
                ctx.headers.insert(header_name, header_value);
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "HeaderMiddleware"
    }
}

/// Middleware that logs requests and responses
#[derive(Debug, Clone)]
pub struct LoggingMiddleware {
    /// Whether to log request details
    pub log_request: bool,
    /// Whether to log response details
    pub log_response: bool,
    /// Whether to log errors
    pub log_errors: bool,
}

impl LoggingMiddleware {
    pub fn new() -> Self {
        Self {
            log_request: true,
            log_response: true,
            log_errors: true,
        }
    }

    pub fn with_log_request(mut self, enabled: bool) -> Self {
        self.log_request = enabled;
        self
    }

    pub fn with_log_response(mut self, enabled: bool) -> Self {
        self.log_response = enabled;
        self
    }
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn before_request(&self, ctx: &mut RequestContext) -> Result<(), CallerError> {
        if self.log_request {
            println!("[Request] {} {}", ctx.method, ctx.url);
        }
        Ok(())
    }

    async fn after_response(&self, ctx: &mut ResponseContext) -> Result<(), CallerError> {
        if self.log_response {
            println!(
                "[Response] {} {} - {} ({}ms)",
                ctx.request.method, ctx.request.url, ctx.status_code, ctx.duration_ms
            );
        }
        Ok(())
    }

    async fn on_error(&self, error: &CallerError, ctx: &RequestContext) {
        if self.log_errors {
            eprintln!("[Error] {} {}: {}", ctx.method, ctx.url, error);
        }
    }

    fn name(&self) -> &str {
        "LoggingMiddleware"
    }
}

/// Middleware that adds timing/tracing information
#[derive(Debug, Clone, Default)]
pub struct TimingMiddleware {
    /// Header name to store request start time
    pub timing_header: Option<String>,
}

impl TimingMiddleware {
    pub fn new() -> Self {
        Self { timing_header: None }
    }

    pub fn with_timing_header(mut self, header_name: &str) -> Self {
        self.timing_header = Some(header_name.to_string());
        self
    }
}

#[async_trait]
impl Middleware for TimingMiddleware {
    async fn before_request(&self, ctx: &mut RequestContext) -> Result<(), CallerError> {
        if let Some(ref header) = self.timing_header {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
                .to_string();
            ctx.metadata.insert(header.clone(), timestamp);
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "TimingMiddleware"
    }
}

/// Middleware that adds a User-Agent header
#[derive(Debug, Clone)]
pub struct UserAgentMiddleware {
    user_agent: String,
}

impl UserAgentMiddleware {
    pub fn new(user_agent: &str) -> Self {
        Self {
            user_agent: user_agent.to_string(),
        }
    }
}

#[async_trait]
impl Middleware for UserAgentMiddleware {
    async fn before_request(&self, ctx: &mut RequestContext) -> Result<(), CallerError> {
        if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(b"user-agent")
            && let Ok(header_value) = reqwest::header::HeaderValue::from_str(&self.user_agent)
        {
            ctx.headers.insert(header_name, header_value);
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "UserAgentMiddleware"
    }
}

/// Middleware that retries requests on specific status codes
#[derive(Debug, Clone)]
pub struct RetryMiddleware {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Status codes that should trigger a retry
    pub retry_status_codes: Vec<u16>,
    /// Base delay between retries (in milliseconds)
    pub base_delay_ms: u64,
}

impl RetryMiddleware {
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            retry_status_codes: vec![429, 500, 502, 503, 504],
            base_delay_ms: 500,
        }
    }

    pub fn with_max_retries(mut self, max: u32) -> Self {
        self.max_retries = max;
        self
    }

    pub fn with_status_codes(mut self, codes: Vec<u16>) -> Self {
        self.retry_status_codes = codes;
        self
    }
}

impl Default for RetryMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for RetryMiddleware {
    fn name(&self) -> &str {
        "RetryMiddleware"
    }
}

// ============================================================================
// Circuit Breaker Middleware
// ============================================================================

/// State of the circuit breaker
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    #[default]
    Closed,
    /// Circuit is open, requests are blocked
    Open,
    /// Circuit is half-open, allowing a test request
    HalfOpen,
}

/// Circuit Breaker middleware that prevents cascading failures
///
/// The circuit breaker pattern prevents an application from repeatedly trying
/// to execute an operation that's likely to fail. It allows the system to fail
/// fast and recover gracefully.
///
/// # States
/// - **Closed**: Normal operation, requests pass through. Failures are counted.
/// - **Open**: Circuit is tripped, requests fail immediately without calling the service.
/// - **Half-Open**: After timeout, allows a single test request to check if service recovered.
///
/// # Example
/// ```rust
/// use caller::domain::middleware::CircuitBreakerMiddleware;
///
/// let breaker = CircuitBreakerMiddleware::new()
///     .with_failure_threshold(5)           // Open after 5 consecutive failures
///     .with_success_threshold(3)           // Close after 3 consecutive successes
///     .with_timeout_ms(30_000);            // Try half-open after 30 seconds
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerMiddleware {
    /// Name identifier for this circuit breaker (for logging/debugging)
    pub name: String,
    /// Number of consecutive failures before opening the circuit
    pub failure_threshold: u32,
    /// Number of consecutive successes in half-open state before closing
    pub success_threshold: u32,
    /// Time in milliseconds before attempting to close (transition to half-open)
    pub timeout_ms: u64,
    /// Current state of the circuit breaker
    #[serde(skip)]
    pub state: CircuitState,
    /// Current failure count (resets on success)
    #[serde(skip)]
    pub failure_count: u32,
    /// Current success count in half-open state
    #[serde(skip)]
    pub success_count: u32,
    /// Timestamp when the circuit was opened (for timeout calculation)
    #[serde(skip)]
    pub opened_at: Option<std::time::Instant>,
}

impl CircuitBreakerMiddleware {
    /// Create a new circuit breaker with default settings
    pub fn new() -> Self {
        Self {
            name: "default".to_string(),
            failure_threshold: 5,
            success_threshold: 2,
            timeout_ms: 30_000, // 30 seconds
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            opened_at: None,
        }
    }

    /// Set the name of this circuit breaker
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// Set the number of failures before opening the circuit
    pub fn with_failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// Set the number of successes needed to close the circuit from half-open
    pub fn with_success_threshold(mut self, threshold: u32) -> Self {
        self.success_threshold = threshold;
        self
    }

    /// Set the timeout before transitioning from open to half-open
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Check if a request should be allowed
    pub fn allow_request(&mut self) -> Result<(), CallerError> {
        match self.state {
            CircuitState::Closed => Ok(()),
            CircuitState::Open => {
                // Check if timeout has elapsed
                if let Some(opened_at) = self.opened_at {
                    let elapsed = opened_at.elapsed().as_millis() as u64;
                    if elapsed >= self.timeout_ms {
                        // Transition to half-open
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        return Ok(());
                    }
                }
                Err(CallerError::HttpError(format!(
                    "Circuit breaker '{}' is open - service unavailable",
                    self.name
                )))
            }
            CircuitState::HalfOpen => Ok(()),
        }
    }

    /// Record a successful request
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    // Close the circuit
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                    self.opened_at = None;
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but reset to closed on success
                self.state = CircuitState::Closed;
                self.failure_count = 0;
                self.opened_at = None;
            }
        }
    }

    /// Record a failed request
    pub fn record_failure(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    // Open the circuit
                    self.state = CircuitState::Open;
                    self.opened_at = Some(std::time::Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                // Failure in half-open state immediately opens the circuit again
                self.state = CircuitState::Open;
                self.opened_at = Some(std::time::Instant::now());
                self.success_count = 0;
            }
            CircuitState::Open => {
                // Already open, just update the timestamp
                self.opened_at = Some(std::time::Instant::now());
            }
        }
    }

    /// Get the current state
    pub fn state(&self) -> CircuitState {
        self.state
    }

    /// Reset the circuit breaker to closed state
    pub fn reset(&mut self) {
        self.state = CircuitState::Closed;
        self.failure_count = 0;
        self.success_count = 0;
        self.opened_at = None;
    }

    /// Check if the circuit is currently open
    pub fn is_open(&self) -> bool {
        self.state == CircuitState::Open
    }

    /// Check if the circuit is currently closed
    pub fn is_closed(&self) -> bool {
        self.state == CircuitState::Closed
    }
}

impl Default for CircuitBreakerMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for CircuitBreakerMiddleware {
    async fn before_request(&self, ctx: &mut RequestContext) -> Result<(), CallerError> {
        // Note: This requires interior mutability for proper implementation
        // For now, we just add metadata about the circuit breaker state
        ctx.metadata.insert(
            format!("circuit_breaker_{}_state", self.name),
            format!("{:?}", self.state),
        );
        Ok(())
    }

    async fn after_response(&self, ctx: &mut ResponseContext) -> Result<(), CallerError> {
        // Record success/failure based on status code
        if ctx.is_success() {
            // Success - could reset failure count
            ctx.request.metadata.insert(
                format!("circuit_breaker_{}_result", self.name),
                "success".to_string(),
            );
        } else if ctx.is_server_error() {
            // Server error - could increment failure count
            ctx.request.metadata.insert(
                format!("circuit_breaker_{}_result", self.name),
                "failure".to_string(),
            );
        }
        Ok(())
    }

    async fn on_error(&self, error: &CallerError, ctx: &RequestContext) {
        // Log the error for circuit breaker tracking
        let _ = (error, ctx);
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ============================================================================
// Rate Limiting Middleware
// ============================================================================

/// Rate limiting strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateLimitStrategy {
    /// Fixed window rate limiting
    FixedWindow,
    /// Sliding window rate limiting (more accurate but uses more memory)
    SlidingWindow,
    /// Token bucket algorithm
    TokenBucket,
}

/// Rate Limiting middleware to control request frequency
///
/// Prevents overwhelming external services by limiting the rate of requests.
///
/// # Example
/// ```rust
/// use caller::domain::middleware::RateLimitMiddleware;
/// use std::time::Duration;
///
/// let limiter = RateLimitMiddleware::new()
///     .with_max_requests(100)              // Max 100 requests
///     .with_window_secs(60)                // Per 60 seconds
///     .with_burst(10);                     // Allow burst of 10
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitMiddleware {
    /// Maximum requests allowed in the window
    pub max_requests: u32,
    /// Window duration in seconds
    pub window_secs: u64,
    /// Burst size (for token bucket)
    pub burst_size: u32,
    /// Rate limiting strategy
    pub strategy: RateLimitStrategy,
    /// Current request count in window
    #[serde(skip)]
    pub request_count: u32,
    /// Window start time
    #[serde(skip)]
    pub window_start: Option<std::time::Instant>,
    /// Tokens available (for token bucket)
    #[serde(skip)]
    pub tokens: f64,
    /// Last token refill time
    #[serde(skip)]
    pub last_refill: Option<std::time::Instant>,
}

impl RateLimitMiddleware {
    /// Create a new rate limiter with default settings (100 requests per minute)
    pub fn new() -> Self {
        Self {
            max_requests: 100,
            window_secs: 60,
            burst_size: 10,
            strategy: RateLimitStrategy::FixedWindow,
            request_count: 0,
            window_start: None,
            tokens: 10.0,
            last_refill: None,
        }
    }

    /// Set the maximum requests allowed
    pub fn with_max_requests(mut self, max: u32) -> Self {
        self.max_requests = max;
        self
    }

    /// Set the window duration in seconds
    pub fn with_window_secs(mut self, secs: u64) -> Self {
        self.window_secs = secs;
        self
    }

    /// Set the burst size
    pub fn with_burst(mut self, burst: u32) -> Self {
        self.burst_size = burst;
        self.tokens = burst as f64;
        self
    }

    /// Set the rate limiting strategy
    pub fn with_strategy(mut self, strategy: RateLimitStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Check if a request is allowed
    pub fn allow_request(&mut self) -> Result<(), CallerError> {
        match self.strategy {
            RateLimitStrategy::FixedWindow => self.check_fixed_window(),
            RateLimitStrategy::SlidingWindow => self.check_sliding_window(),
            RateLimitStrategy::TokenBucket => self.check_token_bucket(),
        }
    }

    fn check_fixed_window(&mut self) -> Result<(), CallerError> {
        let now = std::time::Instant::now();

        // Initialize or reset window if expired
        if self.window_start.is_none() || now.duration_since(self.window_start.unwrap()).as_secs() >= self.window_secs {
            self.window_start = Some(now);
            self.request_count = 0;
        }

        if self.request_count >= self.max_requests {
            let elapsed = now.duration_since(self.window_start.unwrap()).as_secs();
            let remaining = self.window_secs - elapsed;
            return Err(CallerError::HttpError(format!(
                "Rate limit exceeded. Try again in {} seconds",
                remaining
            )));
        }

        self.request_count += 1;
        Ok(())
    }

    fn check_sliding_window(&mut self) -> Result<(), CallerError> {
        // For simplicity, use fixed window for now
        // A proper sliding window would require tracking individual request timestamps
        self.check_fixed_window()
    }

    fn check_token_bucket(&mut self) -> Result<(), CallerError> {
        let now = std::time::Instant::now();

        // Refill tokens based on elapsed time
        if let Some(last) = self.last_refill {
            let elapsed = now.duration_since(last).as_secs_f64();
            let refill_rate = self.max_requests as f64 / self.window_secs as f64;
            self.tokens = (self.tokens + elapsed * refill_rate).min(self.burst_size as f64);
        }

        self.last_refill = Some(now);

        if self.tokens < 1.0 {
            let wait_time = (1.0 - self.tokens) * self.window_secs as f64 / self.max_requests as f64;
            return Err(CallerError::HttpError(format!(
                "Rate limit exceeded. Try again in {:.1} seconds",
                wait_time
            )));
        }

        self.tokens -= 1.0;
        Ok(())
    }

    /// Get remaining requests in current window
    pub fn remaining(&self) -> u32 {
        match self.strategy {
            RateLimitStrategy::TokenBucket => self.tokens as u32,
            _ => self.max_requests.saturating_sub(self.request_count),
        }
    }

    /// Reset the rate limiter
    pub fn reset(&mut self) {
        self.request_count = 0;
        self.window_start = None;
        self.tokens = self.burst_size as f64;
        self.last_refill = None;
    }
}

impl Default for RateLimitMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn before_request(&self, ctx: &mut RequestContext) -> Result<(), CallerError> {
        ctx.metadata.insert(
            "rate_limit_remaining".to_string(),
            self.remaining().to_string(),
        );
        Ok(())
    }

    fn name(&self) -> &str {
        "RateLimitMiddleware"
    }
}

// ============================================================================
// Request ID Middleware
// ============================================================================

/// Strategy for generating request IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestIdStrategy {
    /// UUID v4 (random)
    Uuid,
    /// Timestamp-based ID (nanoseconds since epoch)
    Timestamp,
    /// Short random ID (8 characters, base62)
    ShortRandom,
    /// Prefixed sequential counter (not thread-safe, for single-threaded use only)
    PrefixedCounter,
}

/// Request ID middleware that adds a unique identifier to each request
///
/// This middleware automatically generates and attaches a unique request ID
/// to every outgoing request. The ID can be added as a header and/or stored
/// in the request context metadata for logging and tracing purposes.
///
/// # Example
/// ```rust
/// use caller::domain::middleware::{RequestIdMiddleware, RequestIdStrategy};
///
/// // Create with default settings (UUID, X-Request-Id header)
/// let middleware = RequestIdMiddleware::new();
///
/// // Create with custom header and strategy
/// let middleware = RequestIdMiddleware::new()
///     .with_header_name("X-Trace-Id")
///     .with_strategy(RequestIdStrategy::ShortRandom)
///     .with_prefix("myapp");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestIdMiddleware {
    /// Header name to use for the request ID
    pub header_name: String,
    /// Strategy for generating IDs
    pub strategy: RequestIdStrategy,
    /// Optional prefix for the ID (e.g., "myapp-")
    pub prefix: Option<String>,
    /// Whether to also store the ID in context metadata
    pub store_in_metadata: bool,
    /// Metadata key name (if storing in metadata)
    pub metadata_key: String,
}

impl RequestIdMiddleware {
    /// Create a new request ID middleware with default settings
    pub fn new() -> Self {
        Self {
            header_name: "X-Request-Id".to_string(),
            strategy: RequestIdStrategy::Uuid,
            prefix: None,
            store_in_metadata: true,
            metadata_key: "request_id".to_string(),
        }
    }

    /// Set the header name for the request ID
    pub fn with_header_name(mut self, name: &str) -> Self {
        self.header_name = name.to_string();
        self
    }

    /// Set the ID generation strategy
    pub fn with_strategy(mut self, strategy: RequestIdStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set a prefix for generated IDs
    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    /// Set whether to store the ID in request metadata
    pub fn with_store_in_metadata(mut self, store: bool) -> Self {
        self.store_in_metadata = store;
        self
    }

    /// Set the metadata key name
    pub fn with_metadata_key(mut self, key: &str) -> Self {
        self.metadata_key = key.to_string();
        self
    }

    /// Generate a unique request ID based on the configured strategy
    pub fn generate_id(&self) -> String {
        let id = match self.strategy {
            RequestIdStrategy::Uuid => self.generate_uuid(),
            RequestIdStrategy::Timestamp => self.generate_timestamp(),
            RequestIdStrategy::ShortRandom => self.generate_short_random(),
            RequestIdStrategy::PrefixedCounter => self.generate_counter(),
        };

        match &self.prefix {
            Some(prefix) => format!("{}-{}", prefix, id),
            None => id,
        }
    }

    fn generate_uuid(&self) -> String {
        // Simple UUID v4-like generation without external dependencies
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        
        // Use timestamp and random data to create a UUID-like string
        let secs = timestamp.as_secs();
        let nanos = timestamp.subsec_nanos();
        
        // Format as UUID v4: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx (exactly 36 chars)
        // y is 8, 9, a, or b (variant bits)
        let time_low = (secs & 0xFFFFFFFF) as u32;
        let time_mid = ((secs >> 32) & 0xFFFF) as u16;
        let time_hi = ((nanos >> 16) & 0x0FFF) as u16;
        let variant = (nanos & 0x3FFF) as u16 | 0x8000;
        let node = (nanos as u64) | ((secs >> 48) & 0xFFFF) << 32;
        
        format!(
            "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
            time_low,
            time_mid,
            time_hi,
            variant,
            node & 0xFFFFFFFFFFFF // Ensure exactly 12 hex digits
        )
    }

    fn generate_timestamp(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        
        // Nanosecond precision timestamp
        format!("{}{}", timestamp.as_secs(), timestamp.subsec_nanos())
    }

    fn generate_short_random(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        
        // Generate 8-character base62-like string
        let mut value = timestamp.as_nanos() as u64;
        let mut value2 = timestamp.subsec_nanos() as u64;
        value = value.wrapping_add(value2.rotate_left(17));
        
        const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        let mut result = [0u8; 8];
        
        for item in result.iter_mut() {
            *item = BASE62[(value % 62) as usize];
            value /= 62;
            if value == 0 {
                value = value2;
                value2 = value2.wrapping_mul(6364136223846793005);
            }
        }
        
        String::from_utf8_lossy(&result).to_string()
    }

    fn generate_counter(&self) -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("{:06x}", count)
    }
}

impl Default for RequestIdMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for RequestIdMiddleware {
    async fn before_request(&self, ctx: &mut RequestContext) -> Result<(), CallerError> {
        let request_id = self.generate_id();
        
        // Add to headers
        if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(self.header_name.as_bytes())
            && let Ok(header_value) = reqwest::header::HeaderValue::from_str(&request_id)
        {
            ctx.headers.insert(header_name, header_value);
        }
        
        // Store in metadata if configured
        if self.store_in_metadata {
            ctx.metadata.insert(self.metadata_key.clone(), request_id);
        }
        
        Ok(())
    }

    fn name(&self) -> &str {
        "RequestIdMiddleware"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_context_builder() {
        let ctx = RequestContext::new("GET", "https://example.com/api")
            .with_header("X-Custom", "value")
            .with_metadata("trace_id", "123");

        assert_eq!(ctx.method, "GET");
        assert_eq!(ctx.url, "https://example.com/api");
        assert_eq!(ctx.metadata.get("trace_id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_response_context_status_checks() {
        let req_ctx = RequestContext::new("GET", "https://example.com");

        let success_resp = ResponseContext::new(200, "ok".to_string(), req_ctx.clone(), 100);
        assert!(success_resp.is_success());
        assert!(!success_resp.is_client_error());
        assert!(!success_resp.is_server_error());

        let client_err = ResponseContext::new(404, "not found".to_string(), req_ctx.clone(), 50);
        assert!(!client_err.is_success());
        assert!(client_err.is_client_error());
        assert!(!client_err.is_server_error());

        let server_err = ResponseContext::new(500, "error".to_string(), req_ctx, 150);
        assert!(!server_err.is_success());
        assert!(!server_err.is_client_error());
        assert!(server_err.is_server_error());
    }

    #[test]
    fn test_middleware_chain() {
        let chain = MiddlewareChain::new()
            .with(HeaderMiddleware::new().with_header("X-Test", "1"))
            .with(LoggingMiddleware::new());

        assert_eq!(chain.len(), 2);
        assert!(!chain.is_empty());
    }

    #[test]
    fn test_header_middleware() {
        let middleware = HeaderMiddleware::new()
            .with_header("X-API-Key", "secret")
            .with_header("X-Request-Id", "123");

        let mut ctx = RequestContext::new("GET", "https://example.com");

        // Use tokio runtime for async test
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            middleware.before_request(&mut ctx).await.unwrap();
        });

        assert!(ctx.headers.contains_key("x-api-key"));
        assert!(ctx.headers.contains_key("x-request-id"));
    }

    #[tokio::test]
    async fn test_middleware_chain_execution() {
        let chain = MiddlewareChain::new()
            .with(HeaderMiddleware::new().with_header("X-First", "1"))
            .with(HeaderMiddleware::new().with_header("X-Second", "2"));

        let mut ctx = RequestContext::new("POST", "https://example.com/api");
        chain.before_request(&mut ctx).await.unwrap();

        assert!(ctx.headers.contains_key("x-first"));
        assert!(ctx.headers.contains_key("x-second"));
    }

    #[test]
    fn test_user_agent_middleware() {
        let middleware = UserAgentMiddleware::new("MyApp/1.0");
        let mut ctx = RequestContext::new("GET", "https://example.com");

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            middleware.before_request(&mut ctx).await.unwrap();
        });

        let ua = ctx.headers.get("user-agent").unwrap();
        assert_eq!(ua, "MyApp/1.0");
    }

    // ============================================================================
    // Circuit Breaker Tests
    // ============================================================================

    #[test]
    fn test_circuit_breaker_default_state() {
        let breaker = CircuitBreakerMiddleware::new();
        assert_eq!(breaker.state(), CircuitState::Closed);
        assert!(breaker.is_closed());
        assert!(!breaker.is_open());
    }

    #[test]
    fn test_circuit_breaker_opens_after_threshold() {
        let mut breaker = CircuitBreakerMiddleware::new()
            .with_failure_threshold(3)
            .with_name("test");

        // Should be closed initially
        assert!(breaker.allow_request().is_ok());

        // Record failures up to threshold
        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Closed);

        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Closed);

        breaker.record_failure(); // Third failure triggers open
        assert_eq!(breaker.state(), CircuitState::Open);
        assert!(breaker.is_open());

        // Should reject requests when open
        assert!(breaker.allow_request().is_err());
    }

    #[test]
    fn test_circuit_breaker_success_resets_failures() {
        let mut breaker = CircuitBreakerMiddleware::new()
            .with_failure_threshold(5);

        // Record some failures
        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.failure_count, 2);

        // Success should reset failure count
        breaker.record_success();
        assert_eq!(breaker.failure_count, 0);
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_reset() {
        let mut breaker = CircuitBreakerMiddleware::new()
            .with_failure_threshold(2);

        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Open);

        breaker.reset();
        assert_eq!(breaker.state(), CircuitState::Closed);
        assert_eq!(breaker.failure_count, 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_middleware() {
        let breaker = CircuitBreakerMiddleware::new()
            .with_name("test_api")
            .with_failure_threshold(5);

        let mut ctx = RequestContext::new("GET", "https://example.com");
        breaker.before_request(&mut ctx).await.unwrap();

        // Check metadata was added
        assert!(ctx.metadata.contains_key("circuit_breaker_test_api_state"));
    }

    // ============================================================================
    // Rate Limiter Tests
    // ============================================================================

    #[test]
    fn test_rate_limiter_default() {
        let limiter = RateLimitMiddleware::new();
        assert_eq!(limiter.max_requests, 100);
        assert_eq!(limiter.window_secs, 60);
    }

    #[test]
    fn test_rate_limiter_allows_under_limit() {
        let mut limiter = RateLimitMiddleware::new()
            .with_max_requests(5)
            .with_window_secs(60);

        // Should allow requests under limit
        for _ in 0..5 {
            assert!(limiter.allow_request().is_ok());
        }

        // Should reject after limit
        assert!(limiter.allow_request().is_err());
    }

    #[test]
    fn test_rate_limiter_remaining() {
        let mut limiter = RateLimitMiddleware::new()
            .with_max_requests(10)
            .with_window_secs(60);

        assert_eq!(limiter.remaining(), 10);

        limiter.allow_request().unwrap();
        assert_eq!(limiter.remaining(), 9);

        limiter.allow_request().unwrap();
        assert_eq!(limiter.remaining(), 8);
    }

    #[test]
    fn test_rate_limiter_reset() {
        let mut limiter = RateLimitMiddleware::new()
            .with_max_requests(2)
            .with_window_secs(60);

        limiter.allow_request().unwrap();
        limiter.allow_request().unwrap();
        assert!(limiter.allow_request().is_err());

        limiter.reset();
        assert_eq!(limiter.remaining(), 2);
        assert!(limiter.allow_request().is_ok());
    }

    #[test]
    fn test_rate_limiter_token_bucket() {
        let mut limiter = RateLimitMiddleware::new()
            .with_max_requests(10)
            .with_window_secs(10)
            .with_burst(5)
            .with_strategy(RateLimitStrategy::TokenBucket);

        // Should allow up to burst size
        for i in 0..5 {
            assert!(limiter.allow_request().is_ok(), "Request {} should succeed", i);
        }

        // After burst is exhausted, should fail
        assert!(limiter.allow_request().is_err());
    }

    // ============================================================================
    // Request ID Middleware Tests
    // ============================================================================

    #[test]
    fn test_request_id_middleware_default() {
        let middleware = RequestIdMiddleware::new();
        assert_eq!(middleware.header_name, "X-Request-Id");
        assert_eq!(middleware.strategy, RequestIdStrategy::Uuid);
        assert!(middleware.store_in_metadata);
        assert!(middleware.prefix.is_none());
    }

    #[test]
    fn test_request_id_middleware_builder() {
        let middleware = RequestIdMiddleware::new()
            .with_header_name("X-Trace-Id")
            .with_strategy(RequestIdStrategy::ShortRandom)
            .with_prefix("myapp")
            .with_metadata_key("trace_id");

        assert_eq!(middleware.header_name, "X-Trace-Id");
        assert_eq!(middleware.strategy, RequestIdStrategy::ShortRandom);
        assert_eq!(middleware.prefix, Some("myapp".to_string()));
        assert_eq!(middleware.metadata_key, "trace_id");
    }

    #[test]
    fn test_request_id_generate_uuid() {
        let middleware = RequestIdMiddleware::new()
            .with_strategy(RequestIdStrategy::Uuid);
        
        let id1 = middleware.generate_id();
        let id2 = middleware.generate_id();
        
        // IDs should be different
        assert_ne!(id1, id2);
        
        // UUID format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx (36 chars)
        assert_eq!(id1.len(), 36);
        assert!(id1.contains('-'));
    }

    #[test]
    fn test_request_id_generate_timestamp() {
        let middleware = RequestIdMiddleware::new()
            .with_strategy(RequestIdStrategy::Timestamp);
        
        let id = middleware.generate_id();
        
        // Timestamp should be numeric
        assert!(id.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_request_id_generate_short_random() {
        let middleware = RequestIdMiddleware::new()
            .with_strategy(RequestIdStrategy::ShortRandom);
        
        let id1 = middleware.generate_id();
        let id2 = middleware.generate_id();
        
        // IDs should be different
        assert_ne!(id1, id2);
        
        // Should be 8 characters
        assert_eq!(id1.len(), 8);
        
        // Should be base62 characters
        assert!(id1.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_request_id_generate_with_prefix() {
        let middleware = RequestIdMiddleware::new()
            .with_strategy(RequestIdStrategy::ShortRandom)
            .with_prefix("myapp");
        
        let id = middleware.generate_id();
        
        // Should start with prefix
        assert!(id.starts_with("myapp-"));
    }

    #[test]
    fn test_request_id_generate_counter() {
        let middleware = RequestIdMiddleware::new()
            .with_strategy(RequestIdStrategy::PrefixedCounter)
            .with_prefix("req");
        
        let id = middleware.generate_id();
        
        // Should start with prefix
        assert!(id.starts_with("req-"));
    }

    #[tokio::test]
    async fn test_request_id_middleware_adds_header() {
        let middleware = RequestIdMiddleware::new();
        let mut ctx = RequestContext::new("GET", "https://example.com");
        
        middleware.before_request(&mut ctx).await.unwrap();
        
        // Should have X-Request-Id header
        assert!(ctx.headers.contains_key("x-request-id"));
        
        // Should have request_id in metadata
        assert!(ctx.metadata.contains_key("request_id"));
        
        // Header and metadata should match
        let header_value = ctx.headers.get("x-request-id").unwrap();
        let metadata_value = ctx.metadata.get("request_id").unwrap();
        assert_eq!(header_value.to_str().unwrap(), metadata_value);
    }

    #[tokio::test]
    async fn test_request_id_middleware_custom_header() {
        let middleware = RequestIdMiddleware::new()
            .with_header_name("X-Correlation-Id")
            .with_metadata_key("correlation_id");
        
        let mut ctx = RequestContext::new("GET", "https://example.com");
        middleware.before_request(&mut ctx).await.unwrap();
        
        // Should have custom header
        assert!(ctx.headers.contains_key("x-correlation-id"));
        assert!(!ctx.headers.contains_key("x-request-id"));
        
        // Should have custom metadata key
        assert!(ctx.metadata.contains_key("correlation_id"));
    }

    #[tokio::test]
    async fn test_request_id_middleware_no_metadata() {
        let middleware = RequestIdMiddleware::new()
            .with_store_in_metadata(false);
        
        let mut ctx = RequestContext::new("GET", "https://example.com");
        middleware.before_request(&mut ctx).await.unwrap();
        
        // Should have header
        assert!(ctx.headers.contains_key("x-request-id"));
        
        // Should NOT have metadata
        assert!(!ctx.metadata.contains_key("request_id"));
    }

    #[tokio::test]
    async fn test_request_id_middleware_unique_ids() {
        let middleware = RequestIdMiddleware::new();
        
        let mut ctx1 = RequestContext::new("GET", "https://example.com/1");
        let mut ctx2 = RequestContext::new("GET", "https://example.com/2");
        
        middleware.before_request(&mut ctx1).await.unwrap();
        middleware.before_request(&mut ctx2).await.unwrap();
        
        let id1 = ctx1.metadata.get("request_id").unwrap();
        let id2 = ctx2.metadata.get("request_id").unwrap();
        
        // Each request should have a unique ID
        assert_ne!(id1, id2);
    }
}
