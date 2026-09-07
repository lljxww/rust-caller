[English](multi-format-config_EN.md) | 简体中文

# JSON、YAML 与 TOML 配置

三种格式都映射到同一套 snake_case 模型，并执行完全相同的校验。扩展名支持
`.json`、`.yaml`/`.yml` 和 `.toml`。

```rust,no_run
use caller::{ConfigFormat, ConfigLoader};

let detected = ConfigLoader::load_config_from_path("caller.yaml")?;
let explicit = ConfigLoader::load_config_from_path_with_format(
    "caller.conf",
    ConfigFormat::Toml,
)?;

ConfigLoader::convert_config("caller.json", "caller.yaml")?;
ConfigLoader::convert_config_with_format(
    "caller.yaml",
    "caller.conf",
    ConfigFormat::Json,
)?;
# Ok::<(), caller::CallerError>(())
```

`ConfigFormat` 是标准公共类型。`ConfigFileFormat` 和 `BuilderConfigFormat` 仅
作为 crate root 的源码兼容别名保留。

以下最小配置等价：

```json
{"service_items":[{"api_name":"health","base_url":"https://api.example.com","api_items":[]}]}
```

```yaml
service_items:
  - api_name: health
    base_url: https://api.example.com
    api_items: []
```

```toml
[[service_items]]
api_name = "health"
base_url = "https://api.example.com"
api_items = []
```

转换会先解析并校验输入，再写出目标文件。未知字段会报错，因此格式转换也可用作
配置检查。输出写入不是事务性的；替换生产配置时，应先写临时文件，再原子 rename。

转换会主动丢弃旧格式的 `authorizations` 元数据，避免格式转换复制已存储凭据。
认证应改用运行时 provider。
