[English](samples_EN.md) | 简体中文

# Caller 示例代码

这个目录包含了 Caller 库的各种用法示例，帮助开发者快速上手和理解库的各种功能。

## 示例列表

### 0. 配置文件示例

我创建了多种 JSON 配置文件示例，你可以根据项目需求选择使用：

#### [`api_config_example.json`](./api_config_example.json) - 完整配置示例
最详细的配置文件示例，包含：
- 多种认证方式（Bearer、Basic、Header、Query）
- 6个不同的 API 服务（GitHub、天气、新闻、翻译、Docker Registry、Mock API）
- 完整的参数配置、缓存设置、超时设置
- 全局配置项（重试、限流、熔断等）

#### [`minimal_config_example.json`](./minimal_config_example.json) - 最小配置示例
简洁的配置，适合快速开始：
- 最基本的认证配置
- 3个常用 API 端点
- 支持环境变量配置

#### [`ecommerce_config_example.json`](./ecommerce_config_example.json) - 电商配置示例
专门为电商项目设计的配置：
- Shopify 商店管理 API
- 支付宝/微信支付 API
- 京东商品搜索 API
- 库存管理 API
- 物流配送 API

### 1. [`basic_usage.rs`](./basic_usage.rs)
**基本用法示例**

展示了 Caller 的基本功能：
- 无参数 API 调用
- 路径参数使用
- 查询参数使用
- JSON 请求体
- 深度路径访问

```bash
cargo run --example basic_usage
```

**学习要点：**
- 理解 `call()` 函数的基本用法
- 掌握不同参数类型的使用
- 学习 `ApiResult` 的基本操作

### 2. [`advanced_usage.rs`](./advanced_usage.rs)
**高级用法示例**

展示了 Caller 的高级功能：
- 不同 HTTP 方法（GET、POST、PUT、PATCH）
- 复杂 JSON 数据操作
- 数组遍历和访问
- 层次化数据访问
- 批量操作示例

```bash
cargo run --example advanced_usage
```

**学习要点：**
- 理解 CRUD 操作的完整流程
- 掌握复杂数据结构的处理
- 学习体积数据的遍历

### 3. [`error_handling.rs`](./error_handling.rs)
**错误处理示例**

展示了各种错误场景的处理：
- 服务不存在错误
- 方法不存在错误
- 参数缺失错误
- HTTP 网络错误
- 错误重试机制
- 安全的错误处理函数

```bash
cargo run --example error_handling
```

**学习要点：**
- 错误类型识别和处理
- 安全的 API 调用模式
- 重试机制的实现

### 4. [`config_management.rs`](./config_management.rs)
**配置管理示例**

展示了配置管理的各种用法：
- 多环境配置文件创建
- 配置验证和检查
- 环境变量集成
- 热重载策略
- 配置文件对比

```bash
cargo run --example config_management
```

**学习要点：**
- 理解配置文件结构
- 多环境配置管理
- 配置验证和安全

## 如何运行示例

### 前置条件
确保你已经编译了 Caller 库：

```bash
cargo build
```

### 运行特定示例
```bash
cargo run --example basic_usage
cargo run --example advanced_usage
cargo run --example error_handling
cargo run --example config_management
```

### 运行所有示例
```bash
for example in basic_usage advanced_usage error_handling config_management; do
    echo "=== 运行示例: $example ==="
    cargo run --example $example
    echo
done
```

## 学完本示例后的技能

通过运行这些示例，你将能够：

### 基础技能
- ✅ 正确配置和使用 Caller 库
- ✅ 调用各种类型的 API 端点
- ✅ 处理路径参数、查询参数、JSON 参数
- ✅ 解析和访问 JSON 响应数据

### 中级技能
- ✅ 实现完整的 CRUD 操作
- ✅ 处理复杂的嵌套数据结构
- ✅ 批量数据操作和处理
- ✅ 错误处理和重试机制

### 高级技能
- ✅ 多环境配置管理
- ✅ 配置文件验证和安全
- ✅ 热重载策略实现
- ✅ 企业级错误处理模式

## 配置文件要求

在运行示例之前，请确保项目根目录有以下配置文件：

1. **caller.json** - 主要的 API 配置文件
   - 定义可用的 API 端点
   - 认证配置
   - 服务配置

### 快速开始

你可以选择使用我提供的示例配置文件：

#### 使用示例配置文件
```bash
# 复制最小化配置（适合快速测试）
cp samples/minimal_config_example.json caller.json

# 或者使用完整配置（功能最丰富）
cp samples/api_config_example.json caller.json

# 或者使用电商配置（如果做电商项目）
cp samples/ecommerce_config_example.json caller.json
```

#### 自定义配置
如果需要自己的配置文件，可以：

1. 参考 `samples/` 目录中的示例配置文件
2. 根据实际 API 需求修改 `caller.json` 文件
3. 运行 `config_management` 示例查看配置验证详情

运行 `config_management` 示例时会自动创建示例配置文件，其他示例需要 `caller.json` 正确配置。

## 自定义示例

你可以基于这些示例创建自己的用法示例：

1. 复制现有示例文件
2. 修改为你实际使用的 API 端点
3. 根据业务需求调整参数和数据处理逻辑
4. 添加更多的错误处理和边界情况测试

## 性能优化建议

1. **缓存策略**: 对于频繁调用的 API，启用缓存
2. **连接池**: 复用 HTTP 连接，减少连接开销
3. **批量操作**: 使用批量 API 减少网络请求次数
4. **错误重试**: 实现合理的重试策略
5. **超时设置**: 为每个 API 设置适当的超时时间

## 常见问题

### Q: 示例运行失败怎么办？
A: 首先检查 `caller.json` 配置文件是否正确，然后查看具体的错误信息。

### Q: 如何添加自己的 API？
A: 在 `caller.json` 添加新的 service_items 和对应的 api_items。

### Q: 如何调试网络问题？
A: 查看 `ApiResult.raw` 字段获取原始响应，检查网络连接和 API 端点是否可达。

### Q: 如何处理认证失败？
A: 检查 authorizations 配置，确保 Token 和认证类型正确。

## 贡献

欢迎提交更多的用法示例！请确保：
- 示例代码有详细的注释
- 包含完整的错误处理
- 代码风格符合 Rust 标准格式
- 提供运行说明和学习要点