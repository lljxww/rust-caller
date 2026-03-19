[English](server_EN.md) | [简体中文](server_CN.md)

# API Documentation Server

Caller provides a built-in API documentation server with Swagger UI and request proxy testing support.

## Enable Feature

Enable the `server` feature in `Cargo.toml`:

```toml
[dependencies]
caller = { version = "0.3.0", features = ["server"] }
```

## Starting the Server

### Method 1: Using Example Program

```bash
cd your-project
cargo run --features server --example server
```

### Method 2: Custom Code

```rust
use caller::{init_config, start_server, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize configuration
    init_config()?;

    // Configure server
    let config = ServerConfig::new()
        .addr("127.0.0.1:8080")?
        .title("My API")
        .version("1.0.0");

    // Start server
    start_server(config).await?;

    Ok(())
}
```

## Server Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /` | Swagger UI - Interactive API documentation |
| `GET /openapi.json` | OpenAPI 3.0 specification (JSON) |
| `GET /proxy/{service}/{method}` | API proxy endpoint |

## Swagger UI

After starting the server, open your browser and visit:

```
http://127.0.0.1:8080/
```

Swagger UI features:
- View all configured API endpoints
- View request/response formats
- **Try it out** - Test APIs directly

### Proxy Mode

"Try it out" requests in Swagger UI are routed through the caller proxy with the path format:

```
/proxy/{service}/{method}?id=VALUE&param1=VALUE1
```

## Proxy Endpoint Usage

### GET Requests

```bash
# List endpoint
curl "http://localhost:8080/proxy/JP/list"

# Path parameters
curl "http://localhost:8080/proxy/JP/get?id=1"

# Query parameters
curl "http://localhost:8080/proxy/JP/filter?userId=1&status=active"
```

### POST Requests

```bash
curl -X POST "http://localhost:8080/proxy/JP/create" \
  -H "Content-Type: application/json" \
  -d '{"title":"Test","body":"Content","userId":1}'
```

### Response Format

Successful JSON responses are returned directly:

```json
{
  "id": 1,
  "title": "Test",
  "body": "Content"
}
```

Non-JSON responses are wrapped:

```json
{
  "response": "<html>...</html>"
}
```

Error responses:

```json
{
  "error": "Service 'UnknownService' not found",
  "available_services": ["JP", "GitHub"]
}
```

## Custom Configuration

### Listen Address

```rust
let config = ServerConfig::new()
    .addr("0.0.0.0:3000")?;  // Listen on all interfaces
```

### API Title and Version

```rust
let config = ServerConfig::new()
    .title("My Company API")
    .version("2.0.0")
    .description("Internal API documentation");
```

## OpenAPI Specification

### Get Specification

```bash
# Download JSON
curl http://localhost:8080/openapi.json > openapi.json

# View online
# Visit https://editor.swagger.io/ and paste content
```

### Programmatic Generation

```rust
use caller::{init_config, OpenApiGenerator};
use std::fs;

init_config()?;

let generator = OpenApiGenerator::from_config_file()?
    .title("My API")
    .version("1.0.0")
    .description("API documentation");

// Save as JSON
fs::write("openapi.json", generator.to_json()?)?;

// Save as YAML
fs::write("openapi.yaml", generator.to_yaml()?)?;
```

### Proxy Mode

The server automatically enables proxy mode, generating OpenAPI paths that point to proxy endpoints:

```json
{
  "paths": {
    "/proxy/JP/list": {
      "get": {
        "summary": "List all posts",
        "tags": ["JP"],
        ...
      }
    }
  }
}
```

## CORS Support

The server has CORS enabled by default, allowing cross-origin requests.

## Integration with Other Tools

### Postman

1. Import OpenAPI specification: `http://localhost:8080/openapi.json`
2. Postman automatically generates request collections

### Insomnia

1. Create new Collection
2. Import from URL: `http://localhost:8080/openapi.json`

### curl Script Generation

Swagger UI can generate curl commands that you can copy and use directly.