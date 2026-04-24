[English](configuration_EN.md) | 简体中文

# 配置文件

Caller 支持多种配置文件格式：JSON、YAML 和 TOML。

本文覆盖两类配置方式：

- 配置文件：适合部署、热更新和跨语言共享
- 程序化配置：适合测试、样例和运行时动态组装

## 配置结构

### 完整示例（JSON）

```json
{
  "authorizations": [],
  "service_items": [
    {
      "api_name": "JP",
      "base_url": "https://jsonplaceholder.typicode.com",
      "authorization_type": null,
      "timeout": 30000,
      "api_items": [
        {
          "method": "list",
          "url": "/posts",
          "http_method": "GET",
          "param_type": "query",
          "description": "List all posts"
        },
        {
          "method": "get",
          "url": "/posts/{id}",
          "http_method": "GET",
          "param_type": "path",
          "description": "Get single post"
        },
        {
          "method": "create",
          "url": "/posts",
          "http_method": "POST",
          "param_type": "json",
          "description": "Create new post"
        }
      ]
    }
  ]
}
```

## 字段说明

### ServiceConfig

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `api_name` | string | ✅ | 服务名称，用于调用时引用 |
| `base_url` | string | ✅ | API 基础 URL |
| `authorization_type` | string | ❌ | 默认认证类型 |
| `timeout` | number | ❌ | 默认超时（毫秒） |
| `api_items` | array | ✅ | API 端点列表 |

### ApiConfig

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `method` | string | ✅ | 方法名，调用时使用 `Servicename.methodname` |
| `url` | string | ✅ | 相对 URL，支持路径参数 `{id}` |
| `http_method` | string | ✅ | HTTP 方法：GET, POST, PUT, DELETE, PATCH |
| `param_type` | string | ✅ | 参数类型（见下表） |
| `description` | string | ❌ | 方法描述 |
| `authorization_type` | string | ❌ | 覆盖服务级认证 |
| `timeout` | number | ❌ | 覆盖服务级超时 |
| `content_type` | string | ❌ | 自定义 Content-Type |

### param_type 参数类型

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
  "service_items": [...]
}
```

### YAML

文件：`caller.yaml` 或 `caller.yml`

```yaml
service_items:
  - api_name: JP
    base_url: https://jsonplaceholder.typicode.com
    api_items:
      - method: list
        url: /posts
        http_method: GET
        param_type: query
```

### TOML

文件：`caller.toml`

```toml
[[service_items]]
api_name = "JP"
base_url = "https://jsonplaceholder.typicode.com"

[[service_items.api_items]]
method = "list"
url = "/posts"
http_method = "GET"
param_type = "query"
```

## 格式转换

```rust
use caller::ConfigLoader;

// JSON → YAML
ConfigLoader::convert_config("caller.json", "caller.yaml")?;

// YAML → TOML
ConfigLoader::convert_config("caller.yaml", "caller.toml")?;

// 显式指定格式
use caller::ConfigFileFormat;
ConfigLoader::convert_config_with_format(
    "config.txt",
    "output.yaml",
    ConfigFileFormat::Json,
)?;
```

## 配置加载

### 自动加载

```rust
use caller::init_config;

// 当前默认只从 ./caller.json 加载
init_config()?;
```

说明：

- `init_config()` / `reload_config()` / `watch_config()` 当前都绑定默认全局路径 `./caller.json`
- 如果你要加载 `caller.yaml` 或 `caller.toml`，请改用 `ConfigLoader::load_config_from_path(...)`
- 如果你希望每个实例各自持有自己的配置，优先使用 `Caller::from_path(...)`

### 手动加载

```rust
use caller::ConfigLoader;

// 从指定路径加载
ConfigLoader::load_config_from_path("config/api.json")?;

// 显式指定格式
use caller::ConfigFileFormat;
ConfigLoader::load_config_from_path_with_format(
    "config/api.txt",
    ConfigFileFormat::Json,
)?;
```

### 程序化配置

```rust
use caller::{ConfigBuilder, ConfigLoader, HttpMethod, ParamType};

let mut builder = ConfigBuilder::new();
builder
    .service("MyAPI", "https://api.example.com")
    .timeout(30_000)
    .api_typed("list", "/items", HttpMethod::Get, [ParamType::Query])
    .api_endpoint("create", "/items/{id}")
    .http_method(HttpMethod::Post)
    .param_types([ParamType::Path, ParamType::Json])
    .description("Create an item")
    .content_type("application/json")
    .timeout(5_000)
    .build()
    .build();

let config = builder.build();

ConfigLoader::init_with_config(config);
```

## 配置热更新

```rust
use caller::ConfigLoader;
use std::time::Duration;

// 启动文件监视
ConfigLoader::start_watching(Duration::from_millis(500))?;

// 配置文件修改后自动重新加载
// 无需重启应用
```

限制说明：

- 当前全局 watch 机制同样只围绕默认全局路径 `./caller.json`
- 如果你使用实例化 `Caller`，可以调用 `Caller::reload_config()` 手动刷新实例配置

## 环境变量

在 URL 中使用环境变量：

```json
{
  "service_items": [
    {
      "api_name": "Internal",
      "base_url": "http://${API_HOST}:${API_PORT}",
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

如果你想保持实例级隔离：

```rust
use caller::Caller;

let env = std::env::var("ENV").unwrap_or("dev".to_string());
let config_path = format!("config/caller.{}.yaml", env);
let caller = Caller::from_path(&config_path)?;
```

### 2. 敏感信息使用认证系统

不要在配置文件中硬编码 token：

```json
// ❌ 不推荐
{
  "authorizations": [
    { "token": "hardcoded-secret-token" }
  ]
}

// ✅ 推荐：配置只引用名称
{
  "service_items": [{
    "authorization_type": "github_auth"
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
  "service_items": [...]
}
```
