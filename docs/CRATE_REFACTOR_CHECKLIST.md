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

但距离“成熟 crate”还有几块硬骨头没有啃完：

- 公开 API 面仍然偏宽，兼容导出太多
- 配置模型仍大量依赖字符串协议
- 配置合法性校验仍有一部分拖到运行时
- `CallerBuilder` 已明显增强，但距离完整实例装配器还有空间
- `server` 模块和发布元数据还有继续收紧空间

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

### 部分完成

#### P2 收紧公开模块边界

现状：

- `core` / `infra` 已改为 `pub(crate)`
- crate root 已不再 `pub use config::*; pub use domain::*` 这种全量导出

但仍有问题：

- `pub mod client`
- `pub mod config`
- `pub mod domain`
- `pub mod openapi`
- `pub mod server`

这些模块本身仍然整体暴露，说明“稳定 API”和“内部目录结构”之间的边界还不够清晰。

结论：

- 这项不是未做，而是做了一半。
- 下一步应该决定哪些模块是正式公共面，哪些只保留根导出。

#### P2 明确同步 / 异步边界

现状：

- `call` / `call_with_retry` / `download` 是明确 async 的
- 配置加载、转换、watch 管理接口是同步的

但仍有问题：

- `watch_config` / `watch_config_with_debounce` 的运行时和线程行为只体现在实现里，没有被文档清楚说明
- 全局 API 与实例 API 的职责边界仍然偏模糊

结论：

- 基础边界已经有了
- 文档和命名层面还需要继续澄清

#### P3 错误分层

现状：

- 已有结构化错误和 `ErrorCategory`

但仍有问题：

- `HttpError(String)`、`ApiError(String)`、`ParameterError(String)`、`AuthenticationError(String)` 仍然存在
- 部分错误只是“先被归类”，并未完全做到“字段化可匹配”
- HTTP 状态异常、网络错误、响应解析失败之间还可以继续拉开

结论：

- P3 已经起势，但还没彻底完成

#### P5 配置大小写与格式规则

现状：

- JSON / YAML / TOML 多格式读写都已建立
- 示例配置可跨格式互转

但仍有问题：

- 配置字段仍以现有序列化表现为准，文档层面对字段大小写策略的说明不够系统
- 目前“能用”多于“规则已定型”

结论：

- 功能已具备
- 类型和规则说明还不算真正收口

#### P6 Feature 与依赖边界

现状：

- `tokio` 已不再使用 `full`，而是收紧为当前实际需要的特性集：
  `macros` / `rt-multi-thread` / `time` / `net`
- `server` feature 下的代理执行路径已复用 `Caller` 实例能力：
  auth provider、参数处理、timeout、复用 HTTP client 等逻辑不再在 server 中重复实现

但仍有问题：

- `reqwest` 特性还可以继续复检是否存在冗余
- `server` 模块本身仍为公开模块，边界尚未完全收口

结论：

- 这项已经开始推进
- 但还没有做到“按 feature 明确隔离公共面”

#### P7 发布元数据成熟度

现状：

- `Cargo.toml` 已补充：
  `documentation`
  `keywords`
  `categories`
  `exclude`
- 发布包内容已做过一次 `cargo package --allow-dirty --list` 复检
- `.vscode/` 与根目录未引用的 `api_result_test.json` 已从发布包排除

但仍有问题：

- 发布包还包含 examples、samples、integration tests 与测试夹具，是否全部适合 crates.io 仍需按发布策略确认
- `Cargo.toml.orig` 会出现在 `cargo package --list` 预览中，这是 Cargo 生成的打包辅助文件，不是仓库待清理文件
- 文档站点内容本身还未系统整理到适合 crates.io / docs.rs 首屏消费的程度

结论：

- crate 元数据和明显的本地杂项文件已经不再是主要短板
- 后续重点应转向“发布内容质量”而不只是字段补齐

### 尚未完成

#### P5 用类型代替字符串协议

现状：

- 配置文件与公开结构体字段为了兼容性，`http_method`、`param_type` 仍保留为字符串
- 但 crate 内部已经新增强类型解析：
  `HttpMethod`
  `ParamType`
- `ApiConfig` 已提供：
  `http_method()`
  `param_types()`
  `has_param_type()`
  `validate()`
- `client` / `openapi` / `server` 的关键执行路径已开始复用这些强类型解析，而不是各自手写字符串分支

但仍有问题：

- 配置的底层存储形式仍然是字符串，尚未真正升级为 enum / 集合字段
- `path,json` 这类组合语义仍然依赖逗号分隔文本作为源表示
- crate 对外暴露的配置构造方式还没有完全转向类型安全 API

结论：

- 这项已经从“纯待做”进入“兼容式迁移中”
- typed builder 入口已经出现，下一步要决定是否继续停留在“字符串存储 + typed builder”，还是升级到底层字段也类型化

#### P5 配置加载阶段即做完整校验

现状：

- `ConfigLoader` 在 parse 后已执行配置校验
- `Caller::from_config` / `CallerBuilder::build()` 也会校验传入配置
- 当前已前移的校验包括：
  非法 `http_method`
  非法 `param_type`
  `none` 与其他参数类型的非法组合
  非法或非 HTTP(S) `base_url`
  由 `base_url + api.url` 组成后的非法请求 URL
  service / api 重名冲突

但仍有问题：

- URL 校验已经能挡住基础格式错误，但还没有明确 endpoint path 的规范，例如是否必须以 `/` 开头、是否允许绝对 URL 覆盖 service `base_url`
- 不同参数类型组合的业务约束仍然比较宽松，例如 `query,json` 是否应作为稳定公共语义继续保留
- 认证 provider 引用合法性仍需运行时结合注册表判断

结论：

- “配置能否加载”和“配置能否基本安全执行”已经明显更接近同一阶段
- 基础 URL 合法性已经前移，但还没有达到完整静态校验的程度

## 重新排序后的下一阶段优先级

基于当前状态，建议后续不再按“救火顺序”推进，而按下面的成熟化顺序推进：

1. 继续推进配置模型类型化
   决定 typed config 的最终公共表达：继续兼容字符串存储，还是引入正式 enum / 集合字段与 builder。
2. 收紧公开 API 面
   确定正式公共模块边界，减少用户对内部目录结构的耦合。
3. 继续加强 `CallerBuilder`
   当前已支持 timeout、默认 header、user-agent、auth 注入；下一步可考虑 client builder 策略与 middleware 注入。
4. 细化错误模型
   继续消除剩余的字符串错误，尤其是 HTTP / 参数 / 认证类错误。
5. Feature / 依赖减重
   收窄 `tokio` 与 `reqwest` 特性，进一步隔离 `server`。
6. 发布元数据与文档成熟化
   完善 crates.io 质量、文档规则、配置格式说明。

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
