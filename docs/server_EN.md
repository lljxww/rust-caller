[English](server_EN.md) | [简体中文](server_CN.md)

# Local development server

The optional `server` feature exposes generated OpenAPI, Swagger UI, and an HTTP
proxy backed by the same `Caller` execution path.

```toml
caller = { version = "0.4", features = ["server"] }
```

```rust,no_run
use caller::{Caller, ServerConfig, start_server_with_caller};

#[tokio::main]
async fn main() -> Result<(), caller::CallerError> {
    let caller = Caller::from_path("caller.json")?;
    let server = ServerConfig::new()
        .addr("127.0.0.1:8080")?
        .title("Upstream APIs")
        .version("1.0.0");
    start_server_with_caller(server, caller).await
}
```

Routes:

| Route | Purpose |
|---|---|
| `GET /` | Swagger UI |
| `GET /openapi.json` | OpenAPI 3.0 document |
| `METHOD /proxy/{service}/{method}` | Proxy using the configured upstream HTTP method |
| `METHOD /proxy?service=...&method=...` | Query-style proxy entry |

The proxy validates the incoming method, separates configured path/query/form/
JSON locations, preserves typed JSON bodies, applies runtime auth, middleware,
timeouts, response limits, and client reuse through `Caller`.

Path parameters are supplied as query fields named after each placeholder. The
legacy `id` query field maps to the first path placeholder. Form endpoints
require `application/x-www-form-urlencoded`. JSON bodies may be any JSON value.

## Security defaults

- Default bind is `127.0.0.1:8080`.
- Non-loopback binds are rejected unless `allow_remote(true)` is explicit.
- Wildcard CORS is disabled unless `allow_any_origin(true)` is explicit.
- Proxy routes can be removed with `enable_proxy(false)`; OpenAPI and Swagger UI
  remain available.
- The server has no inbound authentication, authorization, rate limiting, TLS,
  or audit persistence.

`allow_remote(true)` can expose every registered upstream credential through a
generic proxy. Do not use it on an untrusted network without placing a properly
authenticated, authorized, rate-limited, TLS-terminating gateway in front. The
built-in server is intentionally a development tool, not a production gateway.

Swagger UI assets are loaded from `unpkg.com`; an offline or CSP-restricted
environment must provide its own UI frontend and consume `/openapi.json`.

## OpenAPI authentication schemes

Runtime authenticators are arbitrary Rust code, so their OpenAPI scheme cannot
be inferred safely. The generator uses a documented bearer placeholder unless
you override it:

```rust,no_run
use caller::{OpenApiGenerator, SecurityScheme};

# fn build(caller: &caller::Caller) -> Result<(), caller::CallerError> {
let document = OpenApiGenerator::from_caller(caller)?
    .security_scheme("api_key", SecurityScheme::api_key("x-api-key", "header"))
    .generate();
# Ok(())
# }
```

The generated query schema is necessarily generic because current endpoint
configuration records parameter locations but not individual names or schemas.
