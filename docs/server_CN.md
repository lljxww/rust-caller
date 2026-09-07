[English](server_EN.md) | 简体中文

# 本地开发服务器

可选 `server` feature 提供 OpenAPI、Swagger UI 和 HTTP 代理，代理复用同一条
`Caller` 请求执行链。

```toml
caller = { version = "0.4", features = ["server"] }
```

```rust,no_run
use caller::{Caller, ServerConfig, start_server_with_caller};

#[tokio::main]
async fn main() -> Result<(), caller::CallerError> {
    let caller = Caller::from_path("caller.json")?;
    let server = ServerConfig::new()
        .addr("127.0.0.1:8080")?
        .title("Upstream APIs")
        .version("1.0.0");
    start_server_with_caller(server, caller).await
}
```

路由：

| 路由 | 作用 |
|---|---|
| `GET /` | Swagger UI |
| `GET /openapi.json` | OpenAPI 3.0 文档 |
| `METHOD /proxy/{service}/{method}` | 使用 endpoint 配置的 HTTP 方法转发 |
| `METHOD /proxy?service=...&method=...` | query 风格代理入口 |

代理会校验入站方法，把 path/query/form/JSON 按配置分区，保留 JSON body 类型，
并通过 `Caller` 应用运行时认证、middleware、超时、响应大小限制和连接复用。

路径参数以占位符同名 query 字段传入；兼容字段 `id` 会映射到第一个路径占位符。
form endpoint 要求 `application/x-www-form-urlencoded`。JSON body 可以是任意
JSON 值。

## 安全默认值

- 默认监听 `127.0.0.1:8080`。
- 非回环地址必须显式 `allow_remote(true)`，否则拒绝启动。
- 通配 CORS 默认关闭，只有 `allow_any_origin(true)` 才会开启。
- `enable_proxy(false)` 可完全移除代理路由，仍保留 OpenAPI 与 Swagger UI。
- 内置服务器没有入站认证、授权、限流、TLS 或持久审计。

`allow_remote(true)` 可能通过通用代理暴露所有已注册的上游凭据。除非前置网关
已经实现可靠的认证、授权、限流和 TLS 终止，否则不要在不可信网络启用。该服务
定位是开发工具，不是生产 API 网关。

Swagger UI 静态资源从 `unpkg.com` 加载；离线或严格 CSP 环境应自行提供前端，
并读取 `/openapi.json`。

## OpenAPI 认证模型

运行时 authenticator 可以执行任意 Rust 逻辑，生成器无法可靠推断其认证类型。
默认会生成带说明的 bearer 占位符；非 bearer provider 应显式覆盖：

```rust,no_run
use caller::{OpenApiGenerator, SecurityScheme};

# fn build(caller: &caller::Caller) -> Result<(), caller::CallerError> {
let document = OpenApiGenerator::from_caller(caller)?
    .security_scheme("api_key", SecurityScheme::api_key("x-api-key", "header"))
    .generate();
# Ok(())
# }
```

当前 endpoint 配置只描述参数位置，不描述具体 query 参数名和 schema，因此生成的
query schema 只能是通用对象。
