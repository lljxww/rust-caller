//! 测试 call_params_with_retry 类型安全 API
//!
//! 这个测试文件专门测试带重试功能的类型安全参数 API

use caller::{CallParams, RetryConfig, call_params_with_retry, params};
use std::time::Duration;

#[tokio::test]
#[ignore = "requires external network access"]
async fn test_call_params_with_retry_basic() {
    caller::init_config().unwrap();

    let retry_config = RetryConfig::new();
    let result = call_params_with_retry("JP.list", None, retry_config).await;

    assert!(result.is_ok());
}

#[tokio::test]
#[ignore = "requires external network access"]
async fn test_call_params_with_retry_with_params() {
    caller::init_config().unwrap();

    let retry_config = RetryConfig::new();
    let result =
        call_params_with_retry("JP.get", Some(params! { "post_id" => 1 }), retry_config).await;

    assert!(result.is_ok());

    if let Ok(res) = result {
        let id = res.get_as_i64("id").unwrap_or(0);
        assert!(id > 0);
    }
}

#[tokio::test]
#[ignore = "requires external network access"]
async fn test_call_params_with_retry_custom_config() {
    caller::init_config().unwrap();

    let retry_config = RetryConfig::new()
        .with_max_retries(3)
        .with_base_delay(Duration::from_millis(500));

    let result = call_params_with_retry("JP.list", None, retry_config).await;

    assert!(result.is_ok());
}

#[tokio::test]
#[ignore = "requires external network access"]
async fn test_call_params_with_retry_with_builder() {
    caller::init_config().unwrap();

    let retry_config = RetryConfig::new();
    let params = params!().add("userId", "1").add("active", "true");

    let result = call_params_with_retry("JP.filter", Some(params), retry_config).await;

    assert!(result.is_ok());
}

#[tokio::test]
#[ignore = "requires external network access"]
async fn test_call_params_with_retry_with_json() {
    caller::init_config().unwrap();

    let retry_config = RetryConfig::new();
    let json_value = serde_json::json!({
        "title": "Test Post",
        "body": "Content",
        "userId": 1
    });

    let params = CallParams::from_json(json_value).unwrap();
    let result = call_params_with_retry("JP.create", Some(params), retry_config).await;

    assert!(result.is_ok());

    if let Ok(res) = result {
        let id = res.get_as_i64("id").unwrap_or(0);
        assert!(id > 0);
    }
}

#[tokio::test]
async fn test_call_params_with_retry_invalid_method() {
    caller::init_config().unwrap();

    let retry_config = RetryConfig::new();
    let result = call_params_with_retry("invalid.method", None, retry_config).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_call_params_with_retry_missing_param() {
    caller::init_config().unwrap();

    let retry_config = RetryConfig::new();
    let result = call_params_with_retry("JP.get", None, retry_config).await;

    // 应该返回错误，因为缺少必需的 post_id 参数
    assert!(result.is_err());
}

#[tokio::test]
async fn test_retry_config_builder() {
    let config = RetryConfig::new()
        .with_max_retries(5)
        .with_base_delay(Duration::from_millis(1000))
        .with_max_delay(Duration::from_secs(30));

    assert_eq!(config.max_retries, 5);
    assert_eq!(config.base_delay, Duration::from_millis(1000));
}

#[tokio::test]
#[ignore = "requires external network access"]
async fn test_call_params_with_retry_with_array() {
    caller::init_config().unwrap();

    let retry_config = RetryConfig::new();
    let result = call_params_with_retry(
        "JP.filter",
        Some(params! { "tags" => vec!["rust", "web"] }),
        retry_config,
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
#[ignore = "requires external network access"]
async fn test_call_params_with_retry_with_bool() {
    caller::init_config().unwrap();

    let retry_config = RetryConfig::new();
    let result = call_params_with_retry(
        "JP.filter",
        Some(params! { "active" => true }),
        retry_config,
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
#[ignore = "requires external network access"]
async fn test_call_params_with_retry_with_float() {
    caller::init_config().unwrap();

    let retry_config = RetryConfig::new();
    let result = call_params_with_retry(
        "JP.filter",
        Some(params! { "price" => 99.99 }),
        retry_config,
    )
    .await;

    assert!(result.is_ok());
}
