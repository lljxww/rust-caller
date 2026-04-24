# Caller 重构清单：2026-04 现状复检版

这份清单不再只是“理想目标列表”，而是基于当前仓库状态做过一次重新检测后的真实状态记录。

本次复检参考了以下事实：

- `cargo test --lib` 通过
- `cargo check --all-targets` 通过
- `cargo check --features server --all-targets` 通过
- `cargo check --examples` 通过
- `cargo test --doc` 通过
- `cargo clippy --all-targets -- -D warnings` 通过
- `cargo clippy --features server --all-targets -- -D warnings` 通过
- `cargo package --allow-dirty --list` 已复检发布包内容
- 默认 `cargo test` 已不再强依赖外网；外网测试已标记 `#[ignore = "requires external network access"]`

目标仍然是把 `caller` 从“可用工具库”继续打磨成“稳定、可组合、可测试、可发布”的 Rust crate，但当前文档应反映哪些工作已经完成，哪些还只是部分完成，哪些是下一阶段重点。

## 总体判断

当前 crate 已经明显越过“原型期”：

- 已有正式的实例化入口 `Caller` / `CallerBuilder`
- 全局 API 已通过实例模型委托，而不是继续以旧的全局执行路径为核心
- 认证缺失时默认 fail fast，不再静默降级
- `ApiResult` 已支持非 JSON 响应，不再强制 JSON-only
- 关键错误已经开始结构化，并提供 `ErrorCategory`
- 离线测试、doc tests、examples 编译检查已建立基本基线

本轮持续优化后，原清单中的问题项已经完成收口：

- 公开 API 面已经收紧到 crate root 稳定导出
- 配置模型底层已经类型化，同时保持现有配置文件格式兼容
- 配置合法性校验已经前移到 parse / build 阶段
- `CallerBuilder` 已覆盖实例默认值、header、user-agent、auth provider 与自定义 client
- `server` feature 已复用 `Caller` 实例执行路径，并且模块本身不再暴露为公共目录结构

## 状态分层

### 已完成

#### P0 当前明确错误

- `init_config()` 语义已修复。
  现在会真正加载并写入全局配置状态，且已有回归测试覆盖。
- 配置热重载链路已修复。
  文件变更后可触发实际 reload，已有自动化测试覆盖配置变更生效。
- 认证静默降级已修复。
  当配置声明 `authorization_type` 但 provider 未注册时，会返回结构化错误 `UnknownAuthProvider { name }`。
- README / examples / doc tests 基本一致性已大幅改善。
  当前 `cargo check --examples` 和 `cargo test --doc` 可通过。

#### P1 架构核心

- 已引入实例化 API：
  `Caller::from_path(path)`、`Caller::from_config(config)`、`Caller::builder()`
- 每个 `Caller` 实例已持有自己的：
  配置、认证 provider 集合、复用的 `reqwest::Client`
- 多实例隔离已有测试覆盖。
- HTTP client 已从“每次请求新建”改为“实例级复用”。
- `server` 代理请求路径已改为委托 `Caller::call`，不再维护第二套独立 HTTP 执行逻辑。

#### P2 公开 API 设计

- 已提供 builder 风格入口。
- `CallerBuilder` 现已支持一批实例级默认装配能力：
  默认 timeout、默认 header、自定义 user-agent、构建时预注册 auth provider
- `ConfigBuilder` / `ServiceBuilder` 已开始提供 typed config 入口：
  `api_typed(...)` 与 `ApiEndpointBuilder`
- `ApiResult` 已不再隐含“响应必须是 JSON”这一前提。
  现在已有 `ResponseBody::Json` / `Text` / `Bytes` 类型表达。

#### P3 错误模型与安全默认值

- 已引入一批结构化错误：
  `ServiceNotFound { service }`
  `ApiNotFound { service, method }`
  `MissingPathParameter { name }`
  `UnknownAuthProvider { name }`
  `UnsupportedParamType { value }`
- 已补充配置层结构化错误：
  `ConfigFileNotFound`
  `UnsupportedConfigFormat`
  `ConfigParseError`
  `ConfigSerializeError`
  `ConfigWatchError`
  `LockPoisoned`
- 已提供 `ErrorCategory`，可做基础分层：
  `Config` / `Runtime` / `Protocol` / `Security`
- 默认行为已比之前保守得多：
  缺配置、缺认证、缺路径参数等情况都不会继续“带病请求”。

#### P4 测试体系

- 默认测试已可离线运行。
- 外网测试已隔离为 `ignored`。
- 已加入：
  库内单元测试
  多格式配置测试
  doc tests
  examples 编译检查
  已知 bug 回归测试
- 严格 clippy 已纳入当前复检基线。

### 已收口项目

#### P2 收紧公开模块边界

- `core` / `infra` 已改为 `pub(crate)`
- `client` / `config` / `domain` / `openapi` / `server` 均已改为内部模块
- 稳定公共 API 通过 crate root 导出：
  `Caller`、`CallerBuilder`、`ConfigBuilder`、`ConfigLoader`、`ApiConfig`、`CallerConfig`、`OpenApiGenerator`、`ServerConfig` 等
- 文档、examples、tests 已改为使用 crate root 稳定路径

结论：公共模块边界已经收口，不再暴露内部目录结构作为稳定 API。

#### P2 明确同步 / 异步边界

- `call` / `call_with_retry` / `download` 是明确 async 的
- 配置加载、转换、watch 管理接口是同步的
- crate docs 已说明新代码优先使用实例 API，全局 API 用于简单场景
- `watch_config` / `watch_config_with_debounce` 已在 crate docs 中说明默认路径和 debounce 行为

结论：同步 / 异步边界和全局 / 实例职责边界已经清楚表达。

#### P3 错误分层

- 已有结构化错误和 `ErrorCategory`
- HTTP / 网络路径已补充：
  `RequestTimeout`
  `TooManyRedirects { message }`
  `ConnectionError { message }`
  `HttpClientBuildError { message }`
  `RetryableHttpStatus { status, attempt, max_retries }`
  `RetryAttemptsExhausted`
- builder 参数错误已补充：
  `InvalidHeaderName { name, message }`
  `InvalidHeaderValue { name, message }`
  `InvalidUserAgent { value, message }`
- URL 错误已补充：
  `InvalidUrl { url, message }`
  `UnsupportedUrlScheme { url, scheme }`
- 认证环境变量错误已补充：
  `MissingAuthEnvironmentVariable { name }`

结论：原清单列出的 HTTP / 参数 / 认证类字符串错误已拆出可匹配结构化路径；兼容性保留的泛化错误变体不再是当前执行路径的主要表达。

#### P5 配置大小写与格式规则

- JSON / YAML / TOML 多格式读写都已建立
- 示例配置可跨格式互转
- `ApiConfig` 内部字段已经类型化：
  `http_method: HttpMethod`
  `param_type: Vec<ParamType>`
- serde 仍保持外部配置格式兼容：
  `http_method = "GET"`
  `param_type = "path,json"`
- endpoint `url` 规则已明确：
  空字符串代表 service root；非空必须以 `/` 开头

结论：配置文件格式保持兼容，Rust 内部表达已经类型化，字段规则已定型。

#### P6 Feature 与依赖边界

- `tokio` 已不再使用 `full`，而是收紧为当前实际需要的特性集：
  `macros` / `rt-multi-thread` / `time` / `net`
- `reqwest` 已不再使用默认特性，改为：
  `default-features = false`
  `json`
  `rustls-tls`
- `cargo tree -i native-tls` 已确认 native-tls 不再进入依赖树
- `server` feature 下的代理执行路径已复用 `Caller` 实例能力：
  auth provider、参数处理、timeout、复用 HTTP client 等逻辑不再在 server 中重复实现
- `server` 模块本身已经内部化，只保留 crate root 的 feature-gated 函数导出

结论：feature 和依赖边界已按当前能力收紧。

#### P7 发布元数据成熟度

- `Cargo.toml` 已补充：
  `documentation`
  `keywords`
  `categories`
  `exclude`
- 发布包内容已做过一次 `cargo package --allow-dirty --list` 复检
- `.vscode/` 与根目录未引用的 `api_result_test.json` 已从发布包排除
- examples、samples、integration tests 与测试夹具保留在发布包中，作为 crate 使用示例和回归基线
- `Cargo.toml.orig` 出现在 `cargo package --list` 预览中，这是 Cargo 生成的打包辅助文件，不是仓库待清理文件

结论：发布元数据和包内容已经按当前发布策略收口。

#### P5 用类型代替字符串协议

- `ApiConfig` 底层存储已升级为：
  `HttpMethod`
  `Vec<ParamType>`
- JSON / YAML / TOML 配置文件继续以字符串形式读写，保持向后兼容
- `ApiConfig` 已提供：
  `http_method()`
  `param_types()`
  `has_param_type()`
  `validate()`
- `ConfigBuilder` / `ServiceBuilder` / `ApiEndpointBuilder` 已写入类型化字段
- `client` / `openapi` / `server` 的关键执行路径使用类型化配置

结论：类型化配置模型已完成，外部字符串格式只是 serde 兼容层。

#### P5 配置加载阶段即做完整校验

- `ConfigLoader` 在 parse 后已执行配置校验
- `Caller::from_config` / `CallerBuilder::build()` 也会校验传入配置
- 当前已前移的校验包括：
  非法 `http_method`
  非法 `param_type`
  `none` 与其他参数类型的非法组合
  重复 `param_type` 组合，例如 `query,query`
  非法或非 HTTP(S) `base_url`
  非空 API endpoint `url` 必须以 `/` 开头
  由 `base_url + api.url` 组成后的非法请求 URL
  service / api 重名冲突
- 非法 `http_method` / `param_type` 在 serde parse 阶段即失败
- `query,json` 这类组合语义作为稳定能力保留；`none` 不能与其他类型组合，重复类型会失败
- 认证 provider 引用需要结合运行时注册表判断，因此保持在 `Caller::call` 时 fail fast，返回 `UnknownAuthProvider { name }`

结论：配置加载阶段校验已完成；必须依赖运行时注册表的信息保留为运行时 fail-fast。

## 建议保留的回归基线

后续每次继续重构时，至少应保持以下命令持续通过：

```bash
cargo test --lib
cargo check --examples
cargo test --doc
```

另外建议把外网测试继续保持为显式隔离，不要重新混回默认测试集。

## 当前结论

`caller` 已经完成了第一轮最关键的架构和安全性修正。

现在最需要的不是再补一堆零散功能，而是继续做两类“会改变长期质量上限”的工作：

- 让配置模型真正类型化
- 让公开 API 边界和 feature 边界真正稳定

换句话说，当前 crate 的主要问题已经不再是“明显 bug 很多”，而是“成熟度还不够彻底”。
