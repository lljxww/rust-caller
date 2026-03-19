[English](authentication_EN.md) | 简体中文

# 认证系统

Caller 提供灵活的认证系统，支持静态和动态认证。

## 快速开始

```rust
use caller::{init_config, register_auth, BearerAuth, call};

// 1. 初始化配置
init_config()?;

// 2. 注册认证器
register_auth("my_api", BearerAuth::new("your-token".to_string()))?;

// 3. 调用 API（自动应用认证）
let result = call("MyAPI.protected_method", None).await?;
```

## 内置认证类型

### 1. Bearer Token

```rust
use caller::BearerAuth;

// 直接创建
let auth = BearerAuth::new("your-token".to_string());

// 从环境变量
let auth = BearerAuth::from_env("API_TOKEN")?;
```

### 2. Basic 认证

```rust
use caller::BasicAuth;

let auth = BasicAuth::new("username".to_string(), "password".to_string());

// 从环境变量
let auth = BasicAuth::from_env("API_USER", "API_PASS")?;
```

### 3. API Key

```rust
use caller::ApiKeyAuth;

let auth = ApiKeyAuth::new("X-API-Key".to_string(), "your-key".to_string());

// 从环境变量
let auth = ApiKeyAuth::from_env("X-API-Key", "API_KEY")?;
```

### 4. OAuth2

```rust
use caller::OAuth2Auth;

let auth = OAuth2Auth::new("your-token".to_string());

// 自定义前缀
let auth = OAuth2Auth::with_prefix("token".to_string(), "OAuth".to_string());
```

### 5. 自定义请求头

```rust
use caller::CustomHeaderAuth;

let mut auth = CustomHeaderAuth::new();
auth.add_header("X-API-Key", "key123");
auth.add_header("X-Client-Id", "client456");
```

## 动态认证

### 1. 从回调函数

每次请求时动态获取 token：

```rust
use caller::DynamicBearerAuth;

let auth = DynamicBearerAuth::new(|| {
    // 每次请求时调用
    fetch_token_from_cache_or_oauth()
});
register_auth("dynamic", auth)?;
```

### 2. 从环境变量（动态读取）

```rust
use caller::DynamicBearerAuth;

// 请求时读取环境变量（非启动时）
let auth = DynamicBearerAuth::from_env("API_TOKEN");
register_auth("dynamic_env", auth)?;
```

### 3. 从共享状态（支持 Token 刷新）

```rust
use caller::{DynamicBearerAuth, DynamicApiKeyAuth};
use std::sync::{Arc, RwLock};

// 创建共享 token
let token = Arc::new(RwLock::new("initial-token".to_string()));

// 注册认证器
let auth = DynamicBearerAuth::from_shared(token.clone());
register_auth("refreshable", auth)?;

// 后续刷新 token
*token.write().unwrap() = "new-refreshed-token".to_string();
// 下次请求将使用新 token
```

### 4. 动态 API Key

```rust
use caller::DynamicApiKeyAuth;
use std::sync::{Arc, RwLock};

let api_key = Arc::new(RwLock::new("initial-key".to_string()));
let auth = DynamicApiKeyAuth::from_shared("X-API-Key", api_key.clone());
register_auth("dynamic_key", auth)?;
```

## 闭包认证

最大灵活性，完全自定义：

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

## 运行时更新

### 更新认证器

```rust
use caller::{register_auth, update_auth, BearerAuth};

// 初始注册
register_auth("github", BearerAuth::new("old-token".to_string()))?;

// 运行时更新
update_auth("github", BearerAuth::new("new-token".to_string()))?;
```

### 更新闭包认证

```rust
use caller::update_auth_closure;

update_auth_closure("custom", |builder, ctx| async move {
    Ok(builder.bearer_auth("updated-token"))
})?;
```

## 配置文件关联

在 `caller.json` 中引用认证：

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

然后在代码中注册对应的认证器：

```rust
register_auth("github_auth", BearerAuth::from_env("GITHUB_TOKEN")?)?;
```

## API 参考

| 函数 | 说明 |
|------|------|
| `register_auth(name, auth)` | 注册认证器 |
| `register_auth_closure(name, f)` | 注册闭包认证 |
| `update_auth(name, auth)` | 更新认证器 |
| `update_auth_closure(name, f)` | 更新闭包认证 |
| `has_auth(name)` | 检查是否已注册 |
| `remove_auth(name)` | 移除认证器 |
| `clear_auth()` | 清除所有认证器 |
| `list_auth()` | 列出所有认证器名称 |
| `auth_count()` | 获取认证器数量 |

## 认证优先级

当 Service 和 API Item 都配置了 `AuthorizationType` 时：

**API Item > Service**

```json
{
  "ServiceItems": [
    {
      "ApiName": "MyAPI",
      "AuthorizationType": "default_auth",  // 默认认证
      "ApiItems": [
        {
          "Method": "public",
          "Url": "/public",
          "ParamType": "none"
          // 使用 default_auth
        },
        {
          "Method": "private",
          "Url": "/private",
          "ParamType": "none",
          "AuthorizationType": "special_auth"  // 覆盖为 special_auth
        }
      ]
    }
  ]
}
```

## 最佳实践

### 1. 敏感信息使用环境变量

```rust
// 推荐
let auth = BearerAuth::from_env("API_TOKEN")?;

// 不推荐
let auth = BearerAuth::new("hardcoded-token".to_string());
```

### 2. Token 刷新使用动态认证

```rust
let token = Arc::new(RwLock::new(initial_token()));
let auth = DynamicBearerAuth::from_shared(token.clone());
register_auth("api", auth)?;

// 在后台任务中刷新
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
        let new_token = refresh_token().await;
        *token.write().unwrap() = new_token;
    }
});
```

### 3. 多服务使用不同认证

```rust
register_auth("github", BearerAuth::from_env("GITHUB_TOKEN")?)?;
register_auth("aws", ApiKeyAuth::from_env("X-AWS-Key", "AWS_ACCESS_KEY")?)?;
register_auth("internal", BasicAuth::from_env("INT_USER", "INT_PASS")?)?;
```
