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

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are blocked
    Open,
    /// Circuit is half-open, allowing a test request
    HalfOpen,
}

/// Middleware that implements the Circuit Breaker pattern
///
/// The circuit breaker prevents cascading failures by stopping requests to a failing service.
/// After a configurable number of consecutive failures, the circuit opens and blocks requests.
/// After a timeout period, the circuit enters a half-open state to test if the service has recovered.
#[derive(Debug)]
pub struct CircuitBreakerMiddleware {
    /// Current state of the circuit breaker
    state: std::sync::Arc<std::sync::Mutex<CircuitBreakerState>>,
    /// Configuration for the circuit breaker
    config: CircuitBreakerConfig,
}

/// Internal state for the circuit breaker
#[derive(Debug)]
struct CircuitBreakerState {
    /// Current circuit state
    state: CircuitState,
    /// Number of consecutive failures
    failure_count: u32,
    /// Last failure timestamp (for timeout calculation)
    last_failure_time: Option<std::time::Instant>,
    /// Number of successful requests (for statistics)
    success_count: u64,
    /// Total number of blocked requests
    blocked_count: u64,
}

/// Configuration for the Circuit Breaker
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening the circuit
    pub failure_threshold: u32,
    /// Time to wait before transitioning from Open to HalfOpen (in milliseconds)
    pub timeout_ms: u64,
    /// Number of successful requests in HalfOpen state before closing the circuit
    pub success_threshold: u32,
    /// HTTP status codes that should be considered as failures
    pub failure_status_codes: Vec<u16>,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout_ms: 30000, // 30 seconds
            success_threshold: 2,
            failure_status_codes: vec![500, 502, 503, 504],
        }
    }
}

impl CircuitBreakerConfig {
    /// Create a new CircuitBreakerConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the failure threshold
    pub fn with_failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// Set the timeout in milliseconds
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set the success threshold for half-open state
    pub fn with_success_threshold(mut self, threshold: u32) -> Self {
        self.success_threshold = threshold;
        self
    }

    /// Set the status codes that count as failures
    pub fn with_failure_status_codes(mut self, codes: Vec<u16>) -> Self {
        self.failure_status_codes = codes;
        self
    }
}

impl CircuitBreakerMiddleware {
    /// Create a new CircuitBreakerMiddleware with default configuration
    pub fn new() -> Self {
        Self::with_config(CircuitBreakerConfig::default())
    }

    /// Create a new CircuitBreakerMiddleware with custom configuration
    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        Self {
            state: std::sync::Arc::new(std::sync::Mutex::new(CircuitBreakerState {
                state: CircuitState::Closed,
                failure_count: 0,
                last_failure_time: None,
                success_count: 0,
                blocked_count: 0,
            })),
            config,
        }
    }

    /// Get the current circuit state
    pub fn state(&self) -> CircuitState {
        let state = self.state.lock().unwrap();
        self.calculate_current_state(&state)
    }

    /// Get statistics about the circuit breaker
    pub fn stats(&self) -> CircuitBreakerStats {
        let state = self.state.lock().unwrap();
        CircuitBreakerStats {
            state: self.calculate_current_state(&state),
            failure_count: state.failure_count,
            success_count: state.success_count,
            blocked_count: state.blocked_count,
        }
    }

    /// Reset the circuit breaker to closed state
    pub fn reset(&self) {
        let mut state = self.state.lock().unwrap();
        state.state = CircuitState::Closed;
        state.failure_count = 0;
        state.last_failure_time = None;
    }

    /// Calculate the current state, handling timeout transitions
    fn calculate_current_state(&self, state: &CircuitBreakerState) -> CircuitState {
        match state.state {
            CircuitState::Open => {
                // Check if timeout has elapsed
                if let Some(last_failure) = state.last_failure_time {
                    if last_failure.elapsed().as_millis() as u64 >= self.config.timeout_ms {
                        return CircuitState::HalfOpen;
                    }
                }
                CircuitState::Open
            }
            other => other,
        }
    }

    /// Check if a response should be considered a failure
    fn is_failure_status(&self, status_code: u16) -> bool {
        self.config.failure_status_codes.contains(&status_code)
    }
}

impl Default for CircuitBreakerMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the circuit breaker
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    /// Current state of the circuit
    pub state: CircuitState,
    /// Current consecutive failure count
    pub failure_count: u32,
    /// Total successful requests
    pub success_count: u64,
    /// Total blocked requests
    pub blocked_count: u64,
}

#[async_trait]
impl Middleware for CircuitBreakerMiddleware {
    async fn before_request(&self, ctx: &mut RequestContext) -> Result<(), CallerError> {
        let mut state = self.state.lock().unwrap();
        let current_state = self.calculate_current_state(&state);

        match current_state {
            CircuitState::Open => {
                state.blocked_count += 1;
                drop(state); // Release lock before returning error
                Err(CallerError::RequestError(format!(
                    "Circuit breaker is OPEN for {} {}",
                    ctx.method, ctx.url
                )))
            }
            CircuitState::Closed | CircuitState::HalfOpen => {
                // Update the state if we transitioned to HalfOpen
                if current_state != state.state {
                    state.state = current_state;
                }
                Ok(())
            }
        }
    }

    async fn after_response(&self, ctx: &mut ResponseContext) -> Result<(), CallerError> {
        let mut state = self.state.lock().unwrap();

        if self.is_failure_status(ctx.status_code) {
            // Failure response
            state.failure_count += 1;
            state.last_failure_time = Some(std::time::Instant::now());

            match state.state {
                CircuitState::Closed => {
                    if state.failure_count >= self.config.failure_threshold {
                        state.state = CircuitState::Open;
                    }
                }
                CircuitState::HalfOpen => {
                    // Failure in half-open state -> back to open
                    state.state = CircuitState::Open;
                }
                CircuitState::Open => {
                    // Already open, nothing to do
                }
            }
        } else if ctx.is_success() {
            // Success response
            state.success_count += 1;

            match state.state {
                CircuitState::HalfOpen => {
                    // Check if we've had enough successes to close the circuit
                    if state.failure_count > 0 {
                        state.failure_count -= 1;
                    }
                    if state.failure_count == 0 {
                        state.state = CircuitState::Closed;
                    }
                }
                CircuitState::Closed => {
                    // Reset failure count on success
                    state.failure_count = 0;
                }
                CircuitState::Open => {
                    // Shouldn't happen, but reset if it does
                    state.state = CircuitState::Closed;
                    state.failure_count = 0;
                }
            }
        }

        Ok(())
    }

    async fn on_error(&self, _error: &CallerError, _ctx: &RequestContext) {
        let mut state = self.state.lock().unwrap();
        state.failure_count += 1;
        state.last_failure_time = Some(std::time::Instant::now());

        if state.state == CircuitState::HalfOpen {
            state.state = CircuitState::Open;
        } else if state.failure_count >= self.config.failure_threshold {
            state.state = CircuitState::Open;
        }
    }

    fn name(&self) -> &str {
        "CircuitBreakerMiddleware"
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

    // =========================================================================
    // Circuit Breaker Tests
    // =========================================================================

    #[test]
    fn test_circuit_breaker_config_builder() {
        let config = CircuitBreakerConfig::new()
            .with_failure_threshold(10)
            .with_timeout_ms(60000)
            .with_success_threshold(3)
            .with_failure_status_codes(vec![500, 503]);

        assert_eq!(config.failure_threshold, 10);
        assert_eq!(config.timeout_ms, 60000);
        assert_eq!(config.success_threshold, 3);
        assert_eq!(config.failure_status_codes, vec![500, 503]);
    }

    #[test]
    fn test_circuit_breaker_default_config() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.timeout_ms, 30000);
        assert_eq!(config.success_threshold, 2);
        assert!(config.failure_status_codes.contains(&500));
        assert!(config.failure_status_codes.contains(&503));
    }

    #[test]
    fn test_circuit_breaker_initial_state() {
        let middleware = CircuitBreakerMiddleware::new();
        assert_eq!(middleware.state(), CircuitState::Closed);

        let stats = middleware.stats();
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.failure_count, 0);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.blocked_count, 0);
    }

    #[test]
    fn test_circuit_breaker_reset() {
        let middleware = CircuitBreakerMiddleware::new();
        
        // Simulate some failures
        {
            let mut state = middleware.state.lock().unwrap();
            state.failure_count = 3;
            state.state = CircuitState::Open;
        }
        
        assert_eq!(middleware.state(), CircuitState::Open);
        
        middleware.reset();
        
        assert_eq!(middleware.state(), CircuitState::Closed);
        let stats = middleware.stats();
        assert_eq!(stats.failure_count, 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_allows_requests_when_closed() {
        let middleware = CircuitBreakerMiddleware::new();
        let mut ctx = RequestContext::new("GET", "https://example.com");

        let result = middleware.before_request(&mut ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let config = CircuitBreakerConfig::new()
            .with_failure_threshold(2)
            .with_failure_status_codes(vec![500]);
        
        let middleware = CircuitBreakerMiddleware::with_config(config);
        
        // Simulate first failure
        {
            let req_ctx = RequestContext::new("GET", "https://example.com");
            let mut resp_ctx = ResponseContext::new(500, "error".to_string(), req_ctx, 10);
            middleware.after_response(&mut resp_ctx).await.unwrap();
        }
        
        assert_eq!(middleware.state(), CircuitState::Closed);
        
        // Simulate second failure - should open circuit
        {
            let req_ctx = RequestContext::new("GET", "https://example.com");
            let mut resp_ctx = ResponseContext::new(500, "error".to_string(), req_ctx, 10);
            middleware.after_response(&mut resp_ctx).await.unwrap();
        }
        
        assert_eq!(middleware.state(), CircuitState::Open);
        
        // Next request should be blocked
        let mut ctx = RequestContext::new("GET", "https://example.com");
        let result = middleware.before_request(&mut ctx).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circuit breaker is OPEN"));
    }

    #[tokio::test]
    async fn test_circuit_breaker_success_resets_failure_count() {
        let config = CircuitBreakerConfig::new()
            .with_failure_threshold(3);
        
        let middleware = CircuitBreakerMiddleware::with_config(config);
        
        // Simulate a failure
        {
            let req_ctx = RequestContext::new("GET", "https://example.com");
            let mut resp_ctx = ResponseContext::new(500, "error".to_string(), req_ctx, 10);
            middleware.after_response(&mut resp_ctx).await.unwrap();
        }
        
        let stats = middleware.stats();
        assert_eq!(stats.failure_count, 1);
        
        // Simulate a success
        {
            let req_ctx = RequestContext::new("GET", "https://example.com");
            let mut resp_ctx = ResponseContext::new(200, "ok".to_string(), req_ctx, 10);
            middleware.after_response(&mut resp_ctx).await.unwrap();
        }
        
        let stats = middleware.stats();
        assert_eq!(stats.failure_count, 0);
        assert_eq!(stats.state, CircuitState::Closed);
    }

    #[test]
    fn test_circuit_state_debug_and_clone() {
        let state = CircuitState::Closed;
        let cloned = state.clone();
        assert_eq!(state, cloned);
        
        // Test Debug trait
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("Closed"));
    }

    #[test]
    fn test_circuit_breaker_stats_clone() {
        let stats = CircuitBreakerStats {
            state: CircuitState::Closed,
            failure_count: 1,
            success_count: 10,
            blocked_count: 2,
        };
        
        let cloned = stats.clone();
        assert_eq!(stats.state, cloned.state);
        assert_eq!(stats.failure_count, cloned.failure_count);
        assert_eq!(stats.success_count, cloned.success_count);
        assert_eq!(stats.blocked_count, cloned.blocked_count);
    }
}
