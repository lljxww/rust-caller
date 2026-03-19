[English](configuration_EN.md) | [简体中文](configuration_CN.md)

# Configuration Files

Caller supports multiple configuration file formats: JSON, YAML, and TOML.

## Configuration Structure

### Complete Example (JSON)

```json
{
  "Authorizations": [],
  "ServiceItems": [
    {
      "ApiName": "JP",
      "BaseUrl": "https://jsonplaceholder.typicode.com",
      "AuthorizationType": null,
      "Timeout": 30000,
      "ApiItems": [
        {
          "Method": "list",
          "Url": "/posts",
          "HttpMethod": "GET",
          "ParamType": "query",
          "Description": "List all posts"
        },
        {
          "Method": "get",
          "Url": "/posts/{id}",
          "HttpMethod": "GET",
          "ParamType": "path",
          "Description": "Get single post"
        },
        {
          "Method": "create",
          "Url": "/posts",
          "HttpMethod": "POST",
          "ParamType": "json",
          "Description": "Create new post"
        }
      ]
    }
  ]
}
```

## Field Descriptions

### ServiceItem

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `ApiName` | string | ✅ | Service name, used for reference when calling |
| `BaseUrl` | string | ✅ | API base URL |
| `AuthorizationType` | string | ❌ | Default authentication type |
| `Timeout` | number | ❌ | Default timeout (milliseconds) |
| `ApiItems` | array | ✅ | List of API endpoints |

### ApiItem

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `Method` | string | ✅ | Method name, use `ServiceName.MethodName` when calling |
| `Url` | string | ✅ | Relative URL, supports path parameters `{id}` |
| `HttpMethod` | string | ✅ | HTTP method: GET, POST, PUT, DELETE, PATCH |
| `ParamType` | string | ✅ | Parameter type (see table below) |
| `Description` | string | ❌ | Method description |
| `AuthorizationType` | string | ❌ | Override service-level authentication |
| `Timeout` | number | ❌ | Override service-level timeout |
| `ContentType` | string | ❌ | Custom Content-Type |

### ParamType Parameter Types

| Type | Description | Example |
|------|-------------|---------|
| `none` | No parameters | `/posts` |
| `query` | URL query parameters | `/posts?userId=1` |
| `path` | URL path parameters | `/posts/1` |
| `json` | JSON request body | `{"title":"Test"}` |
| `form` | Form data | `title=Test&body=Content` |
| `path,json` | Path + JSON | `/posts/1` + `{"title":"Updated"}` |
| `path,query` | Path + Query | `/posts/1?fields=id,title` |

## Multi-format Support

### JSON (Default)

File: `caller.json`

```json
{
  "ServiceItems": [...]
}
```

### YAML

File: `caller.yaml` or `caller.yml`

```yaml
ServiceItems:
  - ApiName: JP
    BaseUrl: https://jsonplaceholder.typicode.com
    ApiItems:
      - Method: list
        Url: /posts
        HttpMethod: GET
        ParamType: query
```

### TOML

File: `caller.toml`

```toml
[[ServiceItems]]
ApiName = "JP"
BaseUrl = "https://jsonplaceholder.typicode.com"

[[ServiceItems.ApiItems]]
Method = "list"
Url = "/posts"
HttpMethod = "GET"
ParamType = "query"
```

## Format Conversion

```rust
use caller::config::config_loader::ConfigLoader;

// JSON → YAML
ConfigLoader::convert_config("caller.json", "caller.yaml")?;

// YAML → TOML
ConfigLoader::convert_config("caller.yaml", "caller.toml")?;

// Explicitly specify format
use caller::config::config_loader::ConfigFormat;
ConfigLoader::convert_config_with_format(
    "config.txt",
    "output.yaml",
    ConfigFormat::Json,
)?;
```

## Configuration Loading

### Automatic Loading

```rust
use caller::init_config;

// Load from ./caller.json (or .yaml/.toml)
init_config()?;
```

### Manual Loading

```rust
use caller::config::config_loader::ConfigLoader;

// Load from specific path
ConfigLoader::load_config_from_path("config/api.json")?;

// Explicitly specify format
ConfigLoader::load_config_from_path_with_format(
    "config/api.txt",
    ConfigFormat::Json,
)?;
```

### Programmatic Configuration

```rust
use caller::config::config_loader::ConfigLoader;
use caller::domain::{CallerConfig, ServiceItem, ApiItem};

let config = CallerConfig {
    service_items: vec![
        ServiceItem {
            api_name: "MyAPI".to_string(),
            base_url: "https://api.example.com".to_string(),
            authorization_type: None,
            timeout: Some(30000),
            api_items: vec![
                ApiItem {
                    method: "list".to_string(),
                    url: "/items".to_string(),
                    http_method: "GET".to_string(),
                    param_type: "query".to_string(),
                    description: Some("List items".to_string()),
                    // ...
                },
            ],
            use_new_http_client: None,
        },
    ],
    authorizations: vec![],
};

ConfigLoader::init_with_config(config);
```

## Hot Reload

```rust
use caller::config::config_loader::ConfigLoader;
use std::time::Duration;

// Start file watching
ConfigLoader::start_watching(Duration::from_millis(500))?;

// Automatically reloads when config file changes
// No need to restart application
```

## Environment Variables

Use environment variables in URLs:

```json
{
  "ServiceItems": [
    {
      "ApiName": "Internal",
      "BaseUrl": "http://${API_HOST}:${API_PORT}",
      ...
    }
  ]
}
```

## Best Practices

### 1. Separate by Environment

```
config/
├── caller.dev.json
├── caller.staging.json
└── caller.prod.json
```

```rust
let env = std::env::var("ENV").unwrap_or("dev".to_string());
let config_path = format!("config/caller.{}.json", env);
ConfigLoader::load_config_from_path(&config_path)?;
```

### 2. Use Authentication System for Sensitive Information

Don't hardcode tokens in configuration files:

```json
// ❌ Not recommended
{
  "Authorizations": [
    { "Token": "hardcoded-secret-token" }
  ]
}

// ✅ Recommended: config only references name
{
  "ServiceItems": [{
    "AuthorizationType": "github_auth"
  }]
}

// Register in code
BearerAuth::from_env("GITHUB_TOKEN")?;
register_auth("github_auth", auth)?;
```

### 3. Version Control

```json
{
  "_meta": {
    "version": "1.0.0",
    "last_updated": "2024-01-15"
  },
  "ServiceItems": [...]
}