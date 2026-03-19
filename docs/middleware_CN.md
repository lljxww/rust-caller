[English](middleware_EN.md) | 简体中文

# 中间件系统

Caller 库提供了强大的中间件系统，允许你拦截和修改请求与响应。

## 概述

中间件可以：
- 在发送请求之前修改请求
- 在接收响应之后处理响应
- 处理请求执行过程中的错误
- 实现日志、重试、熔断等横切关注点

## 内置中间件

### HeaderMiddleware

为所有请求添加自定义请求头：

```rust
use caller::domain::middleware::{HeaderMiddleware, MiddlewareChain, RequestContext};

let middleware = HeaderMiddleware::new()
    .with_header("X-API-Key", "secret-key")
    .with_header("X-Request-Id", "12345");
```

### LoggingMiddleware

记录请求和响应：

```rust
use caller::domain::middleware::LoggingMiddleware;

let middleware = LoggingMiddleware::new()
    .with_log_request(true)
    .with_log_response(true);
```

输出：
```
[Request] GET https://api.example.com/users
[Response] GET https://api.example.com/users - 200 (45ms)
```

### UserAgentMiddleware

添加自定义 User-Agent 请求头：

```rust
use caller::domain::middleware::UserAgentMiddleware;

let middleware = UserAgentMiddleware::new("MyApp/1.0");
```

### TimingMiddleware

为请求添加计时/追踪信息：

```rust
use caller::domain::middleware::TimingMiddleware;

let middleware = TimingMiddleware::new()
    .with_timing_header("x-request-start");
```

### RetryMiddleware

重试失败的请求：

```rust
use caller::domain::middleware::RetryMiddleware;

let middleware = RetryMiddleware::new()
    .with_max_retries(3)
    .with_status_codes(vec![429, 500, 502, 503, 504]);
```

### CircuitBreakerMiddleware

通过停止向失败的服务发送请求来防止级联故障：

```rust
use caller::domain::middleware::{CircuitBreakerMiddleware, CircuitBreakerConfig};

let config = CircuitBreakerConfig::new()
    .with_failure_threshold(5)      // 5次失败后打开
    .with_timeout_ms(30000)          // 30秒后重试
    .with_success_threshold(2)       // 2次成功后关闭
    .with_failure_status_codes(vec![500, 502, 503, 504]);

let middleware = CircuitBreakerMiddleware::with_config(config);
```

#### 熔断器状态

| 状态 | 描述 |
|------|------|
| **Closed** | 正常操作，请求通过 |
| **Open** | 请求被阻止，等待超时 |
| **HalfOpen** | 测试服务是否已恢复 |

#### 监控

```rust
// 获取当前状态
let state = middleware.state();

// 获取统计信息
let stats = middleware.stats();
println!("失败次数: {}", stats.failure_count);
println!("阻止次数: {}", stats.blocked_count);

// 重置熔断器
middleware.reset();
```

## 创建自定义中间件

实现 `Middleware` trait：

```rust
use caller::domain::middleware::{Middleware, RequestContext, ResponseContext};
use caller::CallerError;
use async_trait::async_trait;

pub struct MyMiddleware;

#[async_trait]
impl Middleware for MyMiddleware {
    async fn before_request(&self, ctx: &mut RequestContext) -> Result<(), CallerError> {
        // 在发送之前修改请求
        ctx.headers.insert(
            "x-custom-header".parse().unwrap(),
            "value".parse().unwrap()
        );
        Ok(())
    }

    async fn after_response(&self, ctx: &mut ResponseContext) -> Result<(), CallerError> {
        // 在接收之后处理响应
        println!("响应时间: {}ms", ctx.duration_ms);
        Ok(())
    }

    async fn on_error(&self, error: &CallerError, ctx: &RequestContext) {
        // 处理错误
        eprintln!("请求失败: {}", error);
    }

    fn name(&self) -> &str {
        "MyMiddleware"
    }
}
```

## 中间件链

组合多个中间件：

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

// 在请求之前执行
chain.before_request(&mut ctx).await?;

// 在响应之后执行
chain.after_response(&mut response_ctx).await?;
```

## 请求上下文

`RequestContext` 包含关于传出请求的所有信息：

```rust
let ctx = RequestContext::new("GET", "https://api.example.com/users")
    .with_header("Accept", "application/json")
    .with_body(r#"{"key": "value"}"#.to_string())
    .with_params(params)
    .with_metadata("trace_id", "abc123");
```

## 响应上下文

`ResponseContext` 包含关于已接收响应的所有信息：

```rust
// 状态检查
if response.is_success() {
    // 2xx 状态码
}
if response.is_client_error() {
    // 4xx 状态码
}
if response.is_server_error() {
    // 5xx 状态码
}

// 访问数据
println!("状态: {}", response.status_code);
println!("响应体: {}", response.body);
println!("持续时间: {}ms", response.duration_ms);
```

## 最佳实践

1. **顺序很重要**：中间件按照添加到链中的顺序执行
2. **保持简单**：每个中间件应该做好一件事
3. **处理错误**：返回错误以中止请求链
4. **使用元数据**：在 `ctx.metadata` 中存储请求范围的数据
5. **异步安全**：中间件必须是 `Send + Sync`

## 常见模式

### 请求 ID 追踪

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

### 限流

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