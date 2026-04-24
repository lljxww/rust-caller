# 多格式配置文件支持

`caller` 支持三种配置文件格式：

- JSON: `.json`
- YAML: `.yaml` / `.yml`
- TOML: `.toml`

这三种格式在语义上等价，都会解析成同一个 `CallerConfig` 结构。

## 加载方式

### 按扩展名自动识别

```rust
use caller::ConfigLoader;

let json = ConfigLoader::load_config_from_path("config.json")?;
let yaml = ConfigLoader::load_config_from_path("config.yaml")?;
let toml = ConfigLoader::load_config_from_path("config.toml")?;
```

### 显式指定格式

当文件扩展名不可靠时，可以强制指定：

```rust
use caller::{ConfigFileFormat, ConfigLoader};

let config = ConfigLoader::load_config_from_path_with_format(
    "config.data",
    ConfigFileFormat::Yaml,
)?;
```

## 默认全局路径

需要区分两类入口：

- `ConfigLoader::load_config_from_path(...)` / `Caller::from_path(...)`
  可以加载任意 `.json` / `.yaml` / `.yml` / `.toml`
- `init_config()` / `reload_config()` / `watch_config()`
  当前默认只绑定 `./caller.json`

这意味着：

- 如果你使用全局 API，最稳妥的是保留默认 `caller.json`
- 如果你使用 YAML/TOML，推荐直接走实例化 `Caller::from_path(...)`

## 配置示例

### JSON

```json
{
  "authorizations": [
    {
      "name": "BearerAuth",
      "authorization_info": "Bearer your-token"
    }
  ],
  "service_items": [
    {
      "api_name": "GitHub_API",
      "base_url": "https://api.github.com",
      "timeout": 10000,
      "api_items": [
        {
          "method": "get_user",
          "url": "/users/{username}",
          "http_method": "GET",
          "param_type": "path"
        }
      ]
    }
  ]
}
```

### YAML

```yaml
authorizations:
  - name: BearerAuth
    authorization_info: Bearer your-token

service_items:
  - api_name: GitHub_API
    base_url: https://api.github.com
    timeout: 10000
    api_items:
      - method: get_user
        url: /users/{username}
        http_method: GET
        param_type: path
```

### TOML

```toml
[[authorizations]]
name = "BearerAuth"
authorization_info = "Bearer your-token"

[[service_items]]
api_name = "GitHub_API"
base_url = "https://api.github.com"
timeout = 10000

[[service_items.api_items]]
method = "get_user"
url = "/users/{username}"
http_method = "GET"
param_type = "path"
```

## 字段命名规则

所有格式统一使用 `snake_case` 字段名，例如：

- `service_items`
- `api_name`
- `http_method`
- `authorization_type`

## 校验行为

配置不只是“能解析”就算通过；当前加载阶段还会做一轮基础语义校验。

已经前移到加载阶段的错误包括：

- 非法 `http_method`
- 非法 `param_type`
- `none,json` 这类非法组合
- service / api 重名冲突

因此，多格式支持不只是序列化层兼容，也共享同一套配置校验逻辑。

## 格式转换

### 自动按输出扩展名转换

```rust
use caller::ConfigLoader;

ConfigLoader::convert_config("config.json", "config.yaml")?;
ConfigLoader::convert_config("config.yaml", "config.toml")?;
ConfigLoader::convert_config("config.toml", "config.json")?;
```

### 显式指定输出格式

```rust
use caller::{ConfigFileFormat, ConfigLoader};

ConfigLoader::convert_config_with_format(
    "config.json",
    "my_config.txt",
    ConfigFileFormat::Yaml,
)?;
```

## 什么时候选哪种格式

- JSON: 适合工具链集成、机器生成、和其他系统共享
- YAML: 更适合人工维护和审阅
- TOML: 更贴近 Rust 生态，适合和 `Cargo.toml` 风格统一

## 参考示例

仓库内现成示例位于 `samples/`：

- `samples/api_config_example.json`
- `samples/api_config_example.yaml`
- `samples/api_config_example.toml`
- `samples/minimal_config_example.json`
