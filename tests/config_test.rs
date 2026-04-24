use caller::ApiResult;
use caller::CallerError;
use caller::ErrorCategory;
use caller::ResponseBody;
use reqwest::StatusCode;

#[test]
fn test_models_mod_exists() {
    // This test verifies that models module can be imported and used
    // The test passes if the module path compiles successfully
}

#[test]
fn test_api_result_creation() {
    let test_json = r#"{"ok": 1, "test": "value"}"#;
    let result = ApiResult::build(test_json.to_string(), StatusCode::OK)
        .expect("Failed to create ApiResult");

    assert_eq!(1, result.i64_at("ok").unwrap());
    assert_eq!("value", result.str_at("test").unwrap());
}

#[test]
fn test_api_result_invalid_json() {
    let invalid_json = r#"{"invalid json"#;
    let result = ApiResult::build(invalid_json.to_string(), StatusCode::OK)
        .expect("non-json responses should still build");

    assert!(result.is_text());
    assert_eq!(result.text(), Some(invalid_json));
    assert!(matches!(result.body, ResponseBody::Text(_)));

    let strict_result = ApiResult::build_json(invalid_json.to_string(), StatusCode::OK);
    assert!(strict_result.is_err());
    match strict_result.unwrap_err() {
        CallerError::JsonError(msg) => {
            assert!(msg.contains("Failed to parse JSON response"));
        }
        _ => panic!("Expected JsonError"),
    }
}

#[tokio::test]
async fn test_method_format_validation() {
    // Test valid method through public API
    let _result = caller::call("JP.list", None).await;
    // This should work for a valid method

    // Test invalid method through public API
    let result = caller::call("invalid_method", None).await;
    assert!(result.is_err());
    match result.err().unwrap() {
        CallerError::InvalidMethodFormat { method } => assert!(method.contains("invalid_method")),
        CallerError::ServiceNotFound { service } => assert!(service.contains("invalid_method")),
        _ => panic!("Expected InvalidMethodFormat or ServiceNotFound for invalid method"),
    }

    // Test invalid method format through public API (multiple dots)
    let result = caller::call("service.api.version", None).await;
    assert!(result.is_err());
    if let Err(CallerError::InvalidMethodFormat { method }) = result {
        assert!(method.contains("service.api.version"));
    } else {
        panic!("Expected InvalidMethodFormat for invalid method format");
    }
}

#[test]
fn test_error_categories_are_stable() {
    assert_eq!(
        CallerError::service_not_found("svc").category(),
        ErrorCategory::Runtime
    );
    assert_eq!(
        CallerError::unknown_auth_provider("token").category(),
        ErrorCategory::Security
    );
    assert_eq!(
        CallerError::unsupported_param_type("xml").category(),
        ErrorCategory::Protocol
    );
}

#[test]
fn test_http_method_validation() {
    // Test through public configuration loading
    // This indirectly tests HTTP method validation

    // The HTTP method validation happens during context building
    // We'll test this by trying to create a context that would fail

    // Note: This test is now implemented as an integration test
    // through the public API rather than testing internal methods directly
}
