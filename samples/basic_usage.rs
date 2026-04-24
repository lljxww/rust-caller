//! 基本用法示例
//!
//! 这个示例展示了 Caller 的基本使用方法，包括：
//! - 无参数 API 调用
//! - 带参数的 API 调用
//! - 不同参数类型的使用

use caller::call;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Caller 基本用法示例 ===\n");

    // 示例1: 无参数调用 (获取文章列表)
    println!("1. 获取所有文章列表:");
    let result = call("JP.list", None).await?;

    println!("   状态码: {}", result.status_code);
    println!("   原始响应长度: {} 字符", result.raw.len());

    // 获取第一个文章的标题
    if let Some(title) = result.str_at("0.title") {
        println!("   第一个文章标题: {}", title);
    }
    println!();

    // 示例2: 路径参数调用
    println!("2. 获取特定文章 (ID=1):");
    let path_params = HashMap::from([
        ("post_id".to_string(), "1".to_string()),
    ]);
    let result = call("JP.get", Some(path_params)).await?;

    println!("   文章ID: {}", result.str_at("id").unwrap_or("N/A"));
    println!("   文章标题: {}", result.str_at("title").unwrap_or("N/A"));
    println!("   文章内容: {}", result.str_at("body").unwrap_or("N/A"));
    println!();

    // 示例3: 查询参数调用
    println!("3. 筛选文章 (用户ID=1):");
    let query_params = HashMap::from([
        ("userId".to_string(), "1".to_string()),
    ]);
    let result = call("JP.filter", Some(query_params)).await?;

    println!("   找到的文章数量: {}", result.value_at("0").map_or(0, |_| 1));
    println!();

    // 示例4: JSON 请求体调用
    println!("4. 创建新文章:");
    let json_params = HashMap::from([
        ("title".to_string(), "Rust 编程入门".to_string()),
        ("body".to_string(), "这篇文章介绍 Rust 的基本概念。".to_string()),
        ("userId".to_string(), "1".to_string()),
    ]);
    let result = call("JP.create", Some(json_params)).await?;

    println!("   创建状态码: {}", result.status_code);
    if let Some(new_id) = result.i64_at("id") {
        println!("   新文章ID: {}", new_id);
    }
    println!("   文章标题: {}", result.str_at("title").unwrap_or("N/A"));
    println!();

    // 示例5: 多个查询参数
    println!("5. 多参数查询:");
    let multi_params = HashMap::from([
        ("userId".to_string(), "1".to_string()),
        ("id".to_string(), "1".to_string()),
    ]);
    let result = call("JP.filter", Some(multi_params)).await?;

    println!("   筛选结果: 找到 ID=1 且 userId=1 的文章");
    println!("   文章标题: {}", result.str_at("0.title").unwrap_or("N/A"));
    println!();

    // 示例6: 深度路径访问
    println!("6. 深度路径访问示例:");
    // 先获取一个包含数组的响应
    let all_posts = call("JP.list", None).await?;

    // 访问数组中的嵌套字段
    if let Some(first_user_id) = all_posts.i64_at("0.userId") {
        println!("    第一篇文章的用户ID: {}", first_user_id);
    }

    if let Some(first_post_title) = all_posts.str_at("0.title") {
        println!("    第一篇文章标题: {}", first_post_title);
    }

    println!("\n=== 基本用法示例完成 ===");
    Ok(())
}