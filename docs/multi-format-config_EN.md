[English](multi-format-config_EN.md) | [简体中文](multi-format-config_CN.md)

# JSON, YAML, and TOML configuration

All formats map to the same snake_case model and receive identical validation.
File extensions are `.json`, `.yaml`/`.yml`, and `.toml`.

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

`ConfigFormat` is the canonical public type. `ConfigFileFormat` and
`BuilderConfigFormat` remain root aliases for source compatibility.

Equivalent minimal files:

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

Conversion parses and validates the input before writing output. Unknown fields
are errors; format conversion is therefore also a useful configuration check.
Output writes are not transactional: write to a temporary path and atomically
rename it when replacing production configuration.

Format conversion deliberately drops legacy `authorizations` metadata so a
conversion cannot replicate stored credentials. Use runtime auth providers.
