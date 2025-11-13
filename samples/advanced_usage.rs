//! 高级用法示例
//!
//! 这个示例展示了 Caller 的高级功能，包括：
//! - 不同 HTTP 方法的使用
//! - 错误处理
//! - 条件缓存
//! - 结果遍历

use caller::call;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Caller 高级用法示例 ===\n");

    // 示例1: 使用 PATCH 更新资源
    println!("1. 使用 PATCH 方法更新资源:");
    let update_params = HashMap::from([
        ("post_id".to_string(), "1".to_string()),
        ("title".to_string(), "更新后的标题".to_string()),
    ]);
    let result = call("JP.patch", Some(update_params)).await?;

    println!("   更新状态码: {}", result.status_code);
    println!("   更新后的标题: {}", result.get_as_str("title").unwrap_or("N/A"));
    println!();

    // 示例2: 使用 PUT 替换资源
    println!("2. 使用 PUT 方法替换资源:");
    let replace_params = HashMap::from([
        ("post_id".to_string(), "2".to_string()),
        ("title".to_string(), "完全替换的文章".to_string()),
        ("body".to_string(), "这是替换后的内容。".to_string()),
        ("userId".to_string(), "1".to_string()),
    ]);
    let result = call("JP.update", Some(replace_params)).await?;

    println!("   替换状态码: {}", result.status_code);
    println!("   替换后的标题: {}", result.get_as_str("title").unwrap_or("N/A"));
    println!();

    // 示例3: 删除资源 (使用 PATCH，因为配置中定义为 PATCH)
    println!("3. 删除资源 (模拟删除):");
    let delete_params = HashMap::from([
        ("post_id".to_string(), "1".to_string()),
        ("title".to_string(), "\"\"".to_string()), // 空字符串表示删除
    ]);
    let result = call("JP.delete", Some(delete_params)).await?;

    println!("   删除状态码: {}", result.status_code);
    println!("   删除响应: {}", result.raw);
    println!();

    // 示例4: 多层JSON数据操作
    println!("4. 复杂 JSON 数据操作:");

    // 先创建一个包含嵌套结构的文章
    let nested_params = HashMap::from([
        ("title".to_string(), "嵌套数据测试".to_string()),
        ("body".to_string(), r#"{
            "metadata": {
                "author": "Rust Sample",
                "category": "教程",
                "tags": ["rust", "api", "caller"]
            },
            "stats": {
                "views": 100,
                "likes": 25,
                "comments": 3
            }
        }"#.to_string()),
        ("userId".to_string(), "1".to_string()),
    ]);
    let result = call("JP.create", Some(nested_params)).await?;

    println!("   创建状态码: {}", result.status_code);

    // 深度路径访问
    if let Some(author) = result.get("metadata").and_then(|m| m.get("author")).and_then(|a| a.as_str()) {
        println!("   作者: {}", author);
    }

    if let Some(views) = result.get("stats").and_then(|s| s.get("views")).and_then(|v| v.as_i64()) {
        println!("   浏览量: {}", views);
    }
    println!();

    // 示例5: 批量操作示例
    println!("5. 批量操作示例:");
    let mut user_ids = vec!["1", "2", "3"];
    let mut all_posts = Vec::new();

    for user_id in user_ids.iter() {
        let batch_params = HashMap::from([
            ("userId".to_string(), user_id.to_string()),
        ]);
        let result = call("JP.filter", Some(batch_params)).await?;

        if let Some(posts) = result.get_as_str("") {
            println!("    用户 {} 的文章数量: {}", user_id, posts.chars().filter(|c| *c == '[').count());
            all_posts.push(format!("用户{}的文章: {}", user_id, posts));
        }
    }

    println!("   总共获取了 {} 个用户的文章", all_posts.len());
    println!();

    // 示例6: 条件缓存的使用（实际缓存逻辑在库内部）
    println!("6. 微博热搜 API 示例:");
    let hot_search_result = call("weibo.hot", None).await?;

    println!("   微博热搜状态码: {}", hot_search_result.status_code);

    // 假设返回的数据格式
    if let Some(raw_data) = hot_search_result.get_as_str("") {
        println!("   热搜数据示例: {}", if raw_data.len() > 100 {
            format!("{}...", &raw_data[..100])
        } else {
            raw_data.to_string()
        });
    }
    println!();

    // 示例7: 访问嵌套数组的元素
    println!("7. 数组访问示例:");
    let all_posts = call("JP.list", None).await?;

    // 访问前5篇文章
    for i in 0..5 {
        if let Some(id) = all_posts.get_as_i64(&format!("{}.id", i)) {
            if let Some(title) = all_posts.get_as_str(&format!("{}.title", i)) {
                println!("    文章 {}: {} - {}", i + 1, id, title);
            }
        } else {
            break;
        }
    }
    println!();

    // 示例8: 使用不同的内容类型
    println!("8. 内容类型演示:");
    println!("   当前支持的内容类型: application/json");
    println!("   路径参数和JSON参数的组合使用: 通过 config 配置");
    println!("   认证支持: 通过 Authorizations 节配置");
    println!();

    println!("=== 高级用法示例完成 ===");
    Ok(())
}