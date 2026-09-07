[English](middleware_EN.md) | 简体中文

# Middleware 系统

`Caller` 已在真实 HTTP 请求链中执行 middleware。可以通过
`CallerBuilder::middleware`、`middleware_arc` 或 `middleware_chain` 注册。

```rust
use caller::{Caller, HeaderMiddleware, LoggingMiddleware};

let caller = Caller::builder()
    .config_path("caller.json")
    .middleware(HeaderMiddleware::new().with_header("X-App", "my-service")?)
    .middleware(LoggingMiddleware::new())
    .build()?;
# Ok::<(), caller::CallerError>(())
```

## 执行顺序

每次网络尝试都会执行：

```text
构造 RequestContext
  → 按注册顺序执行 before_request
  → 应用参数和认证
  → 发送请求
  → 读取响应
  → 按注册顺序的逆序执行 after_response
```

请求准备、认证、发送、响应读取或 `after_response` 失败时，会调用
`on_error`。`call_with_retry` 会为每次 attempt 重新执行整条链，因此追踪、
动态 Header 和熔断器看到的是真实网络尝试。

middleware 可以修改 `RequestContext` 中的 URL、方法、Header、兼容 `params`、
分区 `path_params`/`query_params`/`form_params` 和 JSON body；响应 middleware
可以在生成 `ApiResult` 前修改状态码、Header 和文本响应体。

当前下载路径会执行请求 hook 和错误 hook，但二进制下载响应尚不会进入
`after_response`，因为现有 `ResponseContext` 仍是文本模型。

## 自定义 Middleware

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

## 内置能力

- `HeaderMiddleware`：添加请求头。
- `UserAgentMiddleware`：覆盖 User-Agent。
- `LoggingMiddleware`：用户显式启用后输出请求、响应和错误信息。
- `TimingMiddleware`：添加 Unix 毫秒时间戳请求 Header。
- `CircuitBreakerMiddleware`：达到失败阈值后阻断请求，并限制为单个 half-open
  探测请求。

真实重试使用 `Caller::call_with_retry`、`call_params_with_retry` 或
`call_args_with_retry` 以及 `RetryConfig`。重试属于执行策略，因为每次 attempt
都必须重建请求并重新应用认证。

## 使用约束

- 熔断器应放在执行成本较高的 middleware 之前。
- middleware 按栈理解：请求 hook 按注册顺序执行，响应和错误 hook 逆序回退。
- 不要记录 Authorization、Cookie、API key 或完整敏感查询串。
- middleware 可能被并发调用，必须满足 `Send + Sync`。
- 启用重试时，请求 body 必须可重放。
