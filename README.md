# Caller For Rust

一个灵活、可配置的 Web API 请求库，基于 Rust 开发，支持通过 JSON 配置文件管理多个 API 端点。

## 特性

- 🚀 **异步调用**：基于 Tokio 异步运行时，支持高并发请求
- ⚡ **配置化管理**：通过 JSON 配置文件定义 API 端点
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
│   └── service_item.rs  # 服务项目定义
├── config/        # 配置管理
│   └── config_loader.rs # 配置加载器
├── infra/         # 基础设施层
│   └── http.rs     # HTTP 客户端封装
└── shared/        # 共享模块
    └── error.rs    # 错误定义
```

### 工作流程
1. **配置加载**：从 JSON 文件加载 API 配置
2. **上下文管理**：管理调用上下文和中间件
3. **HTTP 调用**：构造和发送 HTTP 请求
4. **结果处理**：解析和格式化响应结果

## 快速开始

### 添加依赖

```toml
[dependencies]
caller = "0.1.0"
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

## 配置文件

创建 `caller.json` 配置文件来定义 API 端点：

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

#### `CallerError`
```rust
pub enum CallerError {
    ConfigError(String),
    HttpError(reqwest::Error),
    JsonError(String),
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

### 公共 API

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

### v0.1.0
- 初始版本发布
- 支持基于配置文件的 API 调用
- 支持多种 HTTP 方法和参数类型
- 内置认证和缓存支持
- 完善的错误处理
- 全面的测试覆盖
