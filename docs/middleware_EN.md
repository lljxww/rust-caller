[English](middleware_EN.md) | [简体中文](middleware_CN.md)

# Middleware

`Caller` executes middleware as part of its real HTTP request pipeline. Add
middleware through `CallerBuilder::middleware`, `middleware_arc`, or
`middleware_chain`.

```rust
use caller::{Caller, HeaderMiddleware, LoggingMiddleware};

let caller = Caller::builder()
    .config_path("caller.json")
    .middleware(HeaderMiddleware::new().with_header("X-App", "my-service")?)
    .middleware(LoggingMiddleware::new())
    .build()?;
# Ok::<(), caller::CallerError>(())
```

## Execution order

For each network attempt, `Caller` executes:

```text
build RequestContext
  -> before_request in registration order
  -> apply parameters and authentication
  -> send request
  -> read response
  -> after_response in reverse registration order
```

If preparation, authentication, sending, response reading, or
`after_response` fails, `on_error` is invoked. `call_with_retry` runs the chain
once per attempt, which lets tracing and circuit breaking observe every real
network attempt.

Middleware may update `RequestContext.url`, `method`, `headers`, compatibility
`params`, separated `path_params`/`query_params`/`form_params`, and JSON `body`.
Response middleware may update status, headers, and the text body before an
`ApiResult` is created.

The current download path executes request and error hooks, but does not pass
binary download bodies through `after_response`; `ResponseContext` is currently
text-oriented.

## Custom middleware

```rust
use async_trait::async_trait;
use caller::{CallerError, Middleware, RequestContext, ResponseContext};

struct TraceMiddleware;

#[async_trait]
impl Middleware for TraceMiddleware {
    async fn before_request(&self, ctx: &mut RequestContext) -> Result<(), CallerError> {
        ctx.headers.insert(
            "x-trace-id".parse().unwrap(),
            "trace-001".parse().unwrap(),
        );
        Ok(())
    }

    async fn after_response(&self, ctx: &mut ResponseContext) -> Result<(), CallerError> {
        assert!(ctx.duration_ms > 0 || ctx.is_success());
        Ok(())
    }

    async fn on_error(&self, error: &CallerError, ctx: &RequestContext) {
        eprintln!("{} {}: {error}", ctx.method, ctx.url);
    }
}
```

## Built-in middleware

- `HeaderMiddleware`: adds request headers.
- `UserAgentMiddleware`: overrides the User-Agent request header.
- `LoggingMiddleware`: prints opt-in request, response, and error messages.
- `TimingMiddleware`: adds a Unix-millisecond start timestamp request header.
- `CircuitBreakerMiddleware`: blocks requests after configured failures and
  permits a single half-open probe.

Use `Caller::call_with_retry`, `call_params_with_retry`, or
`call_args_with_retry` with `RetryConfig`. Retry is an execution policy because
it must recreate and authenticate every request attempt.

## Ordering guidance

- Put a circuit breaker before middleware that performs expensive work.
- Treat middleware as a stack: request hooks run in registration order;
  response/error hooks unwind in reverse order.
- Do not log authorization, cookie, API-key, or full query-string values.
- A middleware instance can be used concurrently and must be `Send + Sync`.
- Keep request bodies replayable when retry is enabled.
