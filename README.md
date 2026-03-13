# Caller For Rust

English | [简体中文](https://github.com/lljxww/rust-caller/blob/main/README_CN.md)

A flexible, configurable Web API request library built with Rust, supporting multiple API endpoints management through configuration files.

## Features

- 🚀 **Async Calls**: Built on Tokio async runtime, supports high concurrency requests
- ⚡ **Configuration Management**: Define API endpoints through configuration files (JSON/YAML/TOML)
- 🔄 **Hot Reload**: Support for configuration file hot reload, update configuration without restarting
- 📥 **File Download**: Built-in file download functionality with automatic format detection
- 🔁 **Retry Mechanism**: Automatic retry with exponential backoff for failed requests
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
│   ├── download_result.rs # Download result handling
│   ├── retry_config.rs  # Retry configuration
│   └── service_item.rs  # Service item definitions
├── config/        # Configuration management
│   └── config_loader.rs # Configuration loader
├── infra/         # Infrastructure layer
│   └── http.rs     # HTTP client wrapper
└── shared/        # Shared modules
    └── error.rs    # Error definitions
```

### Workflow
1. **Configuration Loading**: Load API configuration from file (JSON/YAML/TOML)
2. **Context Management**: Manage call context and middleware
3. **HTTP Call**: Construct and send HTTP requests (with optional retry)
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

### Call with Retry

```rust
use caller::{call_with_retry, RetryConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create retry configuration
    let retry_config = RetryConfig::new()
        .with_max_retries(3)
        .with_base_delay(Duration::from_millis(500))
        .with_max_delay(Duration::from_secs(30));

    // Call with automatic retry
    let result = call_with_retry("JP.list", None, retry_config).await?;
    
    Ok(())
}
```

### File Download

```rust
use caller::download;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Download file with auto-detected format
    let result = download("api.download", None, None).await?;
    println!("Downloaded {} bytes", result.size_human());
    
    // Save to file
    result.save("./downloads", "myfile")?;
    
    // Download with specified file extension
    let result = download("api.pdf", None, Some("pdf".to_string())).await?;
    result.save("./downloads", "document")?;
    
    Ok(())
}
```

## Configuration File

Create a configuration file to define API endpoints. Supports JSON, YAML, and TOML formats.

### JSON Example (caller.json)
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

#### `DownloadResult`
```rust
pub struct DownloadResult {
    pub status_code: StatusCode,
    pub content: Vec<u8>,
    pub content_type: String,
    pub file_extension: String,
    pub suggested_filename: Option<String>,
}
```

#### `RetryConfig`
```rust
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub retry_status_codes: Vec<u16>,
    pub retry_on_network_error: bool,
}
```

#### `CallerError`
```rust
pub enum CallerError {
    ConfigError(String),
    HttpError(reqwest::Error),
    JsonError(String),
    IoError(String),
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

#### `download_result.save<P: AsRef<Path>>(directory: P, base_name: &str) -> Result<String, CallerError>`
Save downloaded content to file, returns the filename

#### `download_result.size_human() -> String`
Get human-readable file size (e.g., "1.23 MB")

### Public API Functions

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

#### `call_with_retry(method: &str, params: Option<HashMap<String, String>>, retry_config: RetryConfig) -> Result<ApiResult, CallerError>`
```rust
use caller::{call_with_retry, RetryConfig};
use std::time::Duration;

let retry_config = RetryConfig::new()
    .with_max_retries(3)
    .with_base_delay(Duration::from_millis(500));

let result = call_with_retry("JP.list", None, retry_config).await?;
```

#### `download(method: &str, params: Option<HashMap<String, String>>, extension: Option<String>) -> Result<DownloadResult, CallerError>`
```rust
use caller::download;

// Download with auto-detected format
let result = download("api.download", None, None).await?;
result.save("./downloads", "file")?;

// Download with specified extension
let result = download("api.pdf", None, Some("pdf".to_string())).await?;
```

### Configuration Management

#### `init_config() -> Result<(), CallerError>`
Initialize configuration by loading from file

#### `reload_config() -> Result<(), CallerError>`
Manually reload configuration from file

#### `watch_config() -> Result<(), CallerError>`
Start watching config file for changes (500ms debounce)

#### `watch_config_with_debounce(debounce: Duration) -> Result<(), CallerError>`
Start watching with custom debounce time

#### `stop_watch_config()`
Stop watching config file

#### `is_watching_config() -> bool`
Check if config is currently being watched

#### `is_config_loaded() -> bool`
Check if configuration is loaded

```rust
use caller::{init_config, watch_config, stop_watch_config, is_watching_config};

// Initialize configuration
init_config()?;

// Start file watching
watch_config()?;

// Check status
if is_watching_config() {
    println!("Configuration file is being watched");
}

// Stop watching
stop_watch_config();
```

## Multi-Format Configuration Support

Caller supports multiple configuration file formats:

### Supported Formats
- **JSON** (`.json`)
- **YAML** (`.yaml`, `.yml`)
- **TOML** (`.toml`)

### Format Conversion

```rust
use caller::config::config_loader::{ConfigLoader, ConfigFormat};

// Convert between formats
ConfigLoader::convert_config("config.json", "config.yaml")?;
ConfigLoader::convert_config("config.yaml", "config.toml")?;

// Explicit format specification
ConfigLoader::convert_config_with_format(
    "config.json",
    "output.txt",
    ConfigFormat::Yaml
)?;
```

See [docs/MULTI_FORMAT_CONFIG.md](docs/MULTI_FORMAT_CONFIG.md) for detailed documentation.

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
- **File Download**: Add file download functionality with automatic format detection
- **Retry Mechanism**: Add automatic retry with exponential backoff for failed requests
- **Retry Configuration**: Add configurable retry behavior with `RetryConfig`
- **Download Result**: Add comprehensive download result handling with `DownloadResult`
- **Serialization Support**: Add Serialize trait to all configuration structures
- **Complete Examples**: Create complete configuration file examples in all three formats
- **Comprehensive Tests**: Add 16 multi-format configuration test cases
- **Documentation**: Update documentation to explain new features and multi-format support

### v0.1.0
- Initial release
- Support for configuration-based API calls
- Support for multiple HTTP methods and parameter types
- Built-in authentication and caching support
- Comprehensive error handling
- Full test coverage