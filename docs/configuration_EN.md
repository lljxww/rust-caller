[English](configuration_EN.md) | [简体中文](configuration_CN.md)

# Configuration

Configuration describes routing and transport behavior. Authentication secrets
are registered in Rust at runtime and should not be stored in these files.

## Complete JSON shape

```json
{
  "service_items": [
    {
      "api_name": "catalog",
      "authorization_type": "catalog_token",
      "base_url": "https://api.example.com/v1",
      "timeout": 10000,
      "api_items": [
        {
          "method": "get_product",
          "url": "/products/{id}",
          "http_method": "GET",
          "param_type": "path,query",
          "description": "Get one product",
          "timeout": 3000
        },
        {
          "method": "update_product",
          "url": "/products/{id}",
          "http_method": "PATCH",
          "param_type": "path,json",
          "content_type": "application/json",
          "authorization_type": "admin_token"
        }
      ]
    }
  ]
}
```

`authorizations` is optional and defaults to an empty list. It remains only as
legacy input metadata; it does not create runtime authenticators and is omitted
when serializing or converting configuration. Do not put secrets in
`authorization_info`. Register a provider whose name matches
`authorization_type` instead.

## Service fields

| Field | Required | Meaning |
|---|---:|---|
| `api_name` | yes | Unique service name used by `service.method`; non-empty, no `.` or surrounding whitespace |
| `base_url` | yes | Absolute HTTP(S) URL without userinfo, query, or fragment |
| `authorization_type` | no | Runtime auth provider name inherited by endpoints |
| `timeout` | no | Request timeout in milliseconds; must be greater than zero |
| `api_items` | yes | Endpoint list; an empty list is valid |
| `use_new_http_client` | no | Legacy compatibility field; currently has no effect |

## Endpoint fields

| Field | Required | Meaning |
|---|---:|---|
| `method` | yes | Unique endpoint name inside the service; non-empty, no `.` or surrounding whitespace |
| `url` | yes | Empty for the service root, otherwise starts with `/` |
| `http_method` | yes | `GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `HEAD`, or `OPTIONS` |
| `param_type` | yes | One or more comma-separated parameter locations |
| `description` | no | Human-readable description used by OpenAPI |
| `content_type` | no | Explicit request Content-Type |
| `authorization_type` | no | Endpoint auth provider override |
| `timeout` | no | Endpoint timeout override in milliseconds |
| `need_cache`, `cache_time`, `use_new_http_client` | no | Legacy compatibility fields; currently have no effect |

Valid parameter locations are `none`, `path`, `query`, `json`, and `form`.
`none` cannot be combined with another value; duplicates and `json,form` are
rejected. A URL containing `{name}` must declare `path`, and an endpoint that
declares `path` must contain at least one placeholder.

Timeout resolution is endpoint, then service, then the `CallerBuilder` default
(30 seconds). JSON requests default to `application/json`; other request kinds
do not receive an implicit Content-Type.

## Loading and validation

```rust,no_run
use caller::{Caller, ConfigLoader};

let caller = Caller::from_path("config/caller.yaml")?;
let config = ConfigLoader::load_config_from_path("config/caller.toml")?;
# Ok::<(), caller::CallerError>(())
```

The extension selects JSON, YAML, or TOML. Parsing immediately validates names,
URL rules, duplicate services/endpoints, parameter combinations, timeouts,
header values, and unknown fields. Runtime auth registration is validated when
the request is prepared.

Programmatic configuration should finish with `build_validated()`:

```rust
use caller::{ConfigBuilder, HttpMethod, ParamType};

let mut builder = ConfigBuilder::new();
builder
    .service("health", "https://api.example.com")
    .api_typed("check", "/health", HttpMethod::Get, [ParamType::None])
    .build();
let config = builder.build_validated()?;
# Ok::<(), caller::CallerError>(())
```

The string-based `ServiceBuilder::api(...)` is fallible. Prefer `api_typed` or
`api_endpoint` when constructing new code.

## Global configuration and watching

The crate-root `init_config`, `reload_config`, and watch helpers use exactly
`./caller.json`. For arbitrary files, use an instance `Caller`.

```rust,no_run
use caller::{init_config, last_config_watch_error, watch_config};

init_config()?;
watch_config()?;
if let Some(error) = last_config_watch_error() {
    eprintln!("last asynchronous reload failed: {error}");
}
# Ok::<(), caller::CallerError>(())
```

Watch reload is debounced. A bad update does not replace the last valid config;
inspect `last_config_watch_error()` because filesystem callbacks cannot return
asynchronous failures to the original caller. `ConfigLoader::init_with_config`
also validates before changing global state.

There is no `${ENV_VAR}` interpolation in URLs or other config fields. Resolve
environment-specific values before building the config, or select different
files per environment. Never embed credentials in a URL.
