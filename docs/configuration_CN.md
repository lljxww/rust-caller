[English](configuration_EN.md) | 简体中文

# 配置

配置文件只描述路由与传输行为。认证密钥应在 Rust 运行时注册，不应写进配置文件。

## 完整 JSON 结构

```json
{
  "service_items": [
    {
      "api_name": "catalog",
      "authorization_type": "catalog_token",
      "base_url": "https://api.example.com/v1",
      "timeout": 10000,
      "api_items": [
        {
          "method": "get_product",
          "url": "/products/{id}",
          "http_method": "GET",
          "param_type": "path,query",
          "description": "获取单个商品",
          "timeout": 3000
        },
        {
          "method": "update_product",
          "url": "/products/{id}",
          "http_method": "PATCH",
          "param_type": "path,json",
          "content_type": "application/json",
          "authorization_type": "admin_token"
        }
      ]
    }
  ]
}
```

`authorizations` 可省略，默认空列表。该字段只作为旧格式的输入元数据保留，不会
创建运行时 authenticator，序列化或格式转换时会被省略。不要把密钥写入
`authorization_info`；应注册与 `authorization_type` 同名的运行时 provider。

## Service 字段

| 字段 | 必填 | 说明 |
|---|---:|---|
| `api_name` | 是 | `service.method` 中的唯一服务名；非空、不能含 `.`、不能有首尾空格 |
| `base_url` | 是 | HTTP(S) 绝对 URL；不能含用户凭据、query 或 fragment |
| `authorization_type` | 否 | endpoint 默认继承的运行时认证 provider 名称 |
| `timeout` | 否 | 请求超时，单位毫秒，必须大于 0 |
| `api_items` | 是 | endpoint 列表，允许为空 |
| `use_new_http_client` | 否 | 兼容旧格式的保留字段，当前无行为 |

## Endpoint 字段

| 字段 | 必填 | 说明 |
|---|---:|---|
| `method` | 是 | service 内唯一的方法名；非空、不能含 `.`、不能有首尾空格 |
| `url` | 是 | 空串表示 service 根路径；非空时必须以 `/` 开头 |
| `http_method` | 是 | `GET`、`POST`、`PUT`、`DELETE`、`PATCH`、`HEAD` 或 `OPTIONS` |
| `param_type` | 是 | 一个或多个逗号分隔的参数位置 |
| `description` | 否 | OpenAPI 使用的描述 |
| `content_type` | 否 | 显式请求 Content-Type |
| `authorization_type` | 否 | endpoint 级认证 provider 覆盖 |
| `timeout` | 否 | endpoint 级超时，单位毫秒 |
| `need_cache`、`cache_time`、`use_new_http_client` | 否 | 兼容旧格式的保留字段，当前无行为 |

参数位置支持 `none`、`path`、`query`、`json` 和 `form`。`none` 不能与其他
值组合，重复类型以及 `json,form` 会被拒绝。URL 含 `{name}` 时必须声明
`path`；声明 `path` 时也必须存在至少一个占位符。

超时优先级是 endpoint、service、`CallerBuilder` 默认值（30 秒）。JSON 请求
默认使用 `application/json`，其他请求类型不会被自动添加 Content-Type。

## 加载与校验

```rust,no_run
use caller::{Caller, ConfigLoader};

let caller = Caller::from_path("config/caller.yaml")?;
let config = ConfigLoader::load_config_from_path("config/caller.toml")?;
# Ok::<(), caller::CallerError>(())
```

扩展名决定 JSON、YAML 或 TOML。解析后立即检查名称、URL、重复 service/
endpoint、参数组合、超时、Header 值和未知字段。运行时认证注册表只能在准备
请求时校验。

程序化构造应以 `build_validated()` 收口：

```rust
use caller::{ConfigBuilder, HttpMethod, ParamType};

let mut builder = ConfigBuilder::new();
builder
    .service("health", "https://api.example.com")
    .api_typed("check", "/health", HttpMethod::Get, [ParamType::None])
    .build();
let config = builder.build_validated()?;
# Ok::<(), caller::CallerError>(())
```

字符串版本的 `ServiceBuilder::api(...)` 会返回 `Result`。新代码优先使用
`api_typed` 或 `api_endpoint`。

## 全局配置与监听

crate root 的 `init_config`、`reload_config` 和 watch 函数固定使用
`./caller.json`。任意路径应使用实例化 `Caller`。

```rust,no_run
use caller::{init_config, last_config_watch_error, watch_config};

init_config()?;
watch_config()?;
if let Some(error) = last_config_watch_error() {
    eprintln!("最近一次异步重载失败：{error}");
}
# Ok::<(), caller::CallerError>(())
```

监听重载带 debounce。错误配置不会替换最后一份有效配置；文件系统回调无法把
异步错误返回给最初调用者，因此要检查 `last_config_watch_error()`。
`ConfigLoader::init_with_config` 也会先校验再修改全局状态。

当前不支持在 URL 或其他字段中做 `${ENV_VAR}` 插值。应在构造配置前解析环境
差异，或为不同环境选择不同文件。禁止把凭据嵌入 URL。
