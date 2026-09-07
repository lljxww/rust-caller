[English](authentication_EN.md) | 简体中文

# 认证

认证 provider 以名称注册，并由 service 或 endpoint 的 `authorization_type`
引用；endpoint 配置优先于 service。未注册的 provider 会在网络 I/O 前返回
`UnknownAuthProvider`。

优先使用实例级注册：

```rust,no_run
use caller::{BearerAuth, Caller};

let caller = Caller::from_path("caller.json")?;
caller.register_auth("github", BearerAuth::from_env("GITHUB_TOKEN")?)?;
# Ok::<(), caller::CallerError>(())
```

crate root 的 `register_auth` 只服务于全局 `call` 函数，状态在进程内共享。测试
和多租户应用应使用实例注册表，避免共享可变状态。

## 内置 Provider

```rust,no_run
use caller::{ApiKeyAuth, BasicAuth, BearerAuth, CustomHeaderAuth, OAuth2Auth};

let bearer = BearerAuth::new("token".to_string());
let basic = BasicAuth::new("user".to_string(), "password".to_string());
let key = ApiKeyAuth::new("x-api-key".to_string(), "secret".to_string());
let oauth = OAuth2Auth::new("access-token".to_string());

let mut custom = CustomHeaderAuth::new();
custom.add_header("x-signature", "signature-value");
# Ok::<(), caller::CallerError>(())
```

`BearerAuth` 和 `OAuth2Auth` 默认都使用标准 `Authorization` Header。
`BasicAuth`、`BearerAuth`、`ApiKeyAuth` 和 `OAuth2Auth` 的 `from_env` 在构造
时读取一次环境变量并返回 `Result`。

## 动态凭据

动态 provider 会在每次请求 attempt（包括重试）执行：

```rust,no_run
use caller::{CallerError, DynamicBearerAuth};

let auth = DynamicBearerAuth::try_new(|| {
    obtain_current_token().map_err(|error| {
        CallerError::authentication_error(format!("刷新 token 失败：{error}"))
    })
});
# fn obtain_current_token() -> Result<String, &'static str> { Ok("token".into()) }
```

`DynamicBearerAuth::from_env` 会在请求时读取环境变量；
`from_shared(Arc<RwLock<String>>)` 适合由外部刷新 token。环境变量缺失或锁中毒
会返回明确错误，不再生成空凭据。动态 API key 和自定义 Header provider 具有
相同模式。可能失败的回调应使用 `try_new`，`new` 只用于不会失败的回调。

## 自定义认证

可实现 `Authenticator`，也可直接注册异步闭包：

```rust,no_run
use caller::Caller;

let caller = Caller::from_path("caller.json")?;
caller.register_auth_closure("signed", |builder, context| {
    let method = context.http_method.clone();
    async move { Ok(builder.header("x-signed-method", method)) }
})?;
# Ok::<(), caller::CallerError>(())
```

`AuthContext` 包含配置的 service/endpoint 名称、最终 URL、HTTP 方法、provider
名称和兼容用的标量参数 Map。该 Map 无法表达重复 query key；如果签名算法必须
对重复 key 做规范化，应在自定义 `reqwest::Client` 层对最终请求签名。

## 安全约束

- token 不应进入仓库配置、日志、panic 信息或 URL。
- 内置 provider 的 `Debug` 会隐藏密钥；自定义 middleware/authenticator 仍需
  自己实现脱敏。
- 优先从环境变量或 secret manager 获取凭据，刷新失败必须向上传递。
- 网络刷新 token 时不要持有写锁。
- Header 名称和值会在构建请求时由 reqwest 校验。
- `authorizations[].authorization_info` 是旧元数据，不会应用到请求；不要把它
  当作密钥存储。
