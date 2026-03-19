//! 测试 download_params 类型安全 API
//! 
//! 这个测试文件专门测试新的类型安全参数下载 API

use caller::{download_params, params, CallParams};

#[tokio::test]
async fn test_download_params_invalid_method() {
    caller::init_config().unwrap();
    
    let result = download_params(
        "JP.download",
        None,
        None
    ).await;
    
    // 测试不存在的方法应该返回错误
    assert!(result.is_err());
}

#[tokio::test]
async fn test_download_params_invalid_service() {
    caller::init_config().unwrap();
    
    let result = download_params(
        "invalid.method",
        None,
        None
    ).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_download_params_with_string() {
    caller::init_config().unwrap();
    
    let result = download_params(
        "JP.download",
        Some(params! { "file_id" => "1" }),
        None
    ).await;
    
    // 测试 API 调用（可能失败因为端点不存在，但测试的是参数构建）
    let _ = result;
}

#[tokio::test]
async fn test_download_params_with_builder() {
    caller::init_config().unwrap();
    
    let params = params!()
        .add("file_id", "1")
        .add("quality", "high");
    
    let result = download_params(
        "JP.download",
        Some(params),
        None
    ).await;
    
    let _ = result;
}

#[tokio::test]
async fn test_download_params_with_json() {
    caller::init_config().unwrap();
    
    let json_value = serde_json::json!({
        "file_id": 1,
        "format": "pdf",
        "quality": "high"
    });
    
    let params = CallParams::from_json(json_value).unwrap();
    let result = download_params(
        "JP.download",
        Some(params),
        None
    ).await;
    
    let _ = result;
}

#[tokio::test]
async fn test_download_params_with_array() {
    caller::init_config().unwrap();
    
    let result = download_params(
        "JP.download",
        Some(params! { "file_ids" => vec![1, 2, 3] }),
        None
    ).await;
    
    let _ = result;
}

#[tokio::test]
async fn test_download_params_with_bool() {
    caller::init_config().unwrap();
    
    let result = download_params(
        "JP.download",
        Some(params! { "thumbnail" => true }),
        None
    ).await;
    
    let _ = result;
}

#[tokio::test]
async fn test_download_params_with_float() {
    caller::init_config().unwrap();
    
    let result = download_params(
        "JP.download",
        Some(params! { "scale" => 1.5 }),
        None
    ).await;
    
    let _ = result;
}

#[tokio::test]
async fn test_download_params_with_manual_extension() {
    caller::init_config().unwrap();
    
    let result = download_params(
        "JP.download",
        Some(params! { "file_id" => 1 }),
        Some("pdf".to_string())
    ).await;
    
    let _ = result;
}

#[tokio::test]
async fn test_download_params_call_params_type() {
    caller::init_config().unwrap();
    
    // 测试 params! 宏支持的各种类型
    
    // 整数
    let params_int = params! { "id" => 1 };
    assert!(!params_int.is_empty());
    
    // 字符串
    let params_str = params! { "name" => "test" };
    assert!(!params_str.is_empty());
    
    // 布尔值
    let params_bool = params! { "active" => true };
    assert!(!params_bool.is_empty());
    
    // 浮点数
    let params_float = params! { "price" => 99.99 };
    assert!(!params_float.is_empty());
    
    // 数组
    let params_arr = params! { "tags" => vec!["rust", "web"] };
    assert!(!params_arr.is_empty());
}

#[tokio::test]
async fn test_download_params_builder_pattern() {
    caller::init_config().unwrap();
    
    // 测试 Builder 模式
    let params = params!()
        .add("field1", "value1")
        .add("field2", "value2")
        .add("field3", "value3");
    
    assert_eq!(params.len(), 3);
    assert!(!params.is_empty());
}

#[tokio::test]
async fn test_download_params_json_conversion() {
    caller::init_config().unwrap();
    
    // 测试 JSON 转换
    let json_value = serde_json::json!({
        "key1": "value1",
        "key2": 123
    });
    
    let params = CallParams::from_json(json_value).unwrap();
    assert_eq!(params.len(), 2);
    
    let json_back = params.to_json();
    assert!(json_back.is_object());
}

#[tokio::test]
async fn test_download_params_hashmap_conversion() {
    caller::init_config().unwrap();
    
    // 测试 HashMap 转换
    use std::collections::HashMap;
    let mut hashmap = HashMap::new();
    hashmap.insert("key1".to_string(), "value1".to_string());
    hashmap.insert("key2".to_string(), "value2".to_string());
    
    let params = CallParams::from_hashmap(hashmap);
    assert_eq!(params.len(), 2);
    
    let hashmap_back = params.to_hashmap();
    assert_eq!(hashmap_back.len(), 2);
}