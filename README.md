# Caller for Rust

English | [简体中文](README_CN.md)

`caller` is an asynchronous, configuration-driven HTTP client for applications
that call a known set of upstream APIs. It provides instance-scoped clients,
typed and separated request parameters, runtime authentication, retry policies,
middleware, bounded response buffering, file downloads, OpenAPI generation,
and an optional local development proxy.

## Requirements

- Rust 1.88 or newer
- Tokio runtime
- TLS uses rustls; native-tls is not enabled

```toml
[dependencies]
caller = "0.4"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Quick start

Create `caller.json`:

```json
{
  "service_items": [{
    "api_name": "users",
    "base_url": "https://api.example.com",
    "timeout": 5000,
    "api_items": [{
      "method": "get",
      "url": "/users/{id}",
      "http_method": "GET",
      "param_type": "path,query"
    }]
  }]
}
```

Prefer the instance API for reusable application components:

```rust,no_run
use caller::{Caller, RequestArgs, params};

#[tokio::main]
async fn main() -> Result<(), caller::CallerError> {
    let caller = Caller::from_path("caller.json")?;
    let args = RequestArgs::new()
        .with_path(params! { "id" => "a/b" })
        .with_query(params! { "expand" => true });

    let response = caller.call_args("users.get", args).await?;
    let response = response.error_for_status()?;
    println!("{} in {:?}", response.status_code, response.duration);
    Ok(())
}
```

Path values are percent-encoded as path segments. `RequestArgs` keeps path,
query, form, and JSON values separate. It also supports repeated query/form keys
through `with_query_pairs` and `with_form_pairs`.

The legacy `call` API accepts one `HashMap<String, String>` and applies it to
every configured parameter location. Keep it for simple or existing code; use
`call_args` for combined parameter kinds.

## Programmatic configuration

```rust
use caller::{ConfigBuilder, HttpMethod, ParamType};

let mut builder = ConfigBuilder::new();
builder
    .service("users", "https://api.example.com")
    .api_typed("list", "/users", HttpMethod::Get, [ParamType::Query])
    .api_endpoint("update", "/users/{id}")
    .http_method(HttpMethod::Patch)
    .param_types([ParamType::Path, ParamType::Json])
    .timeout(5_000)
    .build()
    .build();

let config = builder.build_validated()?;
# Ok::<(), caller::CallerError>(())
```

Supported methods are GET, POST, PUT, DELETE, PATCH, HEAD, and OPTIONS.
Configuration is accepted as JSON, YAML, or TOML and rejects unknown fields.

## Authentication

Credentials are runtime objects, not configuration-file secrets:

```rust,no_run
use caller::{BearerAuth, Caller};

let caller = Caller::from_path("caller.json")?;
caller.register_auth("upstream", BearerAuth::from_env("UPSTREAM_TOKEN")?)?;
# Ok::<(), caller::CallerError>(())
```

Set `authorization_type: "upstream"` on a service or endpoint. Missing providers
fail before the request is sent. Dynamic providers offer fallible `try_new`
constructors and are evaluated for every attempt. Built-in authentication types
redact credentials from `Debug` output.

## Retry, middleware, and limits

```rust,no_run
use caller::{Caller, HeaderMiddleware, RetryConfig};
use std::time::Duration;

# fn build() -> Result<(), caller::CallerError> {
let caller = Caller::builder()
    .config_path("caller.json")
    .middleware(HeaderMiddleware::new().with_header("x-app", "billing")?)
    .max_response_body_bytes(8 * 1024 * 1024)
    .max_download_bytes(128 * 1024 * 1024)
    .build()?;

let retry = RetryConfig::new()
    .with_max_retries(3)
    .with_base_delay(Duration::from_millis(250))
    .with_max_delay(Duration::from_secs(10));
# Ok(())
# }
```

Retries cover configured HTTP status codes and genuine network failures only.
Integer-seconds `Retry-After` is honored by default and capped by `max_delay`.
Exponential delays use 20% positive jitter by default. Middleware runs around
every attempt. Ordinary responses buffer at most 16 MiB
and downloads at most 256 MiB unless the builder overrides those limits.

## Optional development server

```toml
caller = { version = "0.4", features = ["server"] }
```

```bash
cargo run --features server --example server
```

The server binds to loopback, disables permissive CORS, and enables its proxy by
default. It refuses a remote bind unless `allow_remote(true)` is explicit. The
proxy is a development tool and has no user authentication; do not expose it to
an untrusted network.

## Documentation

- [Configuration](docs/configuration_EN.md)
- [JSON, YAML, and TOML](docs/multi-format-config_EN.md)
- [Authentication](docs/authentication_EN.md)
- [Middleware](docs/middleware_EN.md)
- [Development server](docs/server_EN.md)
- [Maturity and remaining limits](docs/CRATE_REFACTOR_CHECKLIST.md)
- [Changelog](CHANGELOG.md)

## Verification

```bash
cargo fmt --all -- --check
cargo check --all-features --all-targets
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features --all-targets
cargo test --doc
cargo doc --all-features --no-deps
cargo package --allow-dirty
```

Tests marked `requires external network access` are ignored by default.

## License

[MIT](LICENSE)
