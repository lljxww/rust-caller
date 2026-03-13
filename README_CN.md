# Caller For Rust

[English](https://github.com/lljxww/rust-caller/blob/main/README.md) | 简体中文

一个灵活、可配置的 Web API 请求库，基于 Rust 开发，支持通过配置文件管理多个 API 端点。

## 特性

- 🚀 **异步调用**：基于 Tokio 异步运行时，支持高并发请求
- ⚡ **配置化管理**：通过配置文件定义 API 端点（支持 JSON/YAML/TOML）
- 🔄 **热重载**：支持配置文件热重载，无需重启即可更新配置
- 📥 **文件下载**：内置文件下载功能，支持自动格式检测
- 🔁 **重试机制**：支持请求失败时的自动重试和指数退避
- 🔐 **认证支持**：内置多种认证机制（header、query、basic auth）
- 📊 **结果解析**：强大的 JSON 结果解析，支持深度路径访问
- 🛡️ **错误处理**：完善的错误类型和错误处理机制
- 🧪 **测试覆盖**：内置全面的单元测试和集成测试
- 🏗️ **模块化架构**：清晰的分层架构，易于扩展和维护

## 架构设计

### 项目结构
```
src/
├── core/          # 核心业务逻辑
│   ├── context.rs # 调用上下文管理
│   └── constants.rs # 常量定义
├── domain/        # 领域模型
│   ├── api_item.rs      # API 项目定义
│   ├── api_result.rs    # API 响应结果
│   ├── authorization.rs # 认证相关
│   ├── caller_config.rs # 调用配置
│   ├── download_result.rs # 下载结果处理
│   ├── retry_config.rs  # 重试配置
│   └── service_item.rs  # 服务项目定义
├── config/        # 配置管理
│   └── config_loader.rs # 配置加载器
├── infra/         # 基础设施层
│   └── http.rs     # HTTP 客户端封装
└── shared/        # 共享模块
    └── error.rs    # 错误定义
```

### 工作流程
1. **配置加载**：从文件加载 API 配置（支持 JSON/YAML/TOML）
2. **上下文管理**：管理调用上下文和中间件
3. **HTTP 调用**：构造和发送 HTTP 请求（支持可选的重试）
4. **结果处理**：解析和格式化响应结果

## 快速开始

### 添加依赖

```toml
[dependencies]
caller = "0.2.0"
tokio = { version = "1.0", features = ["full"] }
```

### 基本使用

```rust
use caller::call;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 简单调用
    let result = call("JP.list", None).await?;

    // 解析结果
    println!("Status: {}", result.status_code);
    println!("Raw response: {}", result.raw);

    // 获取特定字段
    if let Some(first_title) = result.get_as_str("0.title") {
        println!("First post title: {}", first_title);
    }

    // 带参数调用
    let params = HashMap::from([
        ("post_id".to_string(), "1".to_string()),
        ("userId".to_string(), "1".to_string()),
    ]);
    let result = call("JP.get", Some(params)).await?;

    Ok(())
}
```

### 带重试的调用

```rust
use caller::{call_with_retry, RetryConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建重试配置
    let retry_config = RetryConfig::new()
        .with_max_retries(3)
        .with_base_delay(Duration::from_millis(500))
        .with_max_delay(Duration::from_secs(30));

    // 带自动重试的调用
    let result = call_with_retry("JP.list", None, retry_config).await?;
    
    Ok(())
}
```

### 文件下载

```rust
use caller::download;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 下载文件，自动检测格式
    let result = download("api.download", None, None).await?;
    println!("下载了 {} 字节", result.size_human());
    
    // 保存到文件
    result.save("./downloads", "myfile")?;
    
    // 指定文件扩展名下载
    let result = download("api.pdf", None, Some("pdf".to_string())).await?;
    result.save("./downloads", "document")?;
    
    Ok(())
}
```

## 配置文件

创建配置文件来定义 API 端点。支持 JSON、YAML 和 TOML 格式。

### JSON 示例 (caller.json)
```json
{
  "Authorizations": [
    {
      "Name": "BearerToken",
      "HeaderName": "Authorization",
      "Type": "Bearer",
      "Token": "your-token-here"
    }
  ],
  "ServiceItems": [
    {
      "ApiName": "weibo",
      "BaseUrl": "https://weibo.com/ajax",
      "ApiItems": [
        {
          "Method": "hot",
          "Url": "/side/hotSearch",
          "HttpMethod": "GET",
          "ParamType": "query",
          "Description": "获取微博热搜",
          "Timeout": 5000,
          "NeedCache": true,
          "CacheTime": 300
        }
      ]
    },
    {
      "ApiName": "JP",
      "BaseUrl": "https://jsonplaceholder.typicode.com",
      "ApiItems": [
        {
          "Method": "get",
          "Url": "/posts/{post_id}",
          "HttpMethod": "GET",
          "ParamType": "path"
        },
        {
          "Method": "create",
          "Url": "/posts",
          "HttpMethod": "POST",
          "ParamType": "json",
          "ContentType": "application/json",
          "AuthorizationType": "BearerToken"
        }
      ]
    }
  ]
}
```

### 参数类型说明

| 参数类型 | 描述 | 示例 |
|---------|------|------|
| `none` | 无参数 | `/posts` |
| `query` | 查询参数 | `/posts?userId=1&id=1` |
| `path` | 路径参数 | `/posts/1` |
| `json` | JSON 请求体 | `{"title": "Hello", "body": "World"}` |
| `path,json` | 路径参数 + JSON 体 | `/posts/1` + `{"title": "Updated"}` |

## API 参考

### 主要类型

#### `ApiResult`
```rust
pub struct ApiResult {
    pub status_code: StatusCode,
    pub raw: String,
    pub j_obj: Value,
}
```

#### `DownloadResult`
```rust
pub struct DownloadResult {
    pub status_code: StatusCode,
    pub content: Vec<u8>,
    pub content_type: String,
    pub file_extension: String,
    pub suggested_filename: Option<String>,
}
```

#### `RetryConfig`
```rust
pub struct RetryConfig {
    pub max_retries: u32,              // 最大重试次数
    pub base_delay: Duration,           // 基础延迟（指数退避基数）
    pub max_delay: Duration,            // 最大延迟
    pub retry_status_codes: Vec<u16>,    // 触发重试的 HTTP 状态码
    pub retry_on_network_error: bool,    // 是否在网络错误时重试
}
```

#### `CallerError`
```rust
pub enum CallerError {
    ConfigError(String),
    HttpError(reqwest::Error),
    JsonError(String),
    IoError(String),
    ServiceNotFound(String),
    MethodNotFound(String),
    ParamMissing(String),
    AuthenticationError(String),
}
```

### 核心方法

#### `api_result.get_as_str(key: &str) -> Option<&str>`
获取字符串类型的值，支持深度路径如 `"0.title"`

#### `api_result.get_as_i64(key: &str) -> Option<i64>`
获取整数值

#### `api_result.get_as_bool(key: &str) -> Option<bool>`
获取布尔值

#### `api_result.get(key: &str) -> Option<&Value>`
获取原生 JSON 值

#### `download_result.save<P: AsRef<Path>>(directory: P, base_name: &str) -> Result<String, CallerError>`
将下载内容保存到文件，返回文件名

#### `download_result.size_human() -> String`
获取人类可读的文件大小（如 "1.23 MB"）

### 公共 API 函数

#### `call(method: &str, params: Option<HashMap<String, String>>) -> Result<ApiResult, CallerError>`
```rust
use caller::call;
use std::collections::HashMap;

// 无参数调用
let result = call("JP.list", None).await?;

// 带参数调用
let params = HashMap::from([
    ("post_id".to_string(), "1".to_string()),
]);
let result = call("JP.get", Some(params)).await?;
```

#### `call_with_retry(method: &str, params: Option<HashMap<String, String>>, retry_config: RetryConfig) -> Result<ApiResult, CallerError>`
```rust
use caller::{call_with_retry, RetryConfig};
use std::time::Duration;

let retry_config = RetryConfig::new()
    .with_max_retries(3)
    .with_base_delay(Duration::from_millis(500));

let result = call_with_retry("JP.list", None, retry_config).await?;
```

#### `download(method: &str, params: Option<HashMap<String, String>>, extension: Option<String>) -> Result<DownloadResult, CallerError>`
```rust
use caller::download;

// 自动检测格式下载
let result = download("api.download", None, None).await?;
result.save("./downloads", "file")?;

// 指定扩展名下载
let result = download("api.pdf", None, Some("pdf".to_string())).await?;
```

### 配置管理

#### `init_config() -> Result<(), CallerError>`
从文件初始化配置

#### `reload_config() -> Result<(), CallerError>`
手动重新加载配置

#### `watch_config() -> Result<(), CallerError>`
开始监听配置文件变更（500ms 防抖）

#### `watch_config_with_debounce(debounce: Duration) -> Result<(), CallerError>`
使用自定义防抖时间开始监听

#### `stop_watch_config()`
停止监听配置文件

#### `is_watching_config() -> bool`
检查配置文件是否正在被监听

#### `is_config_loaded() -> bool`
检查配置是否已加载

```rust
use caller::{init_config, watch_config, stop_watch_config, is_watching_config};

// 初始化配置
init_config()?;

// 开始监听文件变更
watch_config()?;

// 检查状态
if is_watching_config() {
    println!("配置文件正在被监听");
}

// 停止监听
stop_watch_config();
```

## 多格式配置文件支持

Caller 支持多种配置文件格式：

### 支持的格式
- **JSON** (`.json`)
- **YAML** (`.yaml`, `.yml`)
- **TOML** (`.toml`)

### 格式转换

```rust
use caller::config::config_loader::{ConfigLoader, ConfigFormat};

// 在不同格式之间转换
ConfigLoader::convert_config("config.json", "config.yaml")?;
ConfigLoader::convert_config("config.yaml", "config.toml")?;

// 显式指定格式
ConfigLoader::convert_config_with_format(
    "config.json",
    "output.txt",
    ConfigFormat::Yaml
)?;
```

详细文档请参阅 [docs/MULTI_FORMAT_CONFIG.md](docs/MULTI_FORMAT_CONFIG.md)。

## 测试

运行所有测试：

```bash
cargo test
```

运行特定测试：

```bash
cargo test test_call_list_posts
```

## 构建和发布

```bash
# 构建
cargo build

# 运行示例
cargo run --example basic_usage

# 格式化代码
cargo fmt

# 检查代码
cargo check
```

## 贡献指南

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情

## 更新日志

### v0.2.0
- **多格式配置文件支持**：添加对 JSON、YAML 和 TOML 配置文件格式的支持
- **配置文件格式转换**：新增配置文件格式转换功能（支持在 JSON、YAML 和 TOML 之间转换）
- **文件下载**：新增文件下载功能，支持自动格式检测
- **重试机制**：新增请求失败时的自动重试和指数退避功能
- **重试配置**：新增可配置的重试行为 `RetryConfig`
- **下载结果**：新增完整的下载结果处理 `DownloadResult`
- **序列化支持**：为所有配置结构体添加 Serialize trait
- **完整示例**：创建三种格式的完整配置文件示例
- **全面测试**：添加 16 个多格式配置测试用例
- **文档更新**：更新文档以说明新功能和多格式支持

### v0.1.0
- 初始版本发布
- 支持基于配置文件的 API 调用
- 支持多种 HTTP 方法和参数类型
- 内置认证和缓存支持
- 完善的错误处理
- 全面的测试覆盖