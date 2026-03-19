[English](authentication_EN.md) | [简体中文](authentication_CN.md)

# Authentication System

Caller provides a flexible authentication system supporting both static and dynamic authentication.

## Quick Start

```rust
use caller::{init_config, register_auth, BearerAuth, call};

// 1. Initialize configuration
init_config()?;

// 2. Register authenticator
register_auth("my_api", BearerAuth::new("your-token".to_string()))?;

// 3. Call API (authentication applied automatically)
let result = call("MyAPI.protected_method", None).await?;
```

## Built-in Authentication Types

### 1. Bearer Token

```rust
use caller::BearerAuth;

// Direct creation
let auth = BearerAuth::new("your-token".to_string());

// From environment variable
let auth = BearerAuth::from_env("API_TOKEN")?;
```

### 2. Basic Authentication

```rust
use caller::BasicAuth;

let auth = BasicAuth::new("username".to_string(), "password".to_string());

// From environment variable
let auth = BasicAuth::from_env("API_USER", "API_PASS")?;
```

### 3. API Key

```rust
use caller::ApiKeyAuth;

let auth = ApiKeyAuth::new("X-API-Key".to_string(), "your-key".to_string());

// From environment variable
let auth = ApiKeyAuth::from_env("X-API-Key", "API_KEY")?;
```

### 4. OAuth2

```rust
use caller::OAuth2Auth;

let auth = OAuth2Auth::new("your-token".to_string());

// Custom prefix
let auth = OAuth2Auth::with_prefix("token".to_string(), "OAuth".to_string());
```

### 5. Custom Headers

```rust
use caller::CustomHeaderAuth;

let mut auth = CustomHeaderAuth::new();
auth.add_header("X-API-Key", "key123");
auth.add_header("X-Client-Id", "client456");
```

## Dynamic Authentication

### 1. From Callback Function

Fetch token dynamically on each request:

```rust
use caller::DynamicBearerAuth;

let auth = DynamicBearerAuth::new(|| {
    // Called on each request
    fetch_token_from_cache_or_oauth()
});
register_auth("dynamic", auth)?;
```

### 2. From Environment Variable (Dynamic Read)

```rust
use caller::DynamicBearerAuth;

// Read environment variable at request time (not startup)
let auth = DynamicBearerAuth::from_env("API_TOKEN");
register_auth("dynamic_env", auth)?;
```

### 3. From Shared State (Supports Token Refresh)

```rust
use caller::{DynamicBearerAuth, DynamicApiKeyAuth};
use std::sync::{Arc, RwLock};

// Create shared token
let token = Arc::new(RwLock::new("initial-token".to_string()));

// Register authenticator
let auth = DynamicBearerAuth::from_shared(token.clone());
register_auth("refreshable", auth)?;

// Refresh token later
*token.write().unwrap() = "new-refreshed-token".to_string();
// Next request will use new token
```

### 4. Dynamic API Key

```rust
use caller::DynamicApiKeyAuth;
use std::sync::{Arc, RwLock};

let api_key = Arc::new(RwLock::new("initial-key".to_string()));
let auth = DynamicApiKeyAuth::from_shared("X-API-Key", api_key.clone());
register_auth("dynamic_key", auth)?;
```

## Closure Authentication

Maximum flexibility, fully customizable:

```rust
use caller::register_auth_closure;
use reqwest::RequestBuilder;
use caller::AuthContext;

register_auth_closure("custom", |builder: RequestBuilder, ctx: &AuthContext| async move {
    Ok(builder
        .header("X-Service", &ctx.service_name)
        .header("X-Request-Id", uuid::Uuid::new_v4().to_string())
        .bearer_auth(get_token_for(&ctx.service_name))
})?;
```

## Runtime Updates

### Update Authenticator

```rust
use caller::{register_auth, update_auth, BearerAuth};

// Initial registration
register_auth("github", BearerAuth::new("old-token".to_string()))?;

// Runtime update
update_auth("github", BearerAuth::new("new-token".to_string()))?;
```

### Update Closure Authentication

```rust
use caller::update_auth_closure;

update_auth_closure("custom", |builder, ctx| async move {
    Ok(builder.bearer_auth("updated-token"))
})?;
```

## Configuration File Association

Reference authentication in `caller.json`:

```json
{
  "ServiceItems": [
    {
      "ApiName": "GitHub",
      "BaseUrl": "https://api.github.com",
      "AuthorizationType": "github_auth",
      "ApiItems": [
        {
          "Method": "get_user",
          "Url": "/user",
          "HttpMethod": "GET",
          "ParamType": "none"
        }
      ]
    }
  ]
}
```

Then register the corresponding authenticator in code:

```rust
register_auth("github_auth", BearerAuth::from_env("GITHUB_TOKEN")?)?;
```

## API Reference

| Function | Description |
|----------|-------------|
| `register_auth(name, auth)` | Register authenticator |
| `register_auth_closure(name, f)` | Register closure authentication |
| `update_auth(name, auth)` | Update authenticator |
| `update_auth_closure(name, f)` | Update closure authentication |
| `has_auth(name)` | Check if registered |
| `remove_auth(name)` | Remove authenticator |
| `clear_auth()` | Clear all authenticators |
| `list_auth()` | List all authenticator names |
| `auth_count()` | Get authenticator count |

## Authentication Priority

When both Service and API Item have `AuthorizationType` configured:

**API Item > Service**

```json
{
  "ServiceItems": [
    {
      "ApiName": "MyAPI",
      "AuthorizationType": "default_auth",  // Default authentication
      "ApiItems": [
        {
          "Method": "public",
          "Url": "/public",
          "ParamType": "none"
          // Uses default_auth
        },
        {
          "Method": "private",
          "Url": "/private",
          "ParamType": "none",
          "AuthorizationType": "special_auth"  // Override to special_auth
        }
      ]
    }
  ]
}
```

## Best Practices

### 1. Use Environment Variables for Sensitive Information

```rust
// Recommended
let auth = BearerAuth::from_env("API_TOKEN")?;

// Not recommended
let auth = BearerAuth::new("hardcoded-token".to_string());
```

### 2. Use Dynamic Authentication for Token Refresh

```rust
let token = Arc::new(RwLock::new(initial_token()));
let auth = DynamicBearerAuth::from_shared(token.clone());
register_auth("api", auth)?;

// Refresh in background task
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
        let new_token = refresh_token().await;
        *token.write().unwrap() = new_token;
    }
});
```

### 3. Use Different Authentication for Multiple Services

```rust
register_auth("github", BearerAuth::from_env("GITHUB_TOKEN")?)?;
register_auth("aws", ApiKeyAuth::from_env("X-AWS-Key", "AWS_ACCESS_KEY")?)?;
register_auth("internal", BasicAuth::from_env("INT_USER", "INT_PASS")?)?;