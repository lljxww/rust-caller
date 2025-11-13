//! 错误处理示例
//!
//! 这个示例展示了 Caller 中的各种错误处理场景：
//! - 服务不存在错误
//! - 方法不存在错误
//! - 参数缺失错误
//! - HTTP 错误处理
//! - JSON 解析错误处理

use caller::call;
use caller::CallerError;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    println!("=== Caller 错误处理示例 ===\n");

    // 示例1: 服务不存在错误
    println!("1. 服务不存在错误:");
    match call("nonexistentService.list", None).await {
        Ok(_) => println!("   ❌ 不应该成功!"),
        Err(CallerError::ServiceNotFound(service_name)) => {
            println!("   ✅ 捕获到服务不存在错误: {}", service_name);
        }
        Err(e) => println!("   ❌ 意外的错误类型: {}", e),
    }
    println!();

    // 示例2: 方法不存在错误
    println!("2. 方法不存在错误:");
    match call("JP.nonexistentMethod", None).await {
        Ok(_) => println!("   ❌ 不应该成功!"),
        Err(CallerError::MethodNotFound(service_method)) => {
            println!("   ✅ 捕获到方法不存在错误: {}", service_method);
        }
        Err(e) => println!("   ❌ 意外的错误类型: {}", e),
    }
    println!();

    // 示例3: 路径参数缺失错误
    println!("3. 路径参数缺失错误:");
    match call("JP.get", None).await {
        Ok(_) => println!("   ❌ 不应该成功!"),
        Err(CallerError::ParamMissing(param_name)) => {
            println!("   ✅ 捕获到参数缺失错误: {}", param_name);
        }
        Err(e) => println!("   ❌ 意外的错误类型: {}", e),
    }
    println!();

    // 示例4: HTTP 错误处理
    println!("4. HTTP 错误处理:");
    // 使用一个可能返回错误码的API
    let path_params = HashMap::from([
        ("post_id".to_string(), "999999".to_string()), // 很可能不存在的ID
    ]);
    match call("JP.get", Some(path_params)).await {
        Ok(result) => {
            println!("   ⚠️ HTTP 请求成功，但状态码可能不是 200");
            println!("   状态码: {}", result.status_code);
            println!("   原始响应: {}", &result.raw[..100.min(result.raw.len())]);
        }
        Err(CallerError::HttpError(_)) => {
            println!("   ✅ 捕获到HTTP错误（可能是网络问题）");
        }
        Err(e) => println!("   ❌ 捕获到其他错误: {}", e),
    }
    println!();

    // 示例5: 复杂错误处理组合
    println!("5. 错误处理组合模式:");
    let test_cases = vec![
        ("JP.get", None, "缺少路径参数"),
        ("JP.create", HashMap::from([("title".to_string(), "测试".to_string())]), "缺少json参数"),
        ("InvalidService.method", None, "服务不存在"),
    ];

    for (method, params, description) in test_cases {
        println!("   {}: {}", description, method);
        match call(method, params).await {
            Ok(result) => {
                println!("      ⚠️ 意外成功，状态码: {}", result.status_code);
            }
            Err(e) => {
                println!("      ✅ 捕获错误: {}", e);
                match e {
                    CallerError::ServiceNotFound(_) => {
                        println!("          错误类型: 服务不存在");
                    }
                    CallerError::MethodNotFound(_) => {
                        println!("          错误类型: 方法不存在");
                    }
                    CallerError::ParamMissing(_) => {
                        println!("          错误类型: 参数缺失");
                    }
                    CallerError::HttpError(_) => {
                        println!("          错误类型: HTTP网络错误");
                    }
                    CallerError::JsonError(_) => {
                        println!("          错误类型: JSON解析错误");
                    }
                    CallerError::AuthenticationError(_) => {
                        println!("          错误类型: 认证错误");
                    }
                    CallerError::ConfigError(_) => {
                        println!("          错误类型: 配置错误");
                    }
                }
            }
        }
        println!();
    }

    // 示例6: 安全的错误处理函数
    println!("6. 安全的错误处理函数:");
    fn safe_api_call(method: &str, params: Option<HashMap<String, String>>) {
        println!("   尝试调用: {}", method);

        match call(method, params).await {
            Ok(result) => {
                if result.status_code.is_success() {
                    println!("      ✅ 调用成功");
                    // 安全地获取第一个元素的ID
                    if let Some(id) = result.get_as_i64("0.id") {
                        println!("      第一个元素的ID: {}", id);
                    }
                } else {
                    println!("      ⚠️ 调用返回非2xx状态码: {}", result.status_code);
                    println!("      响应预览: {}", &result.raw[..50.min(result.raw.len())]);
                }
            }
            Err(e) => {
                println!("      ❌ 调用失败: {}", e);
            }
        }
        println!();
    }

    // 测试各种情况
    safe_api_call("JP.list", None);
    safe_api_call("JP.get", Some(HashMap::from([("post_id".to_string(), "1".to_string())])));
    safe_api_call("JP.get", None); // 应该失败，缺少post_id

    // 示例7: 错误重试机制
    println!("7. 错误重试机制示例:");
    async fn with_retry<F, Fut, R>(
        method: &str,
        params: Option<HashMap<String, String>>,
        max_retries: usize,
    ) -> Result<R, CallerError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<R, CallerError>>,
    {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match call(method, params.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        println!("      重试 {}/{}...", attempt + 1, max_retries);
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    // 测试重试机制（第一次调用会失败，第二次会成功）
    match with_retry("JP.list", None, 3).await {
        Ok(result) => {
            println!("   ✅ 重试成功，最后结果: 状态码 {}", result.status_code);
        }
        Err(e) => {
            println!("   ❌ 重试后仍然失败: {}", e);
        }
    }
    println!();

    println!("=== 错误处理示例完成 ===");
}