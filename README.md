# Caller For Rust

English | [简体中文](https://github.com/lljxww/rust-caller/blob/main/README_CN.md)

A flexible, configurable Web API request library built with Rust.

## Features

- 🚀 **Async Calls**: Built on Tokio, supports high concurrency
- ⚙️ **Configuration Management**: JSON/YAML/TOML config files
- 🔐 **Authentication**: Multiple auth types with dynamic token support
- 📊 **OpenAPI Generation**: Generate API documentation automatically
- 🌐 **Swagger UI Server**: Built-in API testing interface
- 🔄 **Hot Reload**: Update config without restart
- 📥 **File Download**: Auto format detection
- 🔁 **Retry Mechanism**: Exponential backoff retry

## Quick Start

### Add Dependency

```toml
[dependencies]
caller = "0.3.0"
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage

```rust
use caller::{init_config, call};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize configuration
    init_config()?;

    // Simple API call
    let result = call("JP.list", None).await?;
    println!("Status: {}", result.status_code);

    // Get nested field
    if let Some(title) = result.get_as_str("0.title") {
        println!("First post: {}", title);
    }

    // Call with parameters
    let params = HashMap::from([
        ("id".to_string(), "1".to_string()),
    ]);
    let result = call("JP.get", Some(params)).await?;

    Ok(())
}
```

## Documentation

| Topic | Description |
|-------|-------------|
| [Configuration](docs/configuration_EN.md) | Config file format, multi-format support, hot reload |
| [Authentication](docs/authentication_EN.md) | Auth types, dynamic tokens, runtime updates |
| [Middleware](docs/middleware_EN.md) | Request/response interception, circuit breaker, logging |
| [API Server](docs/server_EN.md) | Swagger UI, OpenAPI generation, proxy testing |

## Authentication

```rust
use caller::{register_auth, BearerAuth, DynamicBearerAuth};
use std::sync::{Arc, RwLock};

// Static token
register_auth("my_api", BearerAuth::new("token".to_string()))?;

// From environment variable
register_auth("github", BearerAuth::from_env("GITHUB_TOKEN")?)?;

// Dynamic token (refreshable)
let token = Arc::new(RwLock::new("initial".to_string()));
register_auth("dynamic", DynamicBearerAuth::from_shared(token.clone()))?;

// Update at runtime
*token.write().unwrap() = "refreshed-token".to_string();
```

→ [Full Authentication Guide](docs/authentication_EN.md)

## API Documentation Server

Enable the `server` feature for Swagger UI:

```toml
[dependencies]
caller = { version = "0.3.0", features = ["server"] }
```

```bash
cargo run --features server --example server
# Open http://localhost:8080 for Swagger UI
```

→ [Server Documentation](docs/server_EN.md)

## Configuration Example

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

→ [Configuration Guide](docs/configuration_EN.md)

## API Reference

### Core Functions

```rust
// Basic API call
call(method, params) -> Result<ApiResult>

// With retry
call_with_retry(method, params, retry_config) -> Result<ApiResult>

// File download
download(method, params, extension) -> Result<DownloadResult>
```

### ApiResult

```rust
let result = call("JP.list", None).await?;

result.status_code     // HTTP status
result.raw            // Raw response string
result.j_obj          // JSON Value

result.get("0.title")           // Get JSON value
result.get_as_str("0.title")    // Get as &str
result.get_as_i64("0.userId")   // Get as i64
```

### Retry Configuration

```rust
use caller::RetryConfig;
use std::time::Duration;

let retry = RetryConfig::new()
    .with_max_retries(3)
    .with_base_delay(Duration::from_millis(500))
    .with_max_delay(Duration::from_secs(10));
```

## Examples

```bash
# Basic usage
cargo run --example basic_usage

# Combined parameters
cargo run --example combined_params

# API documentation server
cargo run --features server --example server
```

## Testing

```bash
# Run all tests
cargo test

# Run with server feature
cargo test --features server
```

## License

MIT License - see [LICENSE](LICENSE) file.
