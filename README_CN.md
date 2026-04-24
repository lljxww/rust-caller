# Caller For Rust

[English](https://github.com/lljxww/rust-caller/blob/main/README.md) | 简体中文

一个灵活、可配置的 Rust Web API 请求库。

## 特性

- 🚀 **异步调用**: 基于 Tokio，支持高并发
- ⚙️ **配置管理**: 支持 JSON/YAML/TOML 配置文件
- 🔐 **认证系统**: 多种认证类型，支持动态 Token
- 📊 **OpenAPI 生成**: 自动生成 API 文档
- 🌐 **Swagger UI 服务器**: 内置 API 测试界面
- 🔄 **热重载**: 无需重启更新配置
- 📥 **文件下载**: 自动格式检测
- 🔁 **重试机制**: 指数退避重试

## 快速开始

### 添加依赖

```toml
[dependencies]
caller = "0.3.3"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
```

### 推荐：实例化 API

```rust
use caller::Caller;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let caller = Caller::from_path("caller.json")?;
    let result = caller.call("JP.list", None).await?;
    println!("状态码: {}", result.status_code);
    Ok(())
}
```

### 基本使用

```rust
use caller::{init_config, call};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化配置
    init_config()?;

    // 简单调用
    let result = call("JP.list", None).await?;

    // 解析结果
    println!("状态码: {}", result.status_code);
    println!("原始响应: {}", result.raw);

    // 获取特定字段
    if let Some(title) = result.str_at("0.title") {
        println!("第一篇文章: {}", title);
    }

    // 带参数调用
    let params = HashMap::from([
        ("id".to_string(), "1".to_string()),
    ]);
    let result = call("JP.get", Some(params)).await?;

    Ok(())
}
```

## 文档

| 主题 | 描述 |
|------|------|
| [配置文件](docs/configuration.md) | 配置格式、多格式支持、热重载 |
| [多格式配置](docs/MULTI_FORMAT_CONFIG.md) | JSON / YAML / TOML 加载、转换、校验 |
| [认证系统](docs/authentication.md) | 认证类型、动态 Token、运行时更新 |
| [Middleware](docs/middleware.md) | middleware 基础类型、当前作用范围、扩展方式 |
| [API 服务器](docs/server.md) | Swagger UI、OpenAPI 生成、当前代理限制 |
| [重构清单](docs/CRATE_REFACTOR_CHECKLIST.md) | 当前成熟度、已完成工作、后续优先级 |

## 认证

```rust
use caller::{register_auth, BearerAuth, DynamicBearerAuth};
use std::sync::{Arc, RwLock};

// 静态 Token
register_auth("my_api", BearerAuth::new("token".to_string()))?;

// 从环境变量
register_auth("github", BearerAuth::from_env("GITHUB_TOKEN")?)?;

// 动态 Token（可刷新）
let token = Arc::new(RwLock::new("initial".to_string()));
register_auth("dynamic", DynamicBearerAuth::from_shared(token.clone()))?;

// 运行时更新
*token.write().unwrap() = "refreshed-token".to_string();
```

→ [完整认证指南](docs/authentication.md)

## API 文档服务器

启用 `server` 功能获取 Swagger UI：

```toml
[dependencies]
caller = { version = "0.3.3", features = ["server"] }
```

```bash
cargo run --features server --example server
# 访问 http://localhost:8080 查看 Swagger UI
```

→ [服务器文档](docs/server.md)

## 配置示例

`caller.json`:
```json
{
  "service_items": [
    {
      "api_name": "JP",
      "base_url": "https://jsonplaceholder.typicode.com",
      "api_items": [
        {
          "method": "list",
          "url": "/posts",
          "http_method": "GET",
          "param_type": "query"
        },
        {
          "method": "get",
          "url": "/posts/{id}",
          "http_method": "GET",
          "param_type": "path"
        }
      ]
    }
  ]
}
```

→ [配置指南](docs/configuration.md)

### 程序化配置示例

```rust
use caller::{ConfigBuilder, HttpMethod, ParamType};

let mut builder = ConfigBuilder::new();
builder
    .service("JP", "https://jsonplaceholder.typicode.com")
    .api_typed("list", "/posts", HttpMethod::Get, [ParamType::Query])
    .api_endpoint("update", "/posts/{id}")
    .http_method(HttpMethod::Patch)
    .param_types([ParamType::Path, ParamType::Json])
    .description("更新文章")
    .build()
    .build();

let config = builder.build();
```

## API 参考

### 核心函数

```rust
// 基本 API 调用
call(method, params) -> Result<ApiResult>

// 带重试
call_with_retry(method, params, retry_config) -> Result<ApiResult>

// 文件下载
download(method, params, extension) -> Result<DownloadResult>
```

### ApiResult

```rust
let result = call("JP.list", None).await?;

result.status_code     // HTTP 状态码
result.raw            // 原始响应字符串
result.body           // ResponseBody::Json / Text / Bytes
result.json          // JSON Value，非 JSON 响应时为 null

if result.is_json() {
    result.value_at("0.title");          // 获取 JSON 值
    result.str_at("0.title");   // 获取字符串
    result.i64_at("0.userId");  // 获取整数
}

if let Some(text) = result.text() {
    println!("{}", text);
}
```

### 重试配置

```rust
use caller::RetryConfig;
use std::time::Duration;

let retry = RetryConfig::new()
    .with_max_retries(3)
    .with_base_delay(Duration::from_millis(500))
    .with_max_delay(Duration::from_secs(10));
```

## 示例

```bash
# 基本使用
cargo run --example basic_usage

# 组合参数
cargo run --example combined_params

# 实例化客户端
cargo run --example instance_client

# API 文档服务器
cargo run --features server --example server
```

## 测试

```bash
# 运行所有测试
cargo test

# 带 server 功能测试
cargo test --features server
```

## 许可证

MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。
