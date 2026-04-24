[English](server_EN.md) | 简体中文

# API 文档服务器

`caller` 提供一个可选的内置 HTTP 服务器，用于：

- 暴露 Swagger UI
- 导出 OpenAPI JSON
- 通过代理入口调试配置里的 API

## 启用方式

在 `Cargo.toml` 中启用 `server` feature：

```toml
[dependencies]
caller = { version = "0.3.3", features = ["server"] }
```

## 启动服务器

### 使用示例程序

```bash
cargo run --features server --example server
```

### 在代码中启动

```rust
use caller::{Caller, ServerConfig, start_server_with_caller};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let caller = Caller::from_path("caller.json")?;

    let config = ServerConfig::new()
        .addr("127.0.0.1:8080")?
        .title("My API")
        .version("1.0.0");

    start_server_with_caller(config, caller).await?;
    Ok(())
}
```

## 当前端点

| 端点 | 说明 |
|------|------|
| `GET /` | Swagger UI 页面 |
| `GET /openapi.json` | OpenAPI 3.0 JSON 文档 |
| `GET /proxy/{service}/{method}` | 代理调用入口 |
| `GET /proxy?service=...&method=...` | 查询参数形式的代理入口 |

## Swagger UI

启动后访问：

```text
http://127.0.0.1:8080/
```

Swagger UI 会读取 `/openapi.json` 并展示当前配置里的服务和方法。

## OpenAPI 生成

也可以不启动 HTTP 服务器，直接在代码里生成：

```rust
use caller::{Caller, OpenApiGenerator};
use std::fs;

let caller = Caller::from_path("caller.json")?;

let generator = OpenApiGenerator::from_caller(&caller)?
    .title("My API")
    .version("1.0.0")
    .description("API documentation");

fs::write("openapi.json", generator.to_json()?)?;
fs::write("openapi.yaml", generator.to_yaml()?)?;
```

## 代理入口怎么工作

代理入口会：

1. 根据 `service` 和 `method` 找到配置中的 API 项
2. 读取该 API 的 `http_method`
3. 由服务器向目标上游发起对应 HTTP 方法的请求

例如：

```bash
curl "http://localhost:8080/proxy/JP/list"
curl "http://localhost:8080/proxy/JP/get?id=1"
curl "http://localhost:8080/proxy?service=JP&method=get&id=1"
```

## 当前限制

这里需要特别注意，当前代理服务器能力是“可调试”，不是“完整 API 网关”。

当前实现的限制包括：

- 对外暴露的代理路由目前只有 `GET`
- 更适合调试 `none` / `query` / `path` 这类参数模型
- 对 `json` / `form` 请求体的代理支持还不完整
- Swagger UI 对 body-based API 的 “Try it out” 体验当前可能与真实上游能力不完全一致

因此：

- 用它浏览和验证文档、路径、查询参数接口是合适的
- 对需要请求体的写操作接口，更适合用 `Caller` 实例或你自己的测试客户端直接调用

## 代理响应格式

成功时：

- 如果上游响应是 JSON，服务器会直接返回 JSON
- 如果上游响应不是 JSON，会包装成 `{ "response": "..." }`

错误时会返回结构化 JSON，例如：

```json
{
  "error": "Service 'UnknownService' not found",
  "available_services": ["JP", "GitHub"]
}
```

## 自定义服务配置

### 监听地址

```rust
let config = ServerConfig::new()
    .addr("0.0.0.0:3000")?;
```

### 标题和版本

```rust
let config = ServerConfig::new()
    .title("My Company API")
    .version("2.0.0");
```

## CORS

服务器默认开启宽松 CORS，便于本地浏览器和 Swagger UI 调试。

## 适合的使用场景

- 本地查看由配置生成的 OpenAPI 文档
- 给测试同学或前端快速暴露一份 Swagger UI
- 验证路径参数和查询参数接口
- 导出 OpenAPI JSON 供 Postman / Insomnia 导入
