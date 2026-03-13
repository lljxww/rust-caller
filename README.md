# Caller For Rust

A flexible, configurable Web API request library built with Rust, supporting multiple API endpoints management through JSON configuration files.

## Features

- 🚀 **Async Calls**: Built on Tokio async runtime, supports high concurrency requests
- ⚡ **Configuration Management**: Define API endpoints through JSON configuration files
- 🔄 **Hot Reload**: Support for configuration file hot reload, update configuration without restarting
- 🔐 **Authentication Support**: Built-in multiple authentication mechanisms (header, query, basic auth)
- 📊 **Result Parsing**: Powerful JSON result parsing with deep path access
- 🛡️ **Error Handling**: Comprehensive error types and error handling mechanisms
- 🧪 **Test Coverage**: Built-in comprehensive unit and integration tests
- 🏗️ **Modular Architecture**: Clear layered architecture, easy to extend and maintain

## Architecture Design

### Project Structure
```
src/
├── core/          # Core business logic
│   ├── context.rs # Call context management
│   └── constants.rs # Constant definitions
├── domain/        # Domain models
│   ├── api_item.rs      # API item definitions
│   ├── api_result.rs    # API response results
│   ├── authorization.rs # Authentication related
│   ├── caller_config.rs # Call configuration
│   └── service_item.rs  # Service item definitions
├── config/        # Configuration management
│   └── config_loader.rs # Configuration loader
├── infra/         # Infrastructure layer
│   └── http.rs     # HTTP client wrapper
└── shared/        # Shared modules
    └── error.rs    # Error definitions
```

### Workflow
1. **Configuration Loading**: Load API configuration from JSON file
2. **Context Management**: Manage call context and middleware
3. **HTTP Call**: Construct and send HTTP requests
4. **Result Processing**: Parse and format response results

## Quick Start

### Add Dependency

```toml
[dependencies]
caller = "0.2.0"
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage

```rust
use caller::call;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple call
    let result = call("JP.list", None).await?;

    // Parse result
    println!("Status: {}", result.status_code);
    println!("Raw response: {}", result.raw);

    // Get specific field
    if let Some(first_title) = result.get_as_str("0.title") {
        println!("First post title: {}", first_title);
    }

    // Call with parameters
    let params = HashMap::from([
        ("post_id".to_string(), "1".to_string()),
        ("userId".to_string(), "1".to_string()),
    ]);
    let result = call("JP.get", Some(params)).await?;

    Ok(())
}
```

## Configuration File

Create a `caller.json` configuration file to define API endpoints:

```json
{
  "Authorizations": [
    {
      "Name": "BearerToken",
      "HeaderName": "Authorization",
      "Type": "Bearer",
      "Token": "your-token-here"
    }
  ],
  "ServiceItems": [
    {
      "ApiName": "weibo",
      "BaseUrl": "https://weibo.com/ajax",
      "ApiItems": [
        {
          "Method": "hot",
          "Url": "/side/hotSearch",
          "HttpMethod": "GET",
          "ParamType": "query",
          "Description": "Get Weibo hot search",
          "Timeout": 5000,
          "NeedCache": true,
          "CacheTime": 300
        }
      ]
    },
    {
      "ApiName": "JP",
      "BaseUrl": "https://jsonplaceholder.typicode.com",
      "ApiItems": [
        {
          "Method": "get",
          "Url": "/posts/{post_id}",
          "HttpMethod": "GET",
          "ParamType": "path"
        },
        {
          "Method": "create",
          "Url": "/posts",
          "HttpMethod": "POST",
          "ParamType": "json",
          "ContentType": "application/json",
          "AuthorizationType": "BearerToken"
        }
      ]
    }
  ]
}
```

### Parameter Types

| Parameter Type | Description | Example |
|---------------|-------------|----------|
| `none` | No parameters | `/posts` |
| `query` | Query parameters | `/posts?userId=1&id=1` |
| `path` | Path parameters | `/posts/1` |
| `json` | JSON request body | `{"title": "Hello", "body": "World"}` |
| `path,json` | Path parameters + JSON body | `/posts/1` + `{"title": "Updated"}` |

## API Reference

### Main Types

#### `ApiResult`
```rust
pub struct ApiResult {
    pub status_code: StatusCode,
    pub raw: String,
    pub j_obj: Value,
}
```

#### `CallerError`
```rust
pub enum CallerError {
    ConfigError(String),
    HttpError(reqwest::Error),
    JsonError(String),
    ServiceNotFound(String),
    MethodNotFound(String),
    ParamMissing(String),
    AuthenticationError(String),
}
```

### Core Methods

#### `api_result.get_as_str(key: &str) -> Option<&str>`
Get string value, supports deep path like `"0.title"`

#### `api_result.get_as_i64(key: &str) -> Option<i64>`
Get integer value

#### `api_result.get_as_bool(key: &str) -> Option<bool>`
Get boolean value

#### `api_result.get(key: &str) -> Option<&Value>`
Get native JSON value

### Configuration Hot Reload

Caller supports configuration file hot reload, update API configuration without restarting the program.

```rust
use caller::{call, init_config, reload_config, watch_config, stop_watch_config, is_watching_config};

// Initialize configuration (usually called when application starts)
init_config()?;

// Manually reload configuration
reload_config()?;

// Start file watching, automatically detect configuration file changes
// Use default 500ms debounce time
watch_config()?;

// Or use custom debounce time (minimum 100ms)
use std::time::Duration;
watch_config_with_debounce(Duration::from_millis(1000))?;

// Check if watching
if is_watching_config() {
    println!("Configuration file is being watched");
}

// Stop watching
stop_watch_config();
```

### Public API

#### `call(method: &str, params: Option<HashMap<String, String>>) -> Result<ApiResult, CallerError>`
```rust
use caller::call;
use std::collections::HashMap;

// Call without parameters
let result = call("JP.list", None).await?;

// Call with parameters
let params = HashMap::from([
    ("post_id".to_string(), "1".to_string()),
]);
let result = call("JP.get", Some(params)).await?;
```

## Testing

Run all tests:

```bash
cargo test
```

Run specific tests:

```bash
cargo test test_call_list_posts
```

## Build and Release

```bash
# Build
cargo build

# Run examples
cargo run --example basic_usage

# Format code
cargo fmt

# Check code
cargo check
```

## Contributing

1. Fork this repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Create a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details

## Changelog

### v0.2.0
- **Multi-format Configuration Support**: Add support for JSON, YAML, and TOML configuration file formats
- **Configuration Format Conversion**: Add configuration file format conversion capabilities (convert between JSON, YAML, and TOML)
- **Serialization Support**: Add Serialize trait to all configuration structures
- **Complete Examples**: Create complete configuration file examples in all three formats
- **Comprehensive Tests**: Add 16 multi-format configuration test cases
- **Documentation**: Update documentation to explain multi-format support and format conversion

### v0.1.0
- Initial release
- Support for configuration-based API calls
- Support for multiple HTTP methods and parameter types
- Built-in authentication and caching support
- Comprehensive error handling
- Full test coverage