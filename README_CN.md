# Caller for Rust

[English](README.md) | 简体中文

`caller` 是面向固定上游 API 的异步、配置驱动 HTTP 客户端。它提供实例级
客户端、分区且类型安全的请求参数、运行时认证、重试策略、middleware、有界
响应缓冲、文件下载、OpenAPI 生成，以及可选的本地开发代理。

## 环境要求

- Rust 1.88 或更高版本
- Tokio runtime
- TLS 使用 rustls，不启用 native-tls

```toml
[dependencies]
caller = "0.4"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## 快速开始

创建 `caller.json`：

```json
{
  "service_items": [{
    "api_name": "users",
    "base_url": "https://api.example.com",
    "timeout": 5000,
    "api_items": [{
      "method": "get",
      "url": "/users/{id}",
      "http_method": "GET",
      "param_type": "path,query"
    }]
  }]
}
```

可复用的应用组件优先使用实例 API：

```rust,no_run
use caller::{Caller, RequestArgs, params};

#[tokio::main]
async fn main() -> Result<(), caller::CallerError> {
    let caller = Caller::from_path("caller.json")?;
    let args = RequestArgs::new()
        .with_path(params! { "id" => "a/b" })
        .with_query(params! { "expand" => true });

    let response = caller.call_args("users.get", args).await?;
    let response = response.error_for_status()?;
    println!("{}，耗时 {:?}", response.status_code, response.duration);
    Ok(())
}
```

路径参数会按单个 path segment 做百分号编码。`RequestArgs` 将 path、query、
form 和 JSON body 分开；`with_query_pairs` / `with_form_pairs` 可保留重复 key。

兼容 API `call` 仍接收一个 `HashMap<String, String>`，并把同一份参数用于所有
配置位置。简单场景或旧代码可以继续使用；组合参数类型应使用 `call_args`。

## 程序化配置

```rust
use caller::{ConfigBuilder, HttpMethod, ParamType};

let mut builder = ConfigBuilder::new();
builder
    .service("users", "https://api.example.com")
    .api_typed("list", "/users", HttpMethod::Get, [ParamType::Query])
    .api_endpoint("update", "/users/{id}")
    .http_method(HttpMethod::Patch)
    .param_types([ParamType::Path, ParamType::Json])
    .timeout(5_000)
    .build()
    .build();

let config = builder.build_validated()?;
# Ok::<(), caller::CallerError>(())
```

支持 GET、POST、PUT、DELETE、PATCH、HEAD 和 OPTIONS。配置可使用 JSON、
YAML 或 TOML；未知字段会直接报错，避免拼写错误被静默忽略。

## 认证

凭据应作为运行时对象提供，不写入配置文件：

```rust,no_run
use caller::{BearerAuth, Caller};

let caller = Caller::from_path("caller.json")?;
caller.register_auth("upstream", BearerAuth::from_env("UPSTREAM_TOKEN")?)?;
# Ok::<(), caller::CallerError>(())
```

在 service 或 endpoint 配置 `authorization_type: "upstream"`。provider 未注册
会在发送前失败。动态 provider 提供可失败的 `try_new`，每次请求尝试都会重新
求值。内置认证类型的 `Debug` 输出会隐藏 token、密码和 API key。

## 重试、Middleware 与大小限制

```rust,no_run
use caller::{Caller, HeaderMiddleware, RetryConfig};
use std::time::Duration;

# fn build() -> Result<(), caller::CallerError> {
let caller = Caller::builder()
    .config_path("caller.json")
    .middleware(HeaderMiddleware::new().with_header("x-app", "billing")?)
    .max_response_body_bytes(8 * 1024 * 1024)
    .max_download_bytes(128 * 1024 * 1024)
    .build()?;

let retry = RetryConfig::new()
    .with_max_retries(3)
    .with_base_delay(Duration::from_millis(250))
    .with_max_delay(Duration::from_secs(10));
# Ok(())
# }
```

重试只覆盖已配置状态码和真实网络错误，不会重试认证、配置或 middleware 拒绝。
默认识别整数秒形式的 `Retry-After`，并受 `max_delay` 限制；指数退避默认加入
20% 正向抖动。middleware 在每次 attempt 外围运行。普通响应默认最多缓冲
16 MiB，下载默认最多 256 MiB，均可
通过 builder 调整。

## 可选开发服务器

```toml
caller = { version = "0.4", features = ["server"] }
```

```bash
cargo run --features server --example server
```

服务器默认绑定回环地址、不开启宽松 CORS，并开启代理路由。非回环绑定必须显式
调用 `allow_remote(true)`。该代理没有用户认证，只适合受信任的本地开发环境，
不要直接暴露到不可信网络。

## 文档

- [配置](docs/configuration_CN.md)
- [JSON、YAML 与 TOML](docs/multi-format-config_CN.md)
- [认证](docs/authentication_CN.md)
- [Middleware](docs/middleware_CN.md)
- [开发服务器](docs/server_CN.md)
- [成熟度与剩余限制](docs/CRATE_REFACTOR_CHECKLIST.md)
- [变更记录](CHANGELOG.md)

## 验证

```bash
cargo fmt --all -- --check
cargo check --all-features --all-targets
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features --all-targets
cargo test --doc
cargo doc --all-features --no-deps
cargo package --allow-dirty
```

标记为 `requires external network access` 的测试默认不执行。

## 许可证

[MIT](LICENSE)
