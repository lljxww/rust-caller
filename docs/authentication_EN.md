[English](authentication_EN.md) | [简体中文](authentication_CN.md)

# Authentication

Authentication providers are registered by name and referenced by
`authorization_type` in service or endpoint configuration. Endpoint settings
override service settings. A missing provider returns `UnknownAuthProvider`
before network I/O.

Prefer instance-local registration:

```rust,no_run
use caller::{BearerAuth, Caller};

let caller = Caller::from_path("caller.json")?;
caller.register_auth("github", BearerAuth::from_env("GITHUB_TOKEN")?)?;
# Ok::<(), caller::CallerError>(())
```

The crate-root `register_auth` registry exists for the global `call` functions.
It is process-wide; tests and multi-tenant applications should use instance
registries to avoid shared mutable state.

## Built-in providers

```rust,no_run
use caller::{ApiKeyAuth, BasicAuth, BearerAuth, CustomHeaderAuth, OAuth2Auth};

let bearer = BearerAuth::new("token".to_string());
let basic = BasicAuth::new("user".to_string(), "password".to_string());
let key = ApiKeyAuth::new("x-api-key".to_string(), "secret".to_string());
let oauth = OAuth2Auth::new("access-token".to_string());

let mut custom = CustomHeaderAuth::new();
custom.add_header("x-signature", "signature-value");
# Ok::<(), caller::CallerError>(())
```

`BearerAuth` and `OAuth2Auth` both use the standard `Authorization` header by
default. `BasicAuth`, `BearerAuth`, `ApiKeyAuth`, and `OAuth2Auth` have fallible
`from_env` constructors that read once during construction.

## Dynamic credentials

Dynamic providers run for every request attempt, including retries:

```rust,no_run
use caller::{CallerError, DynamicBearerAuth};

let auth = DynamicBearerAuth::try_new(|| {
    obtain_current_token().map_err(|error| {
        CallerError::authentication_error(format!("token refresh failed: {error}"))
    })
});
# fn obtain_current_token() -> Result<String, &'static str> { Ok("token".into()) }
```

Use `DynamicBearerAuth::from_env` to read an environment variable at request
time, or `from_shared(Arc<RwLock<String>>)` for an externally refreshed token.
Missing variables and poisoned locks are explicit errors; they do not generate
an empty credential. Equivalent dynamic API-key and custom-header providers are
available. Use `try_new` when a provider can fail; `new` is for infallible
callbacks.

## Custom authentication

Implement `Authenticator` for reusable behavior, or register an async closure:

```rust,no_run
use caller::Caller;

let caller = Caller::from_path("caller.json")?;
caller.register_auth_closure("signed", |builder, context| {
    let method = context.http_method.clone();
    async move { Ok(builder.header("x-signed-method", method)) }
})?;
# Ok::<(), caller::CallerError>(())
```

`AuthContext` contains the configured service and endpoint names, final URL,
HTTP method, auth provider name, and a compatibility map of scalar parameters.
Repeated query keys cannot be represented in that compatibility map; sign the
final request with a custom `reqwest::Client` layer if canonical repeated-key
signatures are required.

## Security rules

- Keep tokens out of repository config, logs, panic messages, and URLs.
- Built-in provider `Debug` implementations redact secret values, but custom
  middleware and authenticators must implement their own redaction.
- Prefer environment variables or a secret manager and return refresh failures.
- Do not hold a write lock while performing network token refresh.
- Header names and values are validated by reqwest when the request is built.
- `authorizations[].authorization_info` is legacy metadata and is not applied to
  requests. Do not use it as a secret store.
