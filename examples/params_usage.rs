//! 类型安全参数使用示例
//! 
//! 这个示例展示了如何使用新的参数构建器来替代 HashMap<String, String>
//! 
//! 运行: cargo run --example params_usage

use caller::{call_params, call_params_with_retry, params, CallParams, RetryConfig};
use std::collections::HashMap;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), caller::CallerError> {
    // 初始化配置
    caller::init_config()?;
    
    println!("=== 旧方式 vs 新方式对比 ===\n");
    
    // ==================== 旧方式：HashMap ====================
    println!("1. 旧方式（HashMap）:");
    let old_params = HashMap::from([
        ("post_id".to_string(), "1".to_string()),
    ]);
    println!("   代码: HashMap::from([(\"post_id\".to_string(), \"1\".to_string())])");
    println!("   问题：需要手动转换类型，代码冗长\n");
    
    // ==================== 新方式：params! 宏 ====================
    println!("2. 新方式（params! 宏）:");
    let result = call_params("JP.get", Some(params! {
        "post_id" => 1
    })).await?;
    println!("   代码: params! {{ \"post_id\" => 1 }}");
    println!("   优势：类型推断，代码简洁");
    println!("   结果: ID={}, Title={}\n", 
        result.get_as_i64("id").unwrap_or(0),
        result.get_as_str("title").unwrap_or("N/A")
    );
    
    // ==================== 新方式：Builder 模式 ====================
    println!("3. 新方式（Builder 模式）:");
    let params = params!()
        .add("userId", "1")
        .add("active", "true")
        .add("name", "Alice");
    let result = call_params("JP.filter", Some(params)).await?;
    println!("   代码: params!().add(\"userId\", \"1\").add(\"active\", \"true\")");
    println!("   优势：流式 API，易于链式调用");
    println!("   结果: 找到 {} 条记录\n", 
        result.get("0").is_some() as usize + result.get("1").is_some() as usize
    );
    
    // ==================== 支持多种数据类型 ====================
    println!("4. 支持多种数据类型:");
    
    // 整数
    let params_int = params! { "id" => 1 };
    println!("   - 整数: params! {{ \"id\" => 1 }}");
    
    // 布尔值
    let params_bool = params! { "active" => true };
    println!("   - 布尔: params! {{ \"active\" => true }}");
    
    // 浮点数
    let params_float = params! { "price" => 99.99 };
    println!("   - 浮点: params! {{ \"price\" => 99.99 }}");
    
    // 字符串
    let params_str = params! { "name" => "Rust" };
    println!("   - 字符串: params! {{ \"name\" => \"Rust\" }}");
    
    // 数组
    let params_arr = params! { "tags" => vec!["rust", "web"] };
    println!("   - 数组: params! {{ \"tags\" => vec![\"rust\", \"web\"] }}");
    
    println!("   类型自动推断，无需手动转换！\n");
    
    // ==================== 嵌套对象 (使用 JSON) ====================
    println!("5. 嵌套对象（使用 JSON 方式）:");
    let json_value = serde_json::json!({
        "user": {
            "name": "Alice",
            "age": 30,
            "active": true
        },
        "action": "login"
    });
    let params = CallParams::from_json(json_value).unwrap();
    println!("   代码: CallParams::from_json(json!({{\"user\": {{...}}, \"action\": \"login\"}}))");
    println!("   JSON: {}", serde_json::to_string_pretty(&params.to_json()).unwrap());
    println!();
    
    // ==================== JSON 方式 ====================
    println!("6. 使用 JSON:");
    let json_value = serde_json::json!({
        "title": "My Post",
        "body": "Content here",
        "userId": 1
    });
    let params = CallParams::from_json(json_value).unwrap();
    let result = call_params("JP.create", Some(params)).await?;
    println!("   代码: CallParams::from_json(json!({{\"title\": \"My Post\", \"userId\": 1}}))");
    println!("   优势：与 JSON 格式一致，易于从 JSON 转换");
    println!("   结果: 创建成功，ID={}\n", result.get_as_i64("id").unwrap_or(0));
    
    // ==================== 带重试的调用 ====================
    println!("7. 带重试的类型安全调用:");
    let retry_config = RetryConfig::new()
        .with_max_retries(3)
        .with_base_delay(Duration::from_millis(500));
    
    let result = call_params_with_retry(
        "JP.get",
        Some(params! { "post_id" => 1 }),
        retry_config
    ).await?;
    println!("   代码: call_params_with_retry(\"JP.get\", params! {{ \"post_id\" => 1 }}, config)");
    println!("   结果: 成功获取，Status={}\n", result.status_code);
    
    // ==================== 兼容性：从 HashMap 转换 ====================
    println!("8. 向后兼容：从 HashMap 转换:");
    let old_hashmap = HashMap::from([
        ("key1".to_string(), "value1".to_string()),
        ("key2".to_string(), "value2".to_string()),
    ]);
    let new_params = CallParams::from_hashmap(old_hashmap);
    println!("   代码: CallParams::from_hashmap(hashmap)");
    println!("   结果: 可以无缝迁移旧代码！\n");
    
    // ==================== 对比总结 ====================
    println!("=== 总结对比 ===");
    println!("┌─────────────┬─────────────────────┬─────────────────────┐");
    println!("│   特性       │   旧方式 (HashMap)   │   新方式 (params!)   │");
    println!("├─────────────┼─────────────────────┼─────────────────────┤");
    println!("│  类型安全     │  ❌ 所有值都是 String  │  ✅ 类型推断         │");
    println!("│  代码简洁性   │  ❌ 冗长，需要 .to_string() │  ✅ 简洁直观         │");
    println!("│  多类型支持   │  ❌ 需要手动转换       │  ✅ 自动转换          │");
    println!("│  数组/对象    │  ❌ 不支持             │  ✅ 原生支持          │");
    println!("│  编译时检查   │  ❌ 运行时发现错误      │  ✅ 编译时发现         │");
    println!("│  IDE 支持     │  ⚠️  有限              │  ✅ 完整的类型提示     │");
    println!("│  向后兼容     │  -                   │  ✅ 支持转换          │");
    println!("└─────────────┴─────────────────────┴─────────────────────┘");
    
    println!("\n推荐：新项目使用 params! 宏，旧项目可以逐步迁移！");
    
    Ok(())
}