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
caller = "0.3.0"
tokio = { version = "1.0", features = ["full"] }
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
    if let Some(title) = result.get_as_str("0.title") {
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
| [认证系统](docs/authentication.md) | 认证类型、动态 Token、运行时更新 |
| [API 服务器](docs/server.md) | Swagger UI、OpenAPI 生成、代理测试 |

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
caller = { version = "0.3.0", features = ["server"] }
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
  "ServiceItems": [
    {
      "ApiName": "JP",
      "BaseUrl": "https://jsonplaceholder.typicode.com",
      "ApiItems": [
        {
          "Method": "list",
          "Url": "/posts",
          "HttpMethod": "GET",
          "ParamType": "query"
        },
        {
          "Method": "get",
          "Url": "/posts/{id}",
          "HttpMethod": "GET",
          "ParamType": "path"
        }
      ]
    }
  ]
}
```

→ [配置指南](docs/configuration.md)

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
result.j_obj          // JSON Value

result.get("0.title")           // 获取 JSON 值
result.get_as_str("0.title")    // 获取字符串
result.get_as_i64("0.userId")   // 获取整数
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
