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

// ============================================================================
// Signing Middleware
// ============================================================================

/// 签名算法类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SigningAlgorithm {
    /// HMAC-SHA256
    #[default]
    HmacSha256,
    /// HMAC-SHA512
    HmacSha512,
    /// 简单的密钥拼接（不推荐用于生产环境）
    SimpleConcat,
}

/// 签名包含的内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningComponents {
    /// 是否包含 HTTP 方法
    pub include_method: bool,
    /// 是否包含 URL 路径
    pub include_path: bool,
    /// 是否包含查询参数
    pub include_query: bool,
    /// 是否包含请求体
    pub include_body: bool,
    /// 是否包含时间戳
    pub include_timestamp: bool,
    /// 时间戳格式（如 "%s" 为 Unix 时间戳）
    pub timestamp_format: String,
    /// 自定义前缀
    pub prefix: Option<String>,
    /// 自定义后缀
    pub suffix: Option<String>,
}

impl Default for SigningComponents {
    fn default() -> Self {
        Self {
            include_method: true,
            include_path: true,
            include_query: true,
            include_body: true,
            include_timestamp: true,
            timestamp_format: "%s".to_string(), // Unix timestamp
            prefix: None,
            suffix: None,
        }
    }
}

/// 请求签名中间件
///
/// 为API请求添加签名，支持HMAC-SHA256/SHA512等算法。
/// 适用于需要API签名验证的场景（如支付接口、开放平台API等）。
///
/// # 功能
/// - 支持多种签名算法（HMAC-SHA256、HMAC-SHA512等）
/// - 可配置签名内容（方法、路径、参数、body、时间戳）
/// - 支持将签名放入Header或Metadata
/// - 自动添加时间戳防重放
///
/// # 示例
/// ```rust
/// use caller::domain::middleware::{SigningMiddleware, SigningAlgorithm};
///
/// // 创建HMAC-SHA256签名中间件
/// let signing = SigningMiddleware::new("your-secret-key")
///     .with_algorithm(SigningAlgorithm::HmacSha256)
///     .with_signature_header("X-Signature")
///     .with_timestamp_header("X-Timestamp");
///
/// // 或者使用简化配置
/// let signing = SigningMiddleware::simple("your-secret-key");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningMiddleware {
    /// 签名密钥
    pub secret_key: String,
    /// 签名算法
    pub algorithm: SigningAlgorithm,
    /// 签名输出的Header名称
    pub signature_header: String,
    /// 时间戳输出的Header名称（None表示不输出）
    pub timestamp_header: Option<String>,
    /// 签名组件配置
    pub components: SigningComponents,
    /// 是否将签名信息存入metadata
    pub store_in_metadata: bool,
}

impl SigningMiddleware {
    /// 创建新的签名中间件
    ///
    /// # 参数
    /// * `secret_key` - 签名密钥
    pub fn new(secret_key: &str) -> Self {
        Self {
            secret_key: secret_key.to_string(),
            algorithm: SigningAlgorithm::default(),
            signature_header: "X-Signature".to_string(),
            timestamp_header: Some("X-Timestamp".to_string()),
            components: SigningComponents::default(),
            store_in_metadata: true,
        }
    }

    /// 创建简化版签名中间件（仅签名，不含时间戳）
    pub fn simple(secret_key: &str) -> Self {
        Self {
            secret_key: secret_key.to_string(),
            algorithm: SigningAlgorithm::HmacSha256,
            signature_header: "X-Signature".to_string(),
            timestamp_header: None,
            components: SigningComponents {
                include_timestamp: false,
                ..Default::default()
            },
            store_in_metadata: false,
        }
    }

    /// 设置签名算法
    pub fn with_algorithm(mut self, algorithm: SigningAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    /// 设置签名输出的Header名称
    pub fn with_signature_header(mut self, header: &str) -> Self {
        self.signature_header = header.to_string();
        self
    }

    /// 设置时间戳输出的Header名称
    pub fn with_timestamp_header(mut self, header: &str) -> Self {
        self.timestamp_header = Some(header.to_string());
        self
    }

    /// 设置是否包含请求体
    pub fn with_include_body(mut self, include: bool) -> Self {
        self.components.include_body = include;
        self
    }

    /// 设置是否包含时间戳
    pub fn with_include_timestamp(mut self, include: bool) -> Self {
        self.components.include_timestamp = include;
        self
    }

    /// 设置签名前缀
    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.components.prefix = Some(prefix.to_string());
        self
    }

    /// 设置是否存入metadata
    pub fn with_store_in_metadata(mut self, store: bool) -> Self {
        self.store_in_metadata = store;
        self
    }

    /// 获取当前时间戳字符串
    fn get_timestamp(&self) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.to_string()
    }

    /// 构建待签名字符串
    pub fn build_string_to_sign(&self, ctx: &RequestContext, timestamp: &str) -> String {
        let mut parts = Vec::new();

        // 添加前缀
        if let Some(ref prefix) = self.components.prefix {
            parts.push(prefix.clone());
        }

        // 添加HTTP方法
        if self.components.include_method {
            parts.push(ctx.method.to_uppercase());
        }

        // 添加URL路径和查询参数
        if self.components.include_path || self.components.include_query {
            let url = url::Url::parse(&ctx.url).ok();
            if let Some(ref parsed_url) = url {
                if self.components.include_path {
                    parts.push(parsed_url.path().to_string());
                }
                if self.components.include_query && parsed_url.query().is_some() {
                    parts.push(parsed_url.query().unwrap_or("").to_string());
                }
            } else {
                // 如果URL解析失败，直接使用原始URL
                parts.push(ctx.url.clone());
            }
        }

        // 添加请求体
        if self.components.include_body {
            if let Some(ref body) = ctx.body {
                if !body.is_empty() {
                    parts.push(body.clone());
                }
            }
        }

        // 添加时间戳
        if self.components.include_timestamp {
            parts.push(timestamp.to_string());
        }

        // 添加后缀
        if let Some(ref suffix) = self.components.suffix {
            parts.push(suffix.clone());
        }

        parts.join("\n")
    }

    /// 计算签名
    pub fn calculate_signature(&self, string_to_sign: &str) -> String {
        use hmac::{Hmac, Mac};
        use sha2::{Sha256, Sha512};

        match self.algorithm {
            SigningAlgorithm::HmacSha256 => {
                let mut mac = <Hmac<Sha256>>::new_from_slice(self.secret_key.as_bytes())
                    .expect("HMAC can take key of any size");
                mac.update(string_to_sign.as_bytes());
                hex::encode(mac.finalize().into_bytes())
            }
            SigningAlgorithm::HmacSha512 => {
                let mut mac = <Hmac<Sha512>>::new_from_slice(self.secret_key.as_bytes())
                    .expect("HMAC can take key of any size");
                mac.update(string_to_sign.as_bytes());
                hex::encode(mac.finalize().into_bytes())
            }
            SigningAlgorithm::SimpleConcat => {
                // 简单的密钥拼接（仅用于测试）
                format!("{}{}", self.secret_key, string_to_sign)
            }
        }
    }

    /// 对请求进行签名并返回签名和时间戳
    pub fn sign(&self, ctx: &RequestContext) -> (String, String) {
        let timestamp = self.get_timestamp();
        let string_to_sign = self.build_string_to_sign(ctx, &timestamp);
        let signature = self.calculate_signature(&string_to_sign);
        (signature, timestamp)
    }
}

impl Default for SigningMiddleware {
    fn default() -> Self {
        Self::new("")
    }
}

#[async_trait]
impl Middleware for SigningMiddleware {
    async fn before_request(&self, ctx: &mut RequestContext) -> Result<(), CallerError> {
        let (signature, timestamp) = self.sign(ctx);

        // 添加签名Header
        if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(self.signature_header.as_bytes())
            && let Ok(header_value) = reqwest::header::HeaderValue::from_str(&signature)
        {
            ctx.headers.insert(header_name, header_value);
        }

        // 添加时间戳Header
        if let Some(ref ts_header) = self.timestamp_header {
            if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(ts_header.as_bytes())
                && let Ok(header_value) = reqwest::header::HeaderValue::from_str(&timestamp)
            {
                ctx.headers.insert(header_name, header_value);
            }
        }

        // 存入metadata
        if self.store_in_metadata {
            ctx.metadata.insert("signature".to_string(), signature.clone());
            ctx.metadata.insert("signature_timestamp".to_string(), timestamp.clone());
            ctx.metadata.insert("signature_algorithm".to_string(), format!("{:?}", self.algorithm));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "SigningMiddleware"
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

    // ============================================================================
    // Timeout Middleware Tests
    // ============================================================================

    #[test]
    fn test_timeout_middleware_default() {
        let middleware = TimeoutMiddleware::new();
        assert_eq!(middleware.timeout_ms, 30_000);
        assert_eq!(middleware.connect_timeout_ms, 5_000);
        assert!(middleware.read_timeout_ms.is_none());
        assert!(middleware.store_in_metadata);
    }

    #[test]
    fn test_timeout_middleware_builder() {
        let middleware = TimeoutMiddleware::new()
            .with_timeout(std::time::Duration::from_secs(60))
            .with_connect_timeout(std::time::Duration::from_secs(10))
            .with_read_timeout(std::time::Duration::from_secs(30))
            .with_store_in_metadata(false);

        assert_eq!(middleware.timeout_ms, 60_000);
        assert_eq!(middleware.connect_timeout_ms, 10_000);
        assert_eq!(middleware.read_timeout_ms, Some(30_000));
        assert!(!middleware.store_in_metadata);
    }

    #[test]
    fn test_timeout_middleware_presets() {
        let quick = TimeoutMiddleware::quick();
        assert_eq!(quick.timeout_ms, 1_000);
        assert_eq!(quick.connect_timeout_ms, 500);

        let long = TimeoutMiddleware::long();
        assert_eq!(long.timeout_ms, 300_000);
        assert_eq!(long.connect_timeout_ms, 30_000);
    }

    #[test]
    fn test_timeout_middleware_getters() {
        let middleware = TimeoutMiddleware::new()
            .with_timeout(std::time::Duration::from_millis(5000))
            .with_connect_timeout(std::time::Duration::from_millis(2000))
            .with_read_timeout(std::time::Duration::from_millis(3000));

        assert_eq!(middleware.timeout(), std::time::Duration::from_millis(5000));
        assert_eq!(middleware.connect_timeout(), std::time::Duration::from_millis(2000));
        assert_eq!(middleware.read_timeout(), Some(std::time::Duration::from_millis(3000)));
    }

    #[test]
    fn test_timeout_middleware_is_timed_out() {
        let middleware = TimeoutMiddleware::quick(); // 1 second timeout
        
        let start = std::time::Instant::now();
        assert!(!middleware.is_timed_out(start));
        
        // After waiting, should be timed out
        std::thread::sleep(std::time::Duration::from_millis(1100));
        assert!(middleware.is_timed_out(start));
    }

    #[test]
    fn test_timeout_middleware_remaining_time() {
        let middleware = TimeoutMiddleware::new()
            .with_timeout(std::time::Duration::from_millis(100));
        
        let start = std::time::Instant::now();
        let remaining = middleware.remaining_time(start);
        
        // Should have most of the timeout remaining
        assert!(remaining.as_millis() >= 90);
        
        // After timeout expires, remaining should be zero
        std::thread::sleep(std::time::Duration::from_millis(110));
        assert_eq!(middleware.remaining_time(start), std::time::Duration::ZERO);
    }

    #[tokio::test]
    async fn test_timeout_middleware_adds_metadata() {
        let middleware = TimeoutMiddleware::new()
            .with_timeout(std::time::Duration::from_secs(10));
        
        let mut ctx = RequestContext::new("GET", "https://example.com");
        middleware.before_request(&mut ctx).await.unwrap();
        
        // Should have timeout info in metadata
        assert_eq!(ctx.metadata.get("timeout_ms"), Some(&"10000".to_string()));
        assert!(ctx.metadata.contains_key("connect_timeout_ms"));
        assert!(ctx.metadata.contains_key("request_start_time"));
    }

    #[tokio::test]
    async fn test_timeout_middleware_no_metadata() {
        let middleware = TimeoutMiddleware::new()
            .with_store_in_metadata(false);
        
        let mut ctx = RequestContext::new("GET", "https://example.com");
        middleware.before_request(&mut ctx).await.unwrap();
        
        // Should NOT have timeout info in metadata
        assert!(!ctx.metadata.contains_key("timeout_ms"));
        assert!(!ctx.metadata.contains_key("request_start_time"));
    }

    #[tokio::test]
    async fn test_timeout_middleware_with_read_timeout() {
        let middleware = TimeoutMiddleware::new()
            .with_read_timeout(std::time::Duration::from_secs(5));
        
        let mut ctx = RequestContext::new("GET", "https://example.com");
        middleware.before_request(&mut ctx).await.unwrap();
        
        // Should have read timeout in metadata
        assert_eq!(ctx.metadata.get("read_timeout_ms"), Some(&"5000".to_string()));
    }

    // ============================================================================
    // Cache Middleware Tests
    // ============================================================================

    #[test]
    fn test_cache_middleware_default() {
        let cache = CacheMiddleware::new();
        assert_eq!(cache.default_ttl_ms, 300_000); // 5 minutes
        assert_eq!(cache.max_entries, 500);
        assert!(!cache.cache_post_requests);
        assert!(cache.respect_cache_control);
    }

    #[test]
    fn test_cache_middleware_builder() {
        let cache = CacheMiddleware::new()
            .with_ttl(std::time::Duration::from_secs(120))
            .with_max_entries(1000)
            .with_cache_post_requests(true)
            .with_respect_cache_control(false);

        assert_eq!(cache.default_ttl_ms, 120_000);
        assert_eq!(cache.max_entries, 1000);
        assert!(cache.cache_post_requests);
        assert!(!cache.respect_cache_control);
    }

    #[test]
    fn test_cache_entry_creation() {
        let entry = CacheEntry::new("test body".to_string(), 200, 60_000);
        
        assert_eq!(entry.body, "test body");
        assert_eq!(entry.status_code, 200);
        assert_eq!(entry.ttl_ms, 60_000);
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_entry_with_header() {
        let entry = CacheEntry::new("body".to_string(), 200, 60_000)
            .with_header("Content-Type", "application/json");
        
        assert_eq!(entry.headers.get("Content-Type"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_cache_entry_expiration() {
        // Create entry with 1ms TTL
        let entry = CacheEntry::new("body".to_string(), 200, 1);
        
        // Should not be expired immediately
        assert!(!entry.is_expired());
        
        // Wait for expiration
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        // Should be expired now
        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_generate_key() {
        let cache = CacheMiddleware::new();
        
        let ctx1 = RequestContext::new("GET", "https://example.com/api");
        let key1 = cache.generate_cache_key(&ctx1);
        
        let ctx2 = RequestContext::new("GET", "https://example.com/api");
        let key2 = cache.generate_cache_key(&ctx2);
        
        // Same request should generate same key
        assert_eq!(key1, key2);
        
        // Different URL should generate different key
        let ctx3 = RequestContext::new("GET", "https://example.com/other");
        let key3 = cache.generate_cache_key(&ctx3);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_cache_generate_key_with_params() {
        let cache = CacheMiddleware::new();
        
        let mut params = HashMap::new();
        params.insert("id".to_string(), "123".to_string());
        
        let ctx = RequestContext::new("GET", "https://example.com/api")
            .with_params(params);
        
        let key = cache.generate_cache_key(&ctx);
        
        // Key should include params
        assert!(key.contains("id=123"));
    }

    #[test]
    fn test_cache_should_cache() {
        let cache = CacheMiddleware::new();
        
        assert!(cache.should_cache("GET"));
        assert!(!cache.should_cache("POST"));
        assert!(!cache.should_cache("PUT"));
        assert!(!cache.should_cache("DELETE"));
        
        let cache_with_post = CacheMiddleware::new()
            .with_cache_post_requests(true);
        
        assert!(cache_with_post.should_cache("GET"));
        assert!(cache_with_post.should_cache("POST"));
        assert!(!cache_with_post.should_cache("PUT"));
    }

    #[test]
    fn test_cache_put_and_get() {
        let mut cache = CacheMiddleware::new()
            .with_ttl(std::time::Duration::from_secs(60));
        
        let key = "test-key".to_string();
        let entry = CacheEntry::new("cached response".to_string(), 200, 60_000);
        
        cache.put(key.clone(), entry);
        
        // Should retrieve cached entry
        let retrieved = cache.get(&key);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().body, "cached response");
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = CacheMiddleware::new();
        
        // Get non-existent key
        let result = cache.get("non-existent");
        assert!(result.is_none());
        
        // Miss count should increase
        assert_eq!(cache.misses, 1);
    }

    #[test]
    fn test_cache_hit_and_miss_stats() {
        let mut cache = CacheMiddleware::new();
        
        let key = "test-key".to_string();
        let entry = CacheEntry::new("body".to_string(), 200, 60_000);
        
        cache.put(key.clone(), entry);
        
        // First get is a hit
        cache.get(&key);
        assert_eq!(cache.hits, 1);
        assert_eq!(cache.misses, 0);
        
        // Non-existent key is a miss
        cache.get("other-key");
        assert_eq!(cache.hits, 1);
        assert_eq!(cache.misses, 1);
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = CacheMiddleware::new();
        
        let key = "test-key".to_string();
        let entry = CacheEntry::new("body".to_string(), 200, 60_000);
        cache.put(key.clone(), entry);
        
        // 2 hits, 1 miss
        cache.get(&key);
        cache.get(&key);
        cache.get("miss");
        
        let stats = cache.stats();
        assert_eq!(stats.entries, 1);
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = CacheMiddleware::new();
        
        cache.put("key1".to_string(), CacheEntry::new("body1".to_string(), 200, 60_000));
        cache.put("key2".to_string(), CacheEntry::new("body2".to_string(), 200, 60_000));
        
        assert_eq!(cache.len(), 2);
        
        cache.clear();
        
        assert!(cache.is_empty());
        assert_eq!(cache.hits, 0);
        assert_eq!(cache.misses, 0);
    }

    #[test]
    fn test_cache_max_entries_eviction() {
        let mut cache = CacheMiddleware::new()
            .with_max_entries(5);
        
        // Add 5 entries
        for i in 0..5 {
            cache.put(
                format!("key{}", i),
                CacheEntry::new(format!("body{}", i), 200, 60_000),
            );
        }
        
        assert_eq!(cache.len(), 5);
        
        // Adding more should trigger eviction
        cache.put("key5".to_string(), CacheEntry::new("body5".to_string(), 200, 60_000));
        
        // Should have evicted some entries
        assert!(cache.len() <= 5);
    }

    #[tokio::test]
    async fn test_cache_middleware_adds_metadata() {
        let cache = CacheMiddleware::new();
        let mut ctx = RequestContext::new("GET", "https://example.com/api");
        
        cache.before_request(&mut ctx).await.unwrap();
        
        // Should have cache_key in metadata
        assert!(ctx.metadata.contains_key("cache_key"));
        assert!(ctx.metadata.contains_key("cache_entries"));
    }

    #[tokio::test]
    async fn test_cache_middleware_response_metadata() {
        let cache = CacheMiddleware::new();
        
        let req_ctx = RequestContext::new("GET", "https://example.com/api");
        let mut resp_ctx = ResponseContext::new(200, "ok".to_string(), req_ctx, 100);
        
        cache.after_response(&mut resp_ctx).await.unwrap();
        
        // Should indicate cache eligibility
        assert_eq!(
            resp_ctx.request.metadata.get("cache_eligible"),
            Some(&"true".to_string())
        );
    }

    #[tokio::test]
    async fn test_cache_middleware_post_not_cacheable() {
        let cache = CacheMiddleware::new(); // cache_post_requests = false
        
        let req_ctx = RequestContext::new("POST", "https://example.com/api");
        let mut resp_ctx = ResponseContext::new(200, "ok".to_string(), req_ctx, 100);
        
        cache.after_response(&mut resp_ctx).await.unwrap();
        
        // POST should not be cacheable by default
        assert_eq!(
            resp_ctx.request.metadata.get("cache_eligible"),
            Some(&"false".to_string())
        );
    }

    #[test]
    fn test_cache_strategy_variants() {
        let memory = CacheMiddleware::new()
            .with_strategy(CacheStrategy::Memory);
        assert_eq!(memory.strategy, CacheStrategy::Memory);
        
        let none = CacheMiddleware::new()
            .with_strategy(CacheStrategy::None);
        assert_eq!(none.strategy, CacheStrategy::None);
        
        let success_only = CacheMiddleware::new()
            .with_strategy(CacheStrategy::SuccessOnly);
        assert_eq!(success_only.strategy, CacheStrategy::SuccessOnly);
    }

    // ============================================================================
    // Signing Middleware Tests
    // ============================================================================

    #[test]
    fn test_signing_middleware_new() {
        let signing = SigningMiddleware::new("test-secret");
        assert_eq!(signing.secret_key, "test-secret");
        assert_eq!(signing.algorithm, SigningAlgorithm::HmacSha256);
        assert_eq!(signing.signature_header, "X-Signature");
        assert_eq!(signing.timestamp_header, Some("X-Timestamp".to_string()));
    }

    #[test]
    fn test_signing_middleware_simple() {
        let signing = SigningMiddleware::simple("simple-key");
        assert_eq!(signing.secret_key, "simple-key");
        assert_eq!(signing.algorithm, SigningAlgorithm::HmacSha256);
        assert!(signing.timestamp_header.is_none());
        assert!(!signing.components.include_timestamp);
    }

    #[test]
    fn test_signing_middleware_builder() {
        let signing = SigningMiddleware::new("secret")
            .with_algorithm(SigningAlgorithm::HmacSha512)
            .with_signature_header("X-Api-Signature")
            .with_timestamp_header("X-Api-Timestamp")
            .with_include_body(false)
            .with_prefix("MYAPP");

        assert_eq!(signing.algorithm, SigningAlgorithm::HmacSha512);
        assert_eq!(signing.signature_header, "X-Api-Signature");
        assert_eq!(signing.timestamp_header, Some("X-Api-Timestamp".to_string()));
        assert!(!signing.components.include_body);
        assert_eq!(signing.components.prefix, Some("MYAPP".to_string()));
    }

    #[test]
    fn test_signing_algorithm_variants() {
        assert_eq!(SigningAlgorithm::default(), SigningAlgorithm::HmacSha256);
    }

    #[test]
    fn test_signing_build_string_to_sign() {
        let signing = SigningMiddleware::new("secret");
        let ctx = RequestContext::new("POST", "https://api.example.com/users?debug=true")
            .with_body(r#"{"name":"test"}"#.to_string());

        let string_to_sign = signing.build_string_to_sign(&ctx, "1234567890");

        // Should contain method
        assert!(string_to_sign.contains("POST"));
        // Should contain path
        assert!(string_to_sign.contains("/users"));
        // Should contain timestamp
        assert!(string_to_sign.contains("1234567890"));
    }

    #[test]
    fn test_signing_calculate_signature_hmac_sha256() {
        let signing = SigningMiddleware::new("my-secret-key")
            .with_algorithm(SigningAlgorithm::HmacSha256);

        let signature = signing.calculate_signature("test data to sign");

        // Should produce a hex string
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
        // Should be consistent
        let signature2 = signing.calculate_signature("test data to sign");
        assert_eq!(signature, signature2);
    }

    #[test]
    fn test_signing_calculate_signature_hmac_sha512() {
        let signing = SigningMiddleware::new("my-secret-key")
            .with_algorithm(SigningAlgorithm::HmacSha512);

        let signature = signing.calculate_signature("test data to sign");

        // Should produce a hex string
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
        // SHA-512 produces longer signatures
        assert!(signature.len() > 64);
    }

    #[test]
    fn test_signing_calculate_signature_simple_concat() {
        let signing = SigningMiddleware::new("my-key")
            .with_algorithm(SigningAlgorithm::SimpleConcat);

        let signature = signing.calculate_signature("data");

        // Should be key + data
        assert_eq!(signature, "my-keydata");
    }

    #[test]
    fn test_signing_sign_returns_timestamp() {
        let signing = SigningMiddleware::new("secret");
        let ctx = RequestContext::new("GET", "https://api.example.com/test");

        let (signature, timestamp) = signing.sign(&ctx);

        // Signature should be hex
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
        // Timestamp should be numeric
        assert!(timestamp.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_signing_different_data_different_signature() {
        let signing = SigningMiddleware::new("secret");

        let sig1 = signing.calculate_signature("data1");
        let sig2 = signing.calculate_signature("data2");

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_signing_different_keys_different_signature() {
        let signing1 = SigningMiddleware::new("key1");
        let signing2 = SigningMiddleware::new("key2");

        let sig1 = signing1.calculate_signature("same data");
        let sig2 = signing2.calculate_signature("same data");

        assert_ne!(sig1, sig2);
    }

    #[tokio::test]
    async fn test_signing_middleware_adds_headers() {
        let signing = SigningMiddleware::new("test-secret");
        let mut ctx = RequestContext::new("GET", "https://api.example.com/test");

        signing.before_request(&mut ctx).await.unwrap();

        // Should have signature header
        assert!(ctx.headers.contains_key("x-signature"));
        // Should have timestamp header
        assert!(ctx.headers.contains_key("x-timestamp"));
        // Should have metadata
        assert!(ctx.metadata.contains_key("signature"));
        assert!(ctx.metadata.contains_key("signature_timestamp"));
        assert!(ctx.metadata.contains_key("signature_algorithm"));
    }

    #[tokio::test]
    async fn test_signing_middleware_custom_headers() {
        let signing = SigningMiddleware::new("secret")
            .with_signature_header("X-Api-Sign")
            .with_timestamp_header("X-Api-Time");

        let mut ctx = RequestContext::new("POST", "https://api.example.com/data");
        signing.before_request(&mut ctx).await.unwrap();

        assert!(ctx.headers.contains_key("x-api-sign"));
        assert!(ctx.headers.contains_key("x-api-time"));
    }

    #[tokio::test]
    async fn test_signing_middleware_no_timestamp_header() {
        let signing = SigningMiddleware::simple("secret");
        let mut ctx = RequestContext::new("GET", "https://api.example.com/test");

        signing.before_request(&mut ctx).await.unwrap();

        // Should have signature
        assert!(ctx.headers.contains_key("x-signature"));
        // Should NOT have timestamp header
        assert!(!ctx.headers.contains_key("x-timestamp"));
    }

    #[tokio::test]
    async fn test_signing_middleware_name() {
        let signing = SigningMiddleware::new("secret");
        assert_eq!(signing.name(), "SigningMiddleware");
    }
}

// ============================================================================
// Timeout Middleware
// ============================================================================

/// Timeout middleware for controlling request timeout duration
///
/// This middleware adds timeout configuration to requests, helping prevent
/// hanging requests and ensuring reasonable response times.
///
/// # Features
/// - Configurable global timeout
/// - Per-request timeout override via metadata
/// - Timeout tracking in request metadata
///
/// # Example
/// ```rust
/// use caller::domain::middleware::TimeoutMiddleware;
/// use std::time::Duration;
///
/// // Create with 30 second timeout
/// let timeout = TimeoutMiddleware::new()
///     .with_timeout(Duration::from_secs(30))
///     .with_connect_timeout(Duration::from_secs(5));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutMiddleware {
    /// Total request timeout (including response body)
    pub timeout_ms: u64,
    /// Connection establishment timeout
    pub connect_timeout_ms: u64,
    /// Time to wait for the first byte of response
    #[serde(default)]
    pub read_timeout_ms: Option<u64>,
    /// Whether to store timeout info in metadata
    #[serde(default = "default_store_in_metadata")]
    pub store_in_metadata: bool,
}

fn default_store_in_metadata() -> bool {
    true
}

impl TimeoutMiddleware {
    /// Create a new timeout middleware with default settings (30s timeout, 5s connect)
    pub fn new() -> Self {
        Self {
            timeout_ms: 30_000,
            connect_timeout_ms: 5_000,
            read_timeout_ms: None,
            store_in_metadata: true,
        }
    }

    /// Create a timeout middleware with custom timeout duration
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout_ms = timeout.as_millis() as u64;
        self
    }

    /// Set the connection timeout
    pub fn with_connect_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.connect_timeout_ms = timeout.as_millis() as u64;
        self
    }

    /// Set the read timeout (time to first byte)
    pub fn with_read_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.read_timeout_ms = Some(timeout.as_millis() as u64);
        self
    }

    /// Set whether to store timeout info in metadata
    pub fn with_store_in_metadata(mut self, store: bool) -> Self {
        self.store_in_metadata = store;
        self
    }

    /// Get the timeout duration
    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_ms)
    }

    /// Get the connect timeout duration
    pub fn connect_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.connect_timeout_ms)
    }

    /// Get the read timeout duration (if set)
    pub fn read_timeout(&self) -> Option<std::time::Duration> {
        self.read_timeout_ms.map(std::time::Duration::from_millis)
    }

    /// Check if the request has exceeded its timeout based on start time
    pub fn is_timed_out(&self, start_time: std::time::Instant) -> bool {
        start_time.elapsed() >= self.timeout()
    }

    /// Get remaining time until timeout
    pub fn remaining_time(&self, start_time: std::time::Instant) -> std::time::Duration {
        let elapsed = start_time.elapsed();
        if elapsed >= self.timeout() {
            std::time::Duration::ZERO
        } else {
            self.timeout() - elapsed
        }
    }
}

impl Default for TimeoutMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for TimeoutMiddleware {
    async fn before_request(&self, ctx: &mut RequestContext) -> Result<(), CallerError> {
        if self.store_in_metadata {
            ctx.metadata.insert("timeout_ms".to_string(), self.timeout_ms.to_string());
            ctx.metadata.insert("connect_timeout_ms".to_string(), self.connect_timeout_ms.to_string());
            if let Some(read_timeout) = self.read_timeout_ms {
                ctx.metadata.insert("read_timeout_ms".to_string(), read_timeout.to_string());
            }
            // Store request start time for timeout tracking
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
                .to_string();
            ctx.metadata.insert("request_start_time".to_string(), now);
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "TimeoutMiddleware"
    }
}

// Additional Timeout Middleware Tests (added to existing tests module)
impl TimeoutMiddleware {
    /// Create a quick timeout middleware for testing (1 second)
    pub fn quick() -> Self {
        Self::new()
            .with_timeout(std::time::Duration::from_secs(1))
            .with_connect_timeout(std::time::Duration::from_millis(500))
    }

    /// Create a long timeout middleware for slow services (5 minutes)
    pub fn long() -> Self {
        Self::new()
            .with_timeout(std::time::Duration::from_secs(300))
            .with_connect_timeout(std::time::Duration::from_secs(30))
    }
}

// ============================================================================
// Cache Middleware
// ============================================================================

/// A single cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// The cached response body
    pub body: String,
    /// HTTP status code
    pub status_code: u16,
    /// Response headers (serialized as HashMap)
    pub headers: HashMap<String, String>,
    /// When the entry was created (UNIX timestamp in milliseconds)
    pub created_at: u64,
    /// Time-to-live in milliseconds
    pub ttl_ms: u64,
}

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(body: String, status_code: u16, ttl_ms: u64) -> Self {
        Self {
            body,
            status_code,
            headers: HashMap::new(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            ttl_ms,
        }
    }

    /// Add a header to the cache entry
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Check if the cache entry has expired
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        now > self.created_at + self.ttl_ms
    }

    /// Get remaining time until expiration (in milliseconds)
    pub fn remaining_ttl(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.created_at + self.ttl_ms.saturating_sub(now)
    }
}

/// Cache storage strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CacheStrategy {
    /// In-memory cache (default, fastest but not persistent)
    #[default]
    Memory,
    /// No caching (disable cache)
    None,
    /// Cache only specific status codes (e.g., 200 only)
    SuccessOnly,
}

/// Response caching middleware
///
/// Caches GET request responses to reduce redundant API calls and improve performance.
/// Supports configurable TTL, cache key generation, and multiple caching strategies.
///
/// # Features
/// - Configurable time-to-live (TTL)
/// - Cache key based on URL + query params
/// - Optional header-based cache control (respect Cache-Control header)
/// - Cache hit/miss tracking in metadata
///
/// # Example
/// ```rust
/// use caller::domain::middleware::CacheMiddleware;
/// use std::time::Duration;
///
/// let cache = CacheMiddleware::new()
///     .with_ttl(Duration::from_secs(60))     // Cache for 60 seconds
///     .with_max_entries(1000)                // Max 1000 cached items
///     .with_cache_post_requests(false);      // Don't cache POST requests
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMiddleware {
    /// Default time-to-live for cached entries (in milliseconds)
    pub default_ttl_ms: u64,
    /// Maximum number of entries in the cache
    pub max_entries: usize,
    /// Whether to cache POST requests (default: false)
    pub cache_post_requests: bool,
    /// Caching strategy
    pub strategy: CacheStrategy,
    /// Whether to respect Cache-Control headers from server
    pub respect_cache_control: bool,
    /// Cache storage (key -> CacheEntry)
    #[serde(skip)]
    pub cache: HashMap<String, CacheEntry>,
    /// Number of cache hits
    #[serde(skip)]
    pub hits: u64,
    /// Number of cache misses
    #[serde(skip)]
    pub misses: u64,
}

impl CacheMiddleware {
    /// Create a new cache middleware with default settings (5 minute TTL, 500 max entries)
    pub fn new() -> Self {
        Self {
            default_ttl_ms: 300_000, // 5 minutes
            max_entries: 500,
            cache_post_requests: false,
            strategy: CacheStrategy::Memory,
            respect_cache_control: true,
            cache: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    /// Set the default TTL for cached entries
    pub fn with_ttl(mut self, ttl: std::time::Duration) -> Self {
        self.default_ttl_ms = ttl.as_millis() as u64;
        self
    }

    /// Set the maximum number of cache entries
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Enable or disable caching of POST requests
    pub fn with_cache_post_requests(mut self, enabled: bool) -> Self {
        self.cache_post_requests = enabled;
        self
    }

    /// Set the caching strategy
    pub fn with_strategy(mut self, strategy: CacheStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Enable or disable respecting Cache-Control headers
    pub fn with_respect_cache_control(mut self, respect: bool) -> Self {
        self.respect_cache_control = respect;
        self
    }

    /// Generate a cache key from request context
    pub fn generate_cache_key(&self, ctx: &RequestContext) -> String {
        // Use method + URL + sorted params as cache key
        let mut key = format!("{}:{}", ctx.method, ctx.url);
        
        if let Some(ref params) = ctx.params {
            // Sort params for consistent key generation
            let mut sorted_params: Vec<_> = params.iter().collect();
            sorted_params.sort_by_key(|(k, _)| *k);
            
            for (k, v) in sorted_params {
                key.push_str(&format!("&{}={}", k, v));
            }
        }
        
        if let Some(ref body) = ctx.body {
            key.push_str(&format!("|body:{}", body));
        }
        
        key
    }

    /// Check if caching is enabled for this request method
    pub fn should_cache(&self, method: &str) -> bool {
        method == "GET" || (self.cache_post_requests && method == "POST")
    }

    /// Get a cached response if available and not expired
    pub fn get(&mut self, key: &str) -> Option<CacheEntry> {
        // Check if entry exists and is not expired
        let should_remove = match self.cache.get(key) {
            Some(entry) => entry.is_expired(),
            None => {
                self.misses += 1;
                return None;
            }
        };
        
        if should_remove {
            self.cache.remove(key);
            self.misses += 1;
            return None;
        }
        
        self.hits += 1;
        self.cache.get(key).cloned()
    }

    /// Store a response in the cache
    pub fn put(&mut self, key: String, entry: CacheEntry) {
        // Evict oldest entries if at capacity
        if self.cache.len() >= self.max_entries {
            self.evict_oldest();
        }
        self.cache.insert(key, entry);
    }

    /// Remove the oldest entries to make room
    fn evict_oldest(&mut self) {
        // Remove at least 1 entry, or 10% of max_entries
        let to_remove = std::cmp::max(1, self.max_entries / 10);
        let mut entries: Vec<_> = self.cache.iter()
            .map(|(k, v)| (k.clone(), v.created_at))
            .collect();
        entries.sort_by_key(|(_, t)| *t);
        
        for (key, _) in entries.into_iter().take(to_remove) {
            self.cache.remove(&key);
        }
    }

    /// Clear all cached entries
    pub fn clear(&mut self) {
        self.cache.clear();
        self.hits = 0;
        self.misses = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.cache.len(),
            hits: self.hits,
            misses: self.misses,
            hit_rate: if self.hits + self.misses > 0 {
                self.hits as f64 / (self.hits + self.misses) as f64
            } else {
                0.0
            },
        }
    }

    /// Get the number of cached entries
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Parse Cache-Control header to get TTL
    fn parse_cache_control_ttl(&self, headers: &HeaderMap) -> Option<u64> {
        if !self.respect_cache_control {
            return None;
        }

        let cache_control = headers.get("cache-control")?.to_str().ok()?;
        
        // Parse max-age directive
        for directive in cache_control.split(',') {
            let directive = directive.trim();
            if let Some(max_age) = directive.strip_prefix("max-age=")
                && let Ok(seconds) = max_age.parse::<u64>()
            {
                return Some(seconds * 1000); // Convert to milliseconds
            }
        }
        
        None
    }
}

impl Default for CacheMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of entries in cache
    pub entries: usize,
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Hit rate (0.0 - 1.0)
    pub hit_rate: f64,
}

#[async_trait]
impl Middleware for CacheMiddleware {
    async fn before_request(&self, ctx: &mut RequestContext) -> Result<(), CallerError> {
        // Add cache key to metadata for potential use
        let cache_key = self.generate_cache_key(ctx);
        ctx.metadata.insert("cache_key".to_string(), cache_key);
        
        // Add cache stats to metadata
        ctx.metadata.insert("cache_entries".to_string(), self.cache.len().to_string());
        
        Ok(())
    }

    async fn after_response(&self, ctx: &mut ResponseContext) -> Result<(), CallerError> {
        // Indicate that caching is available for this response
        ctx.request.metadata.insert(
            "cache_eligible".to_string(),
            self.should_cache(&ctx.request.method).to_string(),
        );

        // Parse Cache-Control header and add TTL to metadata if available
        if let Some(ttl_ms) = self.parse_cache_control_ttl(&ctx.headers) {
            ctx.request
                .metadata
                .insert("cache_ttl_ms".to_string(), ttl_ms.to_string());
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "CacheMiddleware"
    }
}
