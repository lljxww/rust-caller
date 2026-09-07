# `caller` 0.4 成熟度与发布基线

更新时间：2026-09-07

本文记录 0.4 系列的能力边界、稳定性约定和发布门槛。它不是功能愿望清单，
而是维护者在改动公共 API、协议行为或发布配置时应执行的检查基线。

## 定位

`caller` 是配置驱动的异步 HTTP 客户端，适用于应用需要集中描述一组固定上游
API 的场景。它的稳定核心是：

- 一个 `Caller` 实例持有配置、认证 provider、middleware 和复用的
  `reqwest::Client`；
- `RequestArgs` 明确分离 path、query、form 和 JSON body；
- 配置在加载或构建阶段校验，运行时依赖（例如认证 provider）在发送前校验；
- 请求、重试、响应限制、下载、OpenAPI 和可选开发代理复用同一套配置语义；
- 默认功能不依赖 OpenSSL，`server` 能力通过 feature 隔离。

它不是通用浏览器客户端、流式传输框架或生产 API Gateway。需要动态 URL、
multipart、SSE、WebSocket、零拷贝大文件流或网关级治理时，应直接使用
`reqwest`、专用协议库或生产网关。

## 0.4 稳定性约定

### 请求语义

- 方法名格式固定为 `service.api`。
- 支持 GET、POST、PUT、DELETE、PATCH、HEAD 和 OPTIONS。
- service `base_url` 只能使用 HTTP(S)，不能包含凭据、query 或 fragment。
- endpoint URL 为空时表示 service root；非空时必须以 `/` 开头。
- path 参数按单个 path segment 百分号编码，不能改变既定路径层级。
- `RequestArgs` 是组合参数类型的推荐入口；未在 endpoint 声明的位置不接受参数。
- query/form pair API 保留重复 key 和输入顺序。
- JSON body 接收任意 `serde_json::Value`；`CallParams` 保留整数宽度，并拒绝
  JSON 无法表示的非有限浮点数。
- 兼容入口 `call` / `call_params` 会把同一组参数投射到 endpoint 声明的所有
  参数位置。该行为仅为兼容保留，新代码不应依赖它表达组合请求。

### 失败与重试语义

- 配置、协议、安全和运行时失败可通过 `ErrorCategory` 分类。
- HTTP 传输失败保留原始 `reqwest::Error` 作为 source，并提供
  `TransportErrorKind`。
- 收到 4xx/5xx 本身仍是成功收到响应；调用方可显式调用
  `ApiResult::error_for_status()` 转换为 `CallerError::HttpStatus`。
- 重试只覆盖配置的状态码和真实传输错误。认证、配置、参数及 middleware 拒绝
  不会被当作网络抖动重试。
- 达到重试上限时返回最后一次真实结果：状态码响应仍为 `Ok(ApiResult)`，传输
  失败则保留并返回最后一个 `HttpTransport`。
- 指数退避使用溢出安全计算，默认含 20% 正向抖动；整数秒格式的
  `Retry-After` 会被采用，但不超过 `max_delay`。

### 安全与资源边界

- 配置中的旧 `authorizations` 字段只为输入兼容而接受，加载后立即丢弃，序列化
  时不会输出。凭据必须通过运行时 provider 注入。
- 内置认证类型的 `Debug` 不输出 token、密码或 API key。
- 动态认证 provider 可以返回刷新错误；缺失环境变量不会退化为空凭据。
- 普通响应和下载分别有可配置的累计字节上限，既检查 `Content-Length`，也在
  逐块读取时检查实际长度。
- 自动下载文件名拒绝绝对路径、路径分隔符、控制字符及 `.` / `..`，并以
  `create_new` 创建文件，避免路径穿越和静默覆盖。
- `save_to_file` 是调用方显式指定路径的低层接口，仍具有覆盖已有文件的语义。
- 开发服务器默认只绑定回环地址且不开放任意 CORS；远程绑定必须显式允许。

### 并发与全局状态

- 推荐每个应用或上游域持有长生命周期 `Caller` 实例。
- 一个实例的配置、认证和 middleware 与其他实例隔离。
- 认证注册表与配置加载关键路径使用锁保护；可失败操作通过结构化错误报告锁中毒。
  watch 的停止/状态查询保持无失败接口，并只在这些观察型操作中恢复锁内数据。
- crate-root 全局函数用于脚本和单一全局配置场景。它们依赖进程级配置状态，
  不适合作为多租户隔离边界。

## 已完成的发布级收口

- 公共 API 从 crate root 导出，内部目录结构不作为兼容承诺。
- crate 级 `missing_docs` 与 `unreachable_pub` lint 已启用，docs.rs 可见接口必须有
  rustdoc，内部 `pub` 不会再被误当作公共兼容面。
- JSON、YAML、TOML 共用一个 `ConfigFormat`，旧名称仅作为类型别名兼容。
- 配置模型拒绝未知字段、重复 service/API、非法名称、零 timeout、非法 method、
  冲突参数类型、非法 URL 和 path placeholder 不一致。
- endpoint timeout 优先于 service timeout，service timeout 优先于 caller 默认值。
- middleware 完整包裹每次 attempt，响应阶段按注册逆序展开，错误钩子不吞异常。
- circuit breaker 在 half-open 状态只允许一个探测请求；本地配置/认证错误不计入
  上游失败，非目标失败状态会重置连续失败计数。
- OpenAPI 覆盖所有支持的方法，可依据配置生成认证引用，并允许调用方覆盖具体
  `SecurityScheme`。
- `reqwest` 使用 rustls 和受限 feature；Tokio 不启用 `full`；服务端依赖均为
  optional。
- 默认测试不访问公网；协议测试使用本地 TCP server 覆盖编码、重复 query、
  JSON body、超限响应和 `Retry-After`。
- 已声明 MSRV 为 Rust 1.88，CI 同时检查 MSRV 与 stable。

## 有意保留的限制

以下不是 0.4 的未完成 bug，而是需要新设计和相应 semver 评估的能力：

1. 响应和下载最终仍完整保存在内存中。字节上限能阻止无界增长，但不能替代
   流式写盘；大对象下载应等待独立 streaming API。
2. 尚无 multipart、流式 request body、SSE 或 WebSocket API。
3. 自动重试没有内置幂等键，也不会判断 POST 是否业务幂等。调用方必须谨慎配置
   非幂等方法的重试状态码。
4. OpenAPI 无法从字符串配置推导完整 JSON Schema；生成的 body/response schema
   是通用结构。认证 scheme 也需要在运行时通过 `security_scheme` 精确覆盖。
5. 开发代理没有用户认证、限流、TLS 终止或审计能力，不应部署为公网网关。
6. 配置热重载监控单个文件路径，不负责 Kubernetes ConfigMap 的目录级原子切换
   等所有部署模式；生产环境应按实际挂载行为做集成验证。
7. `need_cache`、`cache_time` 和 `use_new_http_client` 仅为旧配置反序列化兼容，
   当前无运行时行为。新配置不应写入这些字段。

## 0.4 破坏性变更检查

0.4 相对早期版本收紧了错误和构建接口，升级时重点检查：

- `ApiResult::build` 返回 `ApiResult`，不再返回无意义的 `Result`；
- `DownloadResult::from_response` 同样直接返回值，不再返回无意义的 `Result`；
- `CallParams::to_json`、`json_params!` 会报告非有限浮点数等转换错误；
- `HeaderMiddleware::with_header`、`RequestContext::with_header` 和
  `CircuitBreakerMiddleware::with_config` 返回 `Result`；
- `AuthRegistry::get` / `contains`、`Caller::has_auth` 和全局 `has_auth` 返回
  `Result`，使锁故障不再伪装成“未找到”；
- `OpenApiGenerator::new` 会先校验配置并返回 `Result`；
- `ServiceBuilder::api` 返回 `Result`，不存在的 service 不再 panic；
- `ConfigLoader::init_with_config` 返回 `Result`；
- 旧的字符串化 HTTP/网络错误已合并为带 source 的 `HttpTransport`；
- 无实际行为的 `RetryMiddleware` 已移除，重试必须使用
  `call_*_with_retry`。

## 发布门槛

每次发布必须在干净或明确审阅过的工作树上运行：

```bash
cargo fmt --all -- --check
cargo check --all-features --all-targets
cargo +1.88.0 check --all-features --all-targets
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features --all-targets
cargo test --doc
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
cargo package --allow-dirty
```

还应人工确认：

- `cargo package --allow-dirty --list` 没有凭据、下载产物或编辑器缓存；
- README、CHANGELOG、crate 版本、MSRV 和 feature 表述一致；
- 所有外网测试继续保持 `#[ignore = "requires external network access"]`；
- 公共 API 变更符合 semver，并在 CHANGELOG 的 migration notes 中列出；
- 生产使用方已评估响应上限、timeout、重试幂等性和 server 暴露范围。

## 后续演进优先级

如果继续扩展，推荐依次处理：

1. 设计不破坏现有缓冲 API 的 streaming response/download 接口；
2. 增加 multipart 和流式 request body，但避免把配置模型变成协议万能层；
3. 为 tracing/metrics 提供可选集成，保持默认依赖轻量；
4. 增加基于 API snapshot 的公共接口兼容检查；
5. 在确有用户场景后再评估异步 auth registry、DNS/连接池调优或更复杂的
   retry budget，不预先引入重量级抽象。

## 当前结论

在“配置驱动、缓冲式 HTTP 请求组件”这一明确边界内，0.4 已具备可发布候选所需
的配置校验、错误可观测性、安全默认值、资源上限、离线协议测试、MSRV 和发布
检查。其成熟度依赖上述边界保持清晰；将开发代理或内存下载误当作生产网关和
流式传输能力，不属于当前稳定承诺。
