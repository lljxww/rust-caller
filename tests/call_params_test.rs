//! 测试 call_params 类型安全 API
//! 
//! 这个测试文件专门测试类型安全参数 API

use caller::{call_params, params, CallParams};

#[tokio::test]
async fn test_call_params_with_int() {
    caller::init_config().unwrap();
    
    let result = call_params("JP.list", None).await;
    assert!(result.is_ok());
    
    let result = call_params("JP.get", Some(params! {
        "post_id" => 1
    })).await;
    assert!(result.is_ok());
    
    if let Ok(res) = result {
        let id = res.get_as_i64("id").unwrap_or(0);
        assert!(id > 0);
    }
}

#[tokio::test]
async fn test_call_params_with_string() {
    caller::init_config().unwrap();
    
    let result = call_params("JP.get", Some(params! {
        "post_id" => "1"
    })).await;
    assert!(result.is_ok());
    
    if let Ok(res) = result {
        let id = res.get_as_i64("id").unwrap_or(0);
        assert!(id > 0);
    }
}

#[tokio::test]
async fn test_call_params_with_bool() {
    caller::init_config().unwrap();
    
    let result = call_params("JP.filter", Some(params! {
        "active" => true
    })).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_call_params_with_float() {
    caller::init_config().unwrap();
    
    let result = call_params("JP.filter", Some(params! {
        "price" => 99.99
    })).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_call_params_builder_pattern() {
    caller::init_config().unwrap();
    
    let params = params!()
        .add("userId", "1")
        .add("active", "true")
        .add("name", "Alice");
    
    let result = call_params("JP.filter", Some(params)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_call_params_with_array() {
    caller::init_config().unwrap();
    
    let result = call_params("JP.filter", Some(params! {
        "tags" => vec!["rust", "web"]
    })).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_call_params_with_json() {
    caller::init_config().unwrap();
    
    let json_value = serde_json::json!({
        "title": "Test Post",
        "body": "Content",
        "userId": 1
    });
    
    let params = CallParams::from_json(json_value).unwrap();
    let result = call_params("JP.create", Some(params)).await;
    
    assert!(result.is_ok());
    
    if let Ok(res) = result {
        let id = res.get_as_i64("id").unwrap_or(0);
        assert!(id > 0);
    }
}

#[tokio::test]
async fn test_call_params_empty() {
    caller::init_config().unwrap();
    
    let result = call_params("JP.list", None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_call_params_invalid_method() {
    caller::init_config().unwrap();
    
    let result = call_params("invalid.method", None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_call_params_missing_required_param() {
    caller::init_config().unwrap();
    
    let result = call_params("JP.get", None).await;
    // 应该返回错误，因为缺少必需的 post_id 参数
    assert!(result.is_err());
}

#[tokio::test]
async fn test_call_params_chain_builder() {
    caller::init_config().unwrap();
    
    let params = params!()
        .add("field1", "value1")
        .add("field2", "value2")
        .add("field3", "value3");
    
    assert!(!params.is_empty());
    assert_eq!(params.len(), 3);
}

#[tokio::test]
async fn test_call_params_to_json() {
    caller::init_config().unwrap();
    
    // 测试整数
    let params_int = params! { "id" => 1 };
    let json = params_int.to_json();
    assert!(json.is_object());
    assert!(json.get("id").is_some());
    
    // 测试字符串
    let params_str = params! { "name" => "Test" };
    let json = params_str.to_json();
    assert!(json.is_object());
    assert!(json.get("name").is_some());
}

#[tokio::test]
async fn test_call_params_to_hashmap() {
    caller::init_config().unwrap();
    
    let params1 = params! { "key1" => "value1" };
    let params2 = params! { "key2" => "value2" };
    
    let hashmap1 = params1.to_hashmap();
    assert_eq!(hashmap1.len(), 1);
    assert_eq!(hashmap1.get("key1"), Some(&"value1".to_string()));
    
    let hashmap2 = params2.to_hashmap();
    assert_eq!(hashmap2.len(), 1);
    assert_eq!(hashmap2.get("key2"), Some(&"value2".to_string()));
}

#[tokio::test]
async fn test_call_params_from_hashmap() {
    caller::init_config().unwrap();
    
    use std::collections::HashMap;
    let mut hashmap = HashMap::new();
    hashmap.insert("key1".to_string(), "value1".to_string());
    hashmap.insert("key2".to_string(), "value2".to_string());
    
    let params = CallParams::from_hashmap(hashmap);
    assert_eq!(params.len(), 2);
}