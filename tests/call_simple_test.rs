use caller::call;
use std::collections::HashMap;

#[tokio::test]
async fn test_call_basic_get() {
    let result = call("JP.list", None).await;
    assert!(result.is_ok(), "Basic GET call should succeed");
    let api_result = result.unwrap();
    assert_eq!(api_result.status_code, 200);
    assert!(!api_result.raw.is_empty());
}

#[tokio::test]
async fn test_call_with_path_params() {
    let params = HashMap::from([("post_id".to_string(), "1".to_string())]);
    let result = call("JP.get", Some(params)).await;
    assert!(result.is_ok(), "Path params call should succeed");
    let api_result = result.unwrap();
    assert_eq!(api_result.status_code, 200);
    assert_eq!(api_result.get_as_i64("id"), Some(1));
}

#[tokio::test]
async fn test_call_create_post() {
    let params = HashMap::from([
        ("title".to_string(), "Test Post".to_string()),
        ("body".to_string(), "Test body".to_string()),
        ("userId".to_string(), "1".to_string()),
    ]);
    let result = call("JP.create", Some(params)).await;
    assert!(result.is_ok(), "Create call should succeed");
    let api_result = result.unwrap();
    assert_eq!(api_result.status_code, 201);
    assert_eq!(api_result.get_as_str("title"), Some("Test Post"));
}

#[tokio::test]
async fn test_call_with_query_params() {
    let params = HashMap::from([("userId".to_string(), "1".to_string())]);
    let result = call("JP.filter", Some(params)).await;
    assert!(result.is_ok(), "Query params call should succeed");
    let api_result = result.unwrap();
    assert_eq!(api_result.status_code, 200);
}
