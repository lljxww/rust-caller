# 多格式配置文件支持

`rust-caller` 现在支持多种配置文件格式，包括 JSON、YAML 和 TOML。

## 支持的格式

- **JSON** (`.json`)
- **YAML** (`.yaml`, `.yml`)
- **TOML** (`.toml`)

## 使用方法

### 自动检测格式

配置加载器会根据文件扩展名自动检测格式：

```rust
use caller::config::config_loader::ConfigLoader;

// 根据扩展名自动检测
let config = ConfigLoader::load_config_from_path("config.json")?;
let config = ConfigLoader::load_config_from_path("config.yaml")?;
let config = ConfigLoader::load_config_from_path("config.toml")?;
```

### 显式指定格式

你也可以显式指定配置文件格式：

```rust
use caller::config::config_loader::{ConfigFormat, ConfigLoader};

// 显式指定格式
let config = ConfigLoader::load_config_from_path_with_format(
    "myconfig",
    ConfigFormat::Yaml
)?;
```

## 配置文件示例

### JSON 配置示例

```json
{
  "Authorizations": [
    {
      "Name": "BearerAuth",
      "AuthorizationInfo": "Bearer your-token"
    }
  ],
  "ServiceItems": [
    {
      "ApiName": "GitHub_API",
      "BaseUrl": "https://api.github.com",
      "Timeout": 10000,
      "ApiItems": [
        {
          "Method": "get_user",
          "Url": "/users/{username}",
          "HttpMethod": "GET",
          "ParamType": "path"
        }
      ]
    }
  ]
}
```

### YAML 配置示例

```yaml
Authorizations:
  - Name: BearerAuth
    AuthorizationInfo: Bearer your-token

ServiceItems:
  - ApiName: GitHub_API
    BaseUrl: https://api.github.com
    Timeout: 10000
    ApiItems:
      - Method: get_user
        Url: /users/{username}
        HttpMethod: GET
        ParamType: path
```

### TOML 配置示例

```toml
[[Authorizations]]
Name = "BearerAuth"
AuthorizationInfo = "Bearer your-token"

[[ServiceItems]]
ApiName = "GitHub_API"
BaseUrl = "https://api.github.com"
Timeout = 10000

[[ServiceItems.ApiItems]]
Method = "get_user"
Url = "/users/{username}"
HttpMethod = "GET"
ParamType = "path"
```

## 注意事项

1. **字段名格式**：所有配置文件中的字段名必须使用 PascalCase 格式（例如：`Authorizations`、`ServiceItems`、`ApiName`），以匹配 Rust 结构体的 `serde` 重命名配置。

2. **配置文件位置**：默认配置文件路径是 `./caller.json`，但你可以通过 `load_config_from_path` 方法加载任意位置的配置文件。

3. **格式选择建议**：
   - JSON：适用于需要与其他工具集成的场景
   - YAML：更易读，适合手动编辑
   - TOML：Rust 生态推荐格式，语法简洁

4. **错误处理**：如果配置文件格式不正确或包含不支持的扩展名，会返回 `CallerError::ConfigError`。

## 配置文件格式转换

`rust-caller` 提供了内置的配置文件格式转换功能，可以在不同格式之间轻松转换。

### 自动检测转换

根据输出文件的扩展名自动检测目标格式：

```rust
use caller::config::config_loader::ConfigLoader;

// JSON 转换为 YAML
ConfigLoader::convert_config("config.json", "config.yaml")?;

// YAML 转换为 TOML
ConfigLoader::convert_config("config.yaml", "config.toml")?;

// TOML 转换为 JSON
ConfigLoader::convert_config("config.toml", "config.json")?;
```

### 显式指定格式

也可以显式指定输出格式，忽略文件扩展名：

```rust
use caller::config::config_loader::{ConfigLoader, ConfigFormat};

// 强制输出为 YAML 格式，无论文件扩展名是什么
ConfigLoader::convert_config_with_format(
    "config.json",
    "my_config.txt",
    ConfigFormat::Yaml
)?;
```

### 使用场景

配置文件转换功能在以下场景中特别有用：

1. **团队协作**：不同团队成员可能偏好不同的配置格式，使用转换功能可以轻松适配
2. **系统集成**：将配置转换为与其他工具兼容的格式
3. **配置迁移**：从一个迁移工具切换到另一个时，批量转换配置文件
4. **格式统一**：将现有的各种格式配置统一为一种标准格式

### 数据完整性保证

转换过程会：
- 保留所有配置字段和数据
- 保持字段名格式（PascalCase）
- 验证输出文件的有效性
- 支持格式化输出（JSON 和 TOML 使用美化格式）

## 完整示例

参见 `samples/` 目录中的完整配置文件示例：
- `samples/api_config_example.json`
- `samples/api_config_example.yaml`
- `samples/api_config_example.toml`
