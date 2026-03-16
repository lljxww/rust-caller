# API 文档服务器

Caller 提供内置的 API 文档服务器，支持 Swagger UI 和请求代理测试。

## 启用功能

在 `Cargo.toml` 中启用 `server` 功能：

```toml
[dependencies]
caller = { version = "0.3.0", features = ["server"] }
```

## 启动服务器

### 方式一：使用示例程序

```bash
cd your-project
cargo run --features server --example server
```

### 方式二：自定义代码

```rust
use caller::{init_config, start_server, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化配置
    init_config()?;

    // 配置服务器
    let config = ServerConfig::new()
        .addr("127.0.0.1:8080")?
        .title("My API")
        .version("1.0.0");

    // 启动服务器
    start_server(config).await?;

    Ok(())
}
```

## 服务器端点

| 端点 | 描述 |
|------|------|
| `GET /` | Swagger UI - 交互式 API 文档 |
| `GET /openapi.json` | OpenAPI 3.0 规范（JSON） |
| `GET /proxy/{service}/{method}` | API 代理端点 |

## Swagger UI

启动服务器后，打开浏览器访问：

```
http://127.0.0.1:8080/
```

Swagger UI 功能：
- 查看所有配置的 API 端点
- 查看请求/响应格式
- **Try it out** - 直接测试 API

### Proxy Mode

Swagger UI 中的 "Try it out" 请求会通过 caller 代理发起，路径格式为：

```
/proxy/{service}/{method}?id=VALUE&param1=VALUE1
```

## 代理端点使用

### GET 请求

```bash
# 列表接口
curl "http://localhost:8080/proxy/JP/list"

# 路径参数
curl "http://localhost:8080/proxy/JP/get?id=1"

# 查询参数
curl "http://localhost:8080/proxy/JP/filter?userId=1&status=active"
```

### POST 请求

```bash
curl -X POST "http://localhost:8080/proxy/JP/create" \
  -H "Content-Type: application/json" \
  -d '{"title":"Test","body":"Content","userId":1}'
```

### 响应格式

成功的 JSON 响应会直接返回：

```json
{
  "id": 1,
  "title": "Test",
  "body": "Content"
}
```

非 JSON 响应会包装：

```json
{
  "response": "<html>...</html>"
}
```

错误响应：

```json
{
  "error": "Service 'UnknownService' not found",
  "available_services": ["JP", "GitHub"]
}
```

## 自定义配置

### 监听地址

```rust
let config = ServerConfig::new()
    .addr("0.0.0.0:3000")?;  // 监听所有网卡
```

### API 标题和版本

```rust
let config = ServerConfig::new()
    .title("My Company API")
    .version("2.0.0")
    .description("Internal API documentation");
```

## OpenAPI 规范

### 获取规范

```bash
# 下载 JSON
curl http://localhost:8080/openapi.json > openapi.json

# 在线查看
# 访问 https://editor.swagger.io/ 粘贴内容
```

### 程序化生成

```rust
use caller::{init_config, OpenApiGenerator};
use std::fs;

init_config()?;

let generator = OpenApiGenerator::from_config_file()?
    .title("My API")
    .version("1.0.0")
    .description("API documentation");

// 保存为 JSON
fs::write("openapi.json", generator.to_json()?)?;

// 保存为 YAML
fs::write("openapi.yaml", generator.to_yaml()?)?;
```

### Proxy Mode

服务器自动启用 proxy mode，生成的 OpenAPI 路径指向代理端点：

```json
{
  "paths": {
    "/proxy/JP/list": {
      "get": {
        "summary": "List all posts",
        "tags": ["JP"],
        ...
      }
    }
  }
}
```

## CORS 支持

服务器默认启用 CORS，允许跨域请求。

## 与其他工具集成

### Postman

1. 导入 OpenAPI 规范：`http://localhost:8080/openapi.json`
2. Postman 自动生成请求集合

### Insomnia

1. 创建新 Collection
2. Import from URL: `http://localhost:8080/openapi.json`

### curl 脚本生成

Swagger UI 可以生成 curl 命令，直接复制使用。
