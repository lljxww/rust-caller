//! 测试组合参数类型（如 path,json）

use caller::call;
use std::collections::HashMap;

#[tokio::test]
async fn test_update_with_combined_params() {
    // 测试：使用组合参数类型 (path,json) 更新文章
    let params = HashMap::from([
        ("post_id".to_string(), "1".to_string()),
        ("title".to_string(), "Updated Title".to_string()),
        ("body".to_string(), "Updated body content".to_string()),
        ("userId".to_string(), "1".to_string()),
    ]);

    let result = call("JP.update", Some(params)).await;
    
    if let Err(e) = &result {
        println!("Update call failed: {:?}", e);
    }
    
    // JSONPlaceholder 的 PUT 请求会返回更新后的数据
    assert!(result.is_ok(), "Update call should succeed");
    let api_result = result.unwrap();
    
    // PUT 请求通常返回 200 状态码
    assert_eq!(api_result.status_code, 200);
    
    // 验证更新后的数据
    assert_eq!(
        api_result.get_as_str("title"),
        Some("Updated Title")
    );
    assert_eq!(
        api_result.get_as_str("body"),
        Some("Updated body content")
    );
    assert_eq!(api_result.get_as_i64("id"), Some(1));
}

#[tokio::test]
async fn test_patch_with_combined_params() {
    // 测试：使用组合参数类型 (path,json) 部分更新文章
    let params = HashMap::from([
        ("post_id".to_string(), "1".to_string()),
        ("title".to_string(), "Patched Title".to_string()),
    ]);

    let result = call("JP.patch", Some(params)).await;
    
    // JSONPlaceholder 的 PATCH 请求会返回更新后的数据
    assert!(result.is_ok(), "Patch call should succeed");
    let api_result = result.unwrap();
    
    // PATCH 请求通常返回 200 状态码
    assert_eq!(api_result.status_code, 200);
    
    // 验证部分更新的数据
    assert_eq!(
        api_result.get_as_str("title"),
        Some("Patched Title")
    );
    assert_eq!(api_result.get_as_i64("id"), Some(1));
}

#[tokio::test]
async fn test_delete_with_path_params() {
    // 测试：使用路径参数删除文章
    let params = HashMap::from([
        ("post_id".to_string(), "1".to_string()),
    ]);

    let result = call("JP.delete", Some(params)).await;
    
    // JSONPlaceholder 的 DELETE 请求返回 200 状态码和一个空对象 {}
    assert!(result.is_ok(), "Delete call should succeed");
    let api_result = result.unwrap();
    
    // DELETE 请求返回 200 状态码
    assert_eq!(api_result.status_code, 200);
}

#[tokio::test]
async fn test_combined_params_missing_path() {
    // 测试：组合参数类型缺少路径参数应该失败
    let params = HashMap::from([
        ("title".to_string(), "Test".to_string()),
    ]);

    let result = call("JP.update", Some(params)).await;
    
    // 应该失败，因为缺少必需的路径参数 post_id
    assert!(result.is_err(), "Should fail when path parameters are missing");
    
    match result.unwrap_err() {
        caller::CallerError::UrlParameterNotFound(param) => {
            assert_eq!(param, "post_id", 
                    "Error should mention missing post_id parameter");
        }
        _ => panic!("Expected UrlParameterNotFound for missing path parameters"),
    }
}
