# 配置文件

Caller 支持多种配置文件格式：JSON、YAML 和 TOML。

## 配置结构

### 完整示例（JSON）

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

## 字段说明

### ServiceItem

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `ApiName` | string | ✅ | 服务名称，用于调用时引用 |
| `BaseUrl` | string | ✅ | API 基础 URL |
| `AuthorizationType` | string | ❌ | 默认认证类型 |
| `Timeout` | number | ❌ | 默认超时（毫秒） |
| `ApiItems` | array | ✅ | API 端点列表 |

### ApiItem

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `Method` | string | ✅ | 方法名，调用时使用 `ServiceName.MethodName` |
| `Url` | string | ✅ | 相对 URL，支持路径参数 `{id}` |
| `HttpMethod` | string | ✅ | HTTP 方法：GET, POST, PUT, DELETE, PATCH |
| `ParamType` | string | ✅ | 参数类型（见下表） |
| `Description` | string | ❌ | 方法描述 |
| `AuthorizationType` | string | ❌ | 覆盖服务级认证 |
| `Timeout` | number | ❌ | 覆盖服务级超时 |
| `ContentType` | string | ❌ | 自定义 Content-Type |

### ParamType 参数类型

| 类型 | 说明 | 示例 |
|------|------|------|
| `none` | 无参数 | `/posts` |
| `query` | URL 查询参数 | `/posts?userId=1` |
| `path` | URL 路径参数 | `/posts/1` |
| `json` | JSON 请求体 | `{"title":"Test"}` |
| `form` | 表单数据 | `title=Test&body=Content` |
| `path,json` | 路径 + JSON | `/posts/1` + `{"title":"Updated"}` |
| `path,query` | 路径 + 查询 | `/posts/1?fields=id,title` |

## 多格式支持

### JSON（默认）

文件：`caller.json`

```json
{
  "ServiceItems": [...]
}
```

### YAML

文件：`caller.yaml` 或 `caller.yml`

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

文件：`caller.toml`

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

## 格式转换

```rust
use caller::config::config_loader::ConfigLoader;

// JSON → YAML
ConfigLoader::convert_config("caller.json", "caller.yaml")?;

// YAML → TOML
ConfigLoader::convert_config("caller.yaml", "caller.toml")?;

// 显式指定格式
use caller::config::config_loader::ConfigFormat;
ConfigLoader::convert_config_with_format(
    "config.txt",
    "output.yaml",
    ConfigFormat::Json,
)?;
```

## 配置加载

### 自动加载

```rust
use caller::init_config;

// 从 ./caller.json 加载（或 .yaml/.toml）
init_config()?;
```

### 手动加载

```rust
use caller::config::config_loader::ConfigLoader;

// 从指定路径加载
ConfigLoader::load_config_from_path("config/api.json")?;

// 显式指定格式
ConfigLoader::load_config_from_path_with_format(
    "config/api.txt",
    ConfigFormat::Json,
)?;
```

### 程序化配置

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

## 配置热更新

```rust
use caller::config::config_loader::ConfigLoader;
use std::time::Duration;

// 启动文件监视
ConfigLoader::start_watching(Duration::from_millis(500))?;

// 配置文件修改后自动重新加载
// 无需重启应用
```

## 环境变量

在 URL 中使用环境变量：

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

## 最佳实践

### 1. 按环境分离

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

### 2. 敏感信息使用认证系统

不要在配置文件中硬编码 token：

```json
// ❌ 不推荐
{
  "Authorizations": [
    { "Token": "hardcoded-secret-token" }
  ]
}

// ✅ 推荐：配置只引用名称
{
  "ServiceItems": [{
    "AuthorizationType": "github_auth"
  }]
}

// 代码中注册
BearerAuth::from_env("GITHUB_TOKEN")?;
register_auth("github_auth", auth)?;
```

### 3. 版本控制

```json
{
  "_meta": {
    "version": "1.0.0",
    "last_updated": "2024-01-15"
  },
  "ServiceItems": [...]
}
```
