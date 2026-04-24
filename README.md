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
caller = "0.3.3"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
```

### Recommended: Instance API

```rust
use caller::Caller;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let caller = Caller::from_path("caller.json")?;
    let result = caller.call("JP.list", None).await?;
    println!("Status: {}", result.status_code);
    Ok(())
}
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
    if let Some(title) = result.str_at("0.title") {
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
| [Configuration](docs/configuration.md) | Config file format, multi-format support, hot reload |
| [Multi-Format Config](docs/MULTI_FORMAT_CONFIG.md) | JSON / YAML / TOML loading, conversion, validation |
| [Authentication](docs/authentication.md) | Auth types, dynamic tokens, runtime updates |
| [Middleware](docs/middleware.md) | Middleware primitives, current scope, extension patterns |
| [API Server](docs/server.md) | Swagger UI, OpenAPI generation, current proxy limitations |
| [Refactor Checklist](docs/CRATE_REFACTOR_CHECKLIST.md) | Current crate maturity, completed work, next priorities |

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

→ [Full Authentication Guide](docs/authentication.md)

## API Documentation Server

Enable the `server` feature for Swagger UI:

```toml
[dependencies]
caller = { version = "0.3.3", features = ["server"] }
```

```bash
cargo run --features server --example server
# Open http://localhost:8080 for Swagger UI
```

→ [Server Documentation](docs/server.md)

## Configuration Example

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

→ [Configuration Guide](docs/configuration.md)

### Programmatic Config Example

```rust
use caller::{ConfigBuilder, HttpMethod, ParamType};

let mut builder = ConfigBuilder::new();
builder
    .service("JP", "https://jsonplaceholder.typicode.com")
    .api_typed("list", "/posts", HttpMethod::Get, [ParamType::Query])
    .api_endpoint("update", "/posts/{id}")
    .http_method(HttpMethod::Patch)
    .param_types([ParamType::Path, ParamType::Json])
    .description("Update a post")
    .build()
    .build();

let config = builder.build();
```

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
result.body           // ResponseBody::Json / Text / Bytes
result.json          // JSON Value, or null for non-JSON responses

if result.is_json() {
    result.value_at("0.title");          // Get JSON value
    result.str_at("0.title");   // Get as &str
    result.i64_at("0.userId");  // Get as i64
}

if let Some(text) = result.text() {
    println!("{}", text);
}
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

# Instance-based client
cargo run --example instance_client

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
