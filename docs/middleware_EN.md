# Middleware 系统

`caller` 当前公开了一组 middleware 相关类型，用于描述和组织请求/响应拦截逻辑。

包括：

- `Middleware`
- `MiddlewareChain`
- `RequestContext`
- `ResponseContext`
- 内置 middleware 类型，如 `HeaderMiddleware`、`LoggingMiddleware`、`UserAgentMiddleware`

## 当前状态

这里最重要的一点是：

**middleware 目前还是独立能力，尚未接入 `Caller` 的真实请求执行链。**

也就是说，当前这些类型更适合：

- 独立测试
- 作为你自己上层封装的基础设施
- 预演未来请求管线设计

而不是：

- 直接注册到 `Caller` 后自动对所有请求生效

## 核心概念

### RequestContext

描述一个待发送请求的上下文：

```rust
use caller::RequestContext;
use std::collections::HashMap;

let params = HashMap::from([("id".to_string(), "1".to_string())]);

let ctx = RequestContext::new("GET", "https://api.example.com/users")
    .with_header("Accept", "application/json")
    .with_params(params)
    .with_metadata("trace_id", "abc123");
```

### ResponseContext

描述一个已完成响应的上下文：

```rust
use caller::{RequestContext, ResponseContext};

let request = RequestContext::new("GET", "https://api.example.com/users");
let response = ResponseContext::new(200, "{\"ok\":true}".to_string(), request, 42);

assert!(response.is_success());
```

### Middleware trait

实现这个 trait 后，可以在三个阶段插入逻辑：

- `before_request`
- `after_response`
- `on_error`

## 内置 Middleware

### HeaderMiddleware

```rust
use caller::HeaderMiddleware;

let middleware = HeaderMiddleware::new()
    .with_header("X-App", "MyApp")
    .with_header("X-Request-Source", "integration-test");
```

### LoggingMiddleware

```rust
use caller::LoggingMiddleware;

let middleware = LoggingMiddleware::new()
    .with_log_request(true)
    .with_log_response(true);
```

### UserAgentMiddleware

```rust
use caller::UserAgentMiddleware;

let middleware = UserAgentMiddleware::new("MyApp/1.0");
```

### TimingMiddleware

```rust
use caller::TimingMiddleware;

let middleware = TimingMiddleware::new()
    .with_timing_header("x-request-start");
```

### RetryMiddleware

注意：当前 `RetryMiddleware` 主要还是一个可复用的配置/命名单元，真正的网络重试能力目前由 `call_with_retry` / `RetryConfig` 驱动。

```rust
use caller::RetryMiddleware;

let middleware = RetryMiddleware::new()
    .with_max_retries(3)
    .with_status_codes(vec![429, 500, 502, 503, 504]);
```

### CircuitBreakerMiddleware

```rust
use caller::{CircuitBreakerConfig, CircuitBreakerMiddleware};

let config = CircuitBreakerConfig::new()
    .with_failure_threshold(5)
    .with_timeout_ms(30_000)
    .with_success_threshold(2)
    .with_failure_status_codes(vec![500, 502, 503, 504]);

let middleware = CircuitBreakerMiddleware::with_config(config);
```

## 组合 Middleware

可以用 `MiddlewareChain` 按顺序组织多个 middleware：

```rust
use caller::{HeaderMiddleware, LoggingMiddleware, MiddlewareChain, UserAgentMiddleware};

let chain = MiddlewareChain::new()
    .with(HeaderMiddleware::new().with_header("X-App", "MyApp"))
    .with(UserAgentMiddleware::new("MyApp/1.0"))
    .with(LoggingMiddleware::new());
```

手动执行：

```rust
# use caller::{CallerError, HeaderMiddleware, LoggingMiddleware, Middleware, MiddlewareChain, RequestContext, ResponseContext, UserAgentMiddleware};
# async fn demo() -> Result<(), CallerError> {
let chain = MiddlewareChain::new()
    .with(HeaderMiddleware::new().with_header("X-App", "MyApp"))
    .with(UserAgentMiddleware::new("MyApp/1.0"))
    .with(LoggingMiddleware::new());

let mut request = RequestContext::new("GET", "https://api.example.com/users");
chain.before_request(&mut request).await?;

let mut response = ResponseContext::new(200, "{}".to_string(), request, 15);
chain.after_response(&mut response).await?;
# Ok(())
# }
```

## 自定义 Middleware

```rust
use async_trait::async_trait;
use caller::{CallerError, Middleware, RequestContext, ResponseContext};

pub struct TraceMiddleware;

#[async_trait]
impl Middleware for TraceMiddleware {
    async fn before_request(&self, ctx: &mut RequestContext) -> Result<(), CallerError> {
        ctx.metadata
            .insert("trace_id".to_string(), "trace-001".to_string());
        Ok(())
    }

    async fn after_response(&self, ctx: &mut ResponseContext) -> Result<(), CallerError> {
        println!("{}ms", ctx.duration_ms);
        Ok(())
    }

    fn name(&self) -> &str {
        "TraceMiddleware"
    }
}
```

## 适合的使用方式

- 在你自己的上层 SDK 中把 `MiddlewareChain` 接到真正的请求实现上
- 在单元测试里验证 header / metadata / circuit breaker 行为
- 作为后续将 middleware 正式接入 `Caller` 的过渡抽象

## 当前不应假设的能力

目前不要假设这些 middleware 已经：

- 自动被 `Caller::call(...)` 执行
- 自动作用于 `download(...)`
- 自动替代 `RetryConfig`

如果你需要真实请求层面的可组合扩展，这一块还需要后续继续集成。
