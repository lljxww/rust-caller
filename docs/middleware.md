# Middleware System

The caller library provides a powerful middleware system that allows you to intercept and modify requests and responses.

## Overview

Middleware can:
- Modify requests before they are sent
- Process responses after they are received
- Handle errors during request execution
- Implement cross-cutting concerns like logging, retry, and circuit breaking

## Built-in Middleware

### HeaderMiddleware

Add custom headers to all requests:

```rust
use caller::domain::middleware::{HeaderMiddleware, MiddlewareChain, RequestContext};

let middleware = HeaderMiddleware::new()
    .with_header("X-API-Key", "secret-key")
    .with_header("X-Request-Id", "12345");
```

### LoggingMiddleware

Log requests and responses:

```rust
use caller::domain::middleware::LoggingMiddleware;

let middleware = LoggingMiddleware::new()
    .with_log_request(true)
    .with_log_response(true);
```

Output:
```
[Request] GET https://api.example.com/users
[Response] GET https://api.example.com/users - 200 (45ms)
```

### UserAgentMiddleware

Add a custom User-Agent header:

```rust
use caller::domain::middleware::UserAgentMiddleware;

let middleware = UserAgentMiddleware::new("MyApp/1.0");
```

### TimingMiddleware

Add timing/tracing information to requests:

```rust
use caller::domain::middleware::TimingMiddleware;

let middleware = TimingMiddleware::new()
    .with_timing_header("x-request-start");
```

### RetryMiddleware

Retry failed requests:

```rust
use caller::domain::middleware::RetryMiddleware;

let middleware = RetryMiddleware::new()
    .with_max_retries(3)
    .with_status_codes(vec![429, 500, 502, 503, 504]);
```

### CircuitBreakerMiddleware

Prevent cascading failures by stopping requests to a failing service:

```rust
use caller::domain::middleware::{CircuitBreakerMiddleware, CircuitBreakerConfig};

let config = CircuitBreakerConfig::new()
    .with_failure_threshold(5)      // Open after 5 failures
    .with_timeout_ms(30000)          // Wait 30s before trying again
    .with_success_threshold(2)       // Close after 2 successes
    .with_failure_status_codes(vec![500, 502, 503, 504]);

let middleware = CircuitBreakerMiddleware::with_config(config);
```

#### Circuit Breaker States

| State | Description |
|-------|-------------|
| **Closed** | Normal operation, requests flow through |
| **Open** | Requests are blocked, waiting for timeout |
| **HalfOpen** | Testing if service has recovered |

#### Monitoring

```rust
// Get current state
let state = middleware.state();

// Get statistics
let stats = middleware.stats();
println!("Failures: {}", stats.failure_count);
println!("Blocked: {}", stats.blocked_count);

// Reset the circuit
middleware.reset();
```

## Creating Custom Middleware

Implement the `Middleware` trait:

```rust
use caller::domain::middleware::{Middleware, RequestContext, ResponseContext};
use caller::CallerError;
use async_trait::async_trait;

pub struct MyMiddleware;

#[async_trait]
impl Middleware for MyMiddleware {
    async fn before_request(&self, ctx: &mut RequestContext) -> Result<(), CallerError> {
        // Modify request before sending
        ctx.headers.insert(
            "x-custom-header".parse().unwrap(),
            "value".parse().unwrap()
        );
        Ok(())
    }

    async fn after_response(&self, ctx: &mut ResponseContext) -> Result<(), CallerError> {
        // Process response after receiving
        println!("Response time: {}ms", ctx.duration_ms);
        Ok(())
    }

    async fn on_error(&self, error: &CallerError, ctx: &RequestContext) {
        // Handle errors
        eprintln!("Request failed: {}", error);
    }

    fn name(&self) -> &str {
        "MyMiddleware"
    }
}
```

## Middleware Chain

Combine multiple middleware:

```rust
use caller::domain::middleware::{
    MiddlewareChain,
    HeaderMiddleware,
    LoggingMiddleware,
    UserAgentMiddleware,
};

let chain = MiddlewareChain::new()
    .with(HeaderMiddleware::new().with_header("X-App", "MyApp"))
    .with(UserAgentMiddleware::new("MyApp/1.0"))
    .with(LoggingMiddleware::new());

// Execute before request
chain.before_request(&mut ctx).await?;

// Execute after response
chain.after_response(&mut response_ctx).await?;
```

## Request Context

The `RequestContext` contains all information about an outgoing request:

```rust
let ctx = RequestContext::new("GET", "https://api.example.com/users")
    .with_header("Accept", "application/json")
    .with_body(r#"{"key": "value"}"#.to_string())
    .with_params(params)
    .with_metadata("trace_id", "abc123");
```

## Response Context

The `ResponseContext` contains all information about a received response:

```rust
// Status checks
if response.is_success() {
    // 2xx status code
}
if response.is_client_error() {
    // 4xx status code
}
if response.is_server_error() {
    // 5xx status code
}

// Access data
println!("Status: {}", response.status_code);
println!("Body: {}", response.body);
println!("Duration: {}ms", response.duration_ms);
```

## Best Practices

1. **Order matters**: Middleware is executed in the order added to the chain
2. **Keep it simple**: Each middleware should do one thing well
3. **Handle errors**: Return errors to abort the request chain
4. **Use metadata**: Store request-scoped data in `ctx.metadata`
5. **Be async-safe**: Middleware must be `Send + Sync`

## Common Patterns

### Request ID Tracking

```rust
pub struct RequestIdMiddleware;

#[async_trait]
impl Middleware for RequestIdMiddleware {
    async fn before_request(&self, ctx: &mut RequestContext) -> Result<(), CallerError> {
        let request_id = uuid::Uuid::new_v4().to_string();
        ctx.metadata.insert("request_id".to_string(), request_id.clone());
        ctx.headers.insert("x-request-id".parse().unwrap(), request_id.parse().unwrap());
        Ok(())
    }

    fn name(&self) -> &str {
        "RequestIdMiddleware"
    }
}
```

### Rate Limiting

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct RateLimitMiddleware {
    semaphore: Arc<Semaphore>,
}

impl RateLimitMiddleware {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn before_request(&self, _ctx: &mut RequestContext) -> Result<(), CallerError> {
        self.semaphore.acquire().await?.forget();
        Ok(())
    }

    fn name(&self) -> &str {
        "RateLimitMiddleware"
    }
}
```
