[English](multi-format-config_EN.md) | [简体中文](multi-format-config_CN.md)

# Multi-Format Configuration File Support

`rust-caller` now supports multiple configuration file formats, including JSON, YAML, and TOML.

## Supported Formats

- **JSON** (`.json`)
- **YAML** (`.yaml`, `.yml`)
- **TOML** (`.toml`)

## Usage

### Auto-Detect Format

The configuration loader automatically detects the format based on file extension:

```rust
use caller::config::config_loader::ConfigLoader;

// Auto-detect based on extension
let config = ConfigLoader::load_config_from_path("config.json")?;
let config = ConfigLoader::load_config_from_path("config.yaml")?;
let config = ConfigLoader::load_config_from_path("config.toml")?;
```

### Explicitly Specify Format

You can also explicitly specify the configuration file format:

```rust
use caller::config::config_loader::{ConfigFormat, ConfigLoader};

// Explicitly specify format
let config = ConfigLoader::load_config_from_path_with_format(
    "myconfig",
    ConfigFormat::Yaml
)?;
```

## Configuration File Examples

### JSON Configuration Example

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

### YAML Configuration Example

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

### TOML Configuration Example

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

## Important Notes

1. **Field Name Format**: All field names in configuration files must use PascalCase format (e.g., `Authorizations`, `ServiceItems`, `ApiName`) to match the `serde` rename configuration in Rust structs.

2. **Configuration File Location**: The default configuration file path is `./caller.json`, but you can load configuration files from any location using the `load_config_from_path` method.

3. **Format Selection Recommendations**:
   - JSON: Suitable for scenarios requiring integration with other tools
   - YAML: More readable, suitable for manual editing
   - TOML: Rust ecosystem recommended format, concise syntax

4. **Error Handling**: If the configuration file format is incorrect or contains an unsupported extension, it will return `CallerError::ConfigError`.

## Configuration File Format Conversion

`rust-caller` provides built-in configuration file format conversion functionality, allowing easy conversion between different formats.

### Auto-Detect Conversion

Automatically detect target format based on output file extension:

```rust
use caller::config::config_loader::ConfigLoader;

// JSON to YAML
ConfigLoader::convert_config("config.json", "config.yaml")?;

// YAML to TOML
ConfigLoader::convert_config("config.yaml", "config.toml")?;

// TOML to JSON
ConfigLoader::convert_config("config.toml", "config.json")?;
```

### Explicitly Specify Format

You can also explicitly specify the output format, ignoring the file extension:

```rust
use caller::config::config_loader::{ConfigLoader, ConfigFormat};

// Force output as YAML format, regardless of file extension
ConfigLoader::convert_config_with_format(
    "config.json",
    "my_config.txt",
    ConfigFormat::Yaml
)?;
```

### Use Cases

The configuration file conversion feature is particularly useful in the following scenarios:

1. **Team Collaboration**: Different team members may prefer different configuration formats, and the conversion feature allows for easy adaptation
2. **System Integration**: Convert configurations to formats compatible with other tools
3. **Configuration Migration**: When switching from one tool to another, batch convert configuration files
4. **Format Unification**: Unify existing configurations in various formats into a standard format

### Data Integrity Guarantee

The conversion process will:
- Preserve all configuration fields and data
- Maintain field name format (PascalCase)
- Validate the output file's validity
- Support formatted output (JSON and TOML use pretty formatting)

## Complete Examples

See complete configuration file examples in the `samples/` directory:
- `samples/api_config_example.json`
- `samples/api_config_example.yaml`
- `samples/api_config_example.toml`