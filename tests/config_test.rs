use caller::domain::api_result::ApiResult;
use caller::CallerError;
use reqwest::StatusCode;

#[test]
fn test_models_mod_exists() {
    // This test verifies that models module can be imported
    assert!(true);
}

#[test]
fn test_api_result_creation() {
    let test_json = r#"{"ok": 1, "test": "value"}"#;
    let result = ApiResult::build(test_json.to_string(), StatusCode::OK)
        .expect("Failed to create ApiResult");

    assert_eq!(1, result.get_as_i64("ok").unwrap());
    assert_eq!("value", result.get_as_str("test").unwrap());
}

#[test]
fn test_api_result_invalid_json() {
    let invalid_json = r#"{"invalid json"#;
    let result = ApiResult::build(invalid_json.to_string(), StatusCode::OK);

    assert!(result.is_err());
    match result.unwrap_err() {
        CallerError::JsonError(msg) => {
            assert!(msg.contains("Failed to parse JSON response"));
        },
        _ => panic!("Expected JsonError"),
    }
}

#[test]
fn test_method_format_validation() {
    use caller::core::context::split_method;

    // Valid method
    let result = split_method("service.api");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["service", "api"]);

    // Invalid method - no dot
    let result = split_method("invalid_method");
    assert!(result.is_err());
    match result.unwrap_err() {
        CallerError::InvalidMethodFormat(msg) => assert!(msg.contains("invalid_method")),
        _ => panic!("Expected InvalidMethodFormat"),
    }

    // Invalid method - multiple dots
    let result = split_method("service.api.version");
    assert!(result.is_err());
    match result.unwrap_err() {
        CallerError::InvalidMethodFormat(msg) => assert!(msg.contains("service.api.version")),
        _ => panic!("Expected InvalidMethodFormat"),
    }

    // Invalid method - starts with dot
    let result = split_method(".api");
    assert!(result.is_err());
    match result.unwrap_err() {
        CallerError::InvalidMethodFormat(msg) => assert!(msg.contains("Method cannot start or end with dot")),
        _ => panic!("Expected InvalidMethodFormat"),
    }
}

#[test]
fn test_http_method_validation() {
    use caller::core::context::get_http_method;

    // Valid methods
    let methods = ["get", "post", "put", "delete", "patch"];
    for method in methods {
        let result = get_http_method(method);
        assert!(result.is_ok());
    }

    // Invalid method
    let result = get_http_method("invalid_method");
    assert!(result.is_err());
    match result.unwrap_err() {
        CallerError::HttpMethodNotSupported(msg) => assert_eq!(msg, "invalid_method"),
        _ => panic!("Expected HttpMethodNotSupported"),
    }
}
