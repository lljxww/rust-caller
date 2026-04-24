//! 配置管理示例
//!
//! 这个示例演示了如何：
//! - 自定义配置文件
//! - 使用多个配置文件
//! - 动态配置更新（热重载）
//! - 环境变量配置
//! - 配置热重载监听

use caller::{
    init_config, is_config_loaded, is_watching_config, reload_config, stop_watch_config,
    watch_config,
};
use std::collections::HashMap;
use std::time::Duration;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Caller 配置管理示例 ===\n");

    // 示例1: 创建不同的配置文件
    println!("1. 创建多个配置文件示例:");

    // 示例配置文件1 - 开发环境
    let dev_config = r#"
{
  "authorizations": [
    {
      "name": "BearerToken",
      "header_name": "authorization",
      "type": "Bearer",
      "token": "dev-secret-token-12345"
    }
  ],
  "service_items": [
    {
      "api_name": "dev_api",
      "base_url": "https://jsonplaceholder.typicode.com",
      "api_items": [
        {
          "method": "get_user",
          "url": "/users/{user_id}",
          "http_method": "GET",
          "param_type": "path"
        },
        {
          "method": "get_users",
          "url": "/users",
          "http_method": "GET",
          "param_type": "none"
        }
      ]
    }
  ]
}"#;

    // 示例配置文件2 - 生产环境
    let prod_config = r#"
{
  "authorizations": [
    {
      "name": "OAuthToken",
      "header_name": "authorization",
      "type": "Bearer",
      "token": "${PROD_API_TOKEN}"
    }
  ],
  "service_items": [
    {
      "api_name": "prod_api",
      "base_url": "https://api.example.com",
      "api_items": [
        {
          "method": "get_user_profile",
          "url": "/users/{user_id}/profile",
          "http_method": "GET",
          "param_type": "path",
          "timeout": 10000,
          "need_cache": true
        }
      ]
    }
  ]
}"#;

    // 写入配置文件
    fs::write("samples/dev_config.json", dev_config).await?;
    fs::write("samples/prod_config.json", prod_config).await?;

    println!("   ✅ 创建开发环境配置: samples/dev_config.json");
    println!("   ✅ 创建生产环境配置: samples/prod_config.json");
    println!();

    // 示例2: 配置文件结构分析
    println!("2. 配置文件结构:");

    let config_content = fs::read_to_string("caller.json").await?;
    let config: serde_json::Value = serde_json::from_str(&config_content)?;

    println!("   📋 配置文件内容分析:");
    if let Some(services) = config.get("service_items").and_then(|s| s.as_array()) {
        println!("   - HTTP 服务数量: {}", services.len());
        for service in services {
            if let Some(name) = service.get("api_name").and_then(|n| n.as_str()) {
                let base_url = service
                    .get("base_url")
                    .and_then(|u| u.as_str())
                    .unwrap_or("N/A");
                let api_count = service
                    .get("api_items")
                    .and_then(|a| a.as_array())
                    .unwrap_or(&vec![])
                    .len();
                println!(
                    "     - {}: {} 个API端点, 基础URL: {}",
                    name, api_count, base_url
                );
            }
        }
    }

    if let Some(auths) = config.get("authorizations").and_then(|a| a.as_array()) {
        println!("   - 认证配置数量: {}", auths.len());
        for auth in auths {
            if let Some(name) = auth.get("name").and_then(|n| n.as_str()) {
                let auth_type = auth.get("type").and_then(|t| t.as_str()).unwrap_or("N/A");
                println!("     - 认证名称: {}, 类型: {}", name, auth_type);
            }
        }
    }
    println!();

    // 示例3: 配置验证
    println!("3. 配置验证函数:");

    fn validate_config(config: &serde_json::Value) -> Result<(), String> {
        // 检查必需字段
        let required_fields = ["service_items"];
        for field in &required_fields {
            if !config.get(field).is_some() {
                return Err(format!("缺少必需字段: {}", field));
            }
        }

        // 检查 service_items 格式
        if let Some(services) = config.get("service_items").and_then(|s| s.as_array()) {
            for (index, service) in services.iter().enumerate() {
                if let Some(api_name) = service.get("api_name").and_then(|n| n.as_str()) {
                    if api_name.contains(' ') {
                        return Err(format!("服务名不能包含空格: (索引 {})", index));
                    }
                } else {
                    return Err(format!("服务缺少 api_name 字段: (索引 {})", index));
                }
            }
        }

        Ok(())
    }

    match validate_config(&config) {
        Ok(_) => println!("   ✅ caller.json 配置验证通过"),
        Err(e) => println!("   ❌ 配置验证失败: {}", e),
    }
    println!();

    // 示例4: 环境变量配置
    println!("4. 环境变量配置示例:");

    // 模拟设置环境变量
    let env_vars = [
        ("API_BASE_URL", "https://custom-api.example.com"),
        ("API_TOKEN", "env-based-token"),
        ("REQUEST_TIMEOUT", "30000"),
    ];

    let mut env_config = HashMap::new();
    env_config.insert("environment".to_string(), "development".to_string());

    println!("   模拟的环境变量配置:");
    for (key, value) in &env_vars {
        println!("     {}: {}", key, value);
        // 在实际应用中，这里会使用 std::env::set_var
    }

    // 示例配置模板
    let template_config = r#"
{
  "base_url": "${API_BASE_URL:-https://default.com}",
  "token": "${API_TOKEN}",
  "timeout": "${REQUEST_TIMEOUT:-5000}"
}"#;

    println!("   🔧 配置模板示例 (支持环境变量):");
    println!("   {}", template_config);
    println!();

    // 示例5: 不同环境的配置加载策略
    println!("5. 环境配置加载策略:");

    async fn load_config_for_env(env: &str) -> Result<String, String> {
        match env {
            "development" | "dev" => Ok(fs::read_to_string("samples/dev_config.json")
                .await
                .map_err(|e| e.to_string())?),
            "production" | "prod" => Ok(fs::read_to_string("samples/prod_config.json")
                .await
                .map_err(|e| e.to_string())?),
            "testing" | "test" => {
                // 返回一个测试配置
                let test_config = r#"
{
  "service_items": [
    {
      "api_name": "test_api",
      "base_url": "http://localhost:8080/test",
      "api_items": [
        {
          "method": "ping",
          "url": "/ping",
          "http_method": "GET",
          "param_type": "none"
        }
      ]
    }
  ]
}"#;
                Ok(test_config.to_string())
            }
            _ => Err(format!("不支持的环境: {}", env)),
        }
    }

    // 测试不同环境的配置加载
    for env in ["dev", "prod", "test"] {
        println!("   加载 {} 环境配置: ", env);
        match load_config_for_env(env).await {
            Ok(config) => {
                let config_json: serde_json::Value = serde_json::from_str(&config)
                    .map_err(|e| e.to_string())
                    .unwrap_or_default();

                let service_count = config_json
                    .get("service_items")
                    .and_then(|s| s.as_array())
                    .map(|arr| arr.len())
                    .unwrap_or(0);

                println!("     ✅ 成功加载 {} 个服务", service_count);
            }
            Err(e) => {
                println!("     ❌ 加载失败: {}", e);
            }
        }
    }
    println!();

    // 示例6: 配置热重载演示
    println!("6. 配置热重载示例:");

    println!("   🔄 启动配置热重载监听...");
    match watch_config() {
        Ok(_) => {
            println!("   ✅ 监听已启动: {}", is_watching_config());
            println!("   提示: 修改 caller.json 后配置将自动重新加载");
            println!("   等待 3 秒测试自动重载...\n");

            // 模拟配置变更检测
            let original_content = fs::read_to_string("caller.json").await?;
            tokio::time::sleep(Duration::from_secs(3)).await;

            // 演示手动重载
            println!("   手动触发配置重载...");
            if let Err(e) = reload_config() {
                println!("   ❌ 重载失败: {}", e);
            } else {
                println!("   ✅ 手动重载成功");
            }

            // 停止监听
            stop_watch_config();
            println!("   监听已停止: {}", !is_watching_config());
        }
        Err(e) => {
            println!("   ❌ 启动监听失败: {}", e);
        }
    }
    println!();

    // 示例7: 配置热重载 API 详解
    println!("7. 热重载 API 使用说明:");

    println!("   📖 可用的热重载函数:");
    println!("   - init_config()          - 从文件加载配置");
    println!("   - reload_config()        - 手动重新加载配置");
    println!("   - watch_config()         - 启动文件监听（默认 500ms 防抖）");
    println!("   - watch_config_with_debounce(d) - 自定义防抖时间");
    println!("   - stop_watch_config()    - 停止文件监听");
    println!("   - is_watching_config()   - 检查是否正在监听");
    println!("   - is_config_loaded()     - 检查配置是否已加载");
    println!();

    // 示例8: 配置文件对比分析
    println!("7. 配置文件对比分析:");

    // 读取原始配置文件
    let original_config = fs::read_to_string("caller.json").await?;
    let original_json: serde_json::Value = serde_json::from_str(&original_config)?;

    // 读取开发环境配置
    let dev_config_content = fs::read_to_string("samples/dev_config.json").await?;
    let dev_json: serde_json::Value = serde_json::from_str(&dev_config_content)?;

    println!("   🔍 配置对比分析:");

    let orig_services = original_json
        .get("service_items")
        .and_then(|s| s.as_array())
        .unwrap_or(&vec![])
        .len();

    let dev_services = dev_json
        .get("service_items")
        .and_then(|s| s.as_array())
        .unwrap_or(&vec![])
        .len();

    println!("   - 原配置服务数量: {}", orig_services);
    println!("   - 开发配置服务数量: {}", dev_services);
    println!(
        "   - 开发配置 Overrides: {}",
        if dev_services > 0 { "是" } else { "否" }
    );

    // 清理示例配置文件
    fs::remove_file("samples/dev_config.json").await.ok();
    fs::remove_file("samples/prod_config.json").await.ok();

    println!("   🧹 已清理示例配置文件");

    println!("\n=== 配置管理示例完成 ===");
    Ok(())
}
