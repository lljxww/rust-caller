use caller::call;
use std::collections::HashMap;

#[tokio::test]
#[ignore = "requires external network access"]
async fn test_call_list_posts() {
    // Test: Get list of all posts
    let result = call("JP.list", None).await;
    println!("List posts result: {:?}", result);
    assert!(result.is_ok(), "List call should succeed");
    let api_result = result.unwrap();

    assert_eq!(api_result.status_code, 200);
    assert!(!api_result.raw.is_empty());

    // Debug the raw response
    println!(
        "Raw response (first 200 chars): {}",
        &api_result.raw[..200.min(api_result.raw.len())]
    );

    // Access the first post directly
    let first_post_id = api_result.i64_at("0.id");
    let first_post_title = api_result.str_at("0.title");

    assert!(
        first_post_id.is_some() && first_post_id.unwrap() > 0,
        "First post should have a valid ID"
    );
    assert!(first_post_title.is_some(), "First post should have a title");
}

#[tokio::test]
#[ignore = "requires external network access"]
async fn test_call_get_single_post() {
    // Test: Get single post by ID
    let params = HashMap::from([("post_id".to_string(), "1".to_string())]);
    let result = call("JP.get", Some(params)).await;
    println!("Get post result: {:?}", result);
    assert!(result.is_ok(), "Get post call should succeed");
    let api_result = result.unwrap();

    assert_eq!(api_result.status_code, 200);

    // Check the raw response
    println!("Raw response: {}", &api_result.raw[..100]);
    if api_result.raw.len() > 100 {
        println!("...");
    }

    assert_eq!(api_result.i64_at("id"), Some(1));
    assert!(api_result.str_at("title").is_some());
    assert!(api_result.str_at("body").is_some());
}

#[tokio::test]
#[ignore = "requires external network access"]
async fn test_call_create_post() {
    // Test: Create a new post
    let params = HashMap::from([
        ("title".to_string(), "Test Integration Post".to_string()),
        (
            "body".to_string(),
            "This is a test post created through integration".to_string(),
        ),
        ("userId".to_string(), "1".to_string()),
    ]);

    let result = call("JP.create", Some(params)).await;
    assert!(result.is_ok(), "Create post call should succeed");
    let api_result = result.unwrap();

    assert_eq!(api_result.status_code, 201); // Created
    assert_eq!(api_result.str_at("title"), Some("Test Integration Post"));
    assert_eq!(
        api_result.str_at("body"),
        Some("This is a test post created through integration")
    );

    // Get the created ID
    let created_id = api_result.i64_at("id");
    assert!(created_id.is_some());

    println!("Created post with ID: {:?}", created_id);
}

#[tokio::test]
#[ignore = "requires external network access"]
async fn test_call_filter_posts() {
    // Test: Filter posts by user ID
    let params = HashMap::from([("userId".to_string(), "1".to_string())]);
    let result = call("JP.filter", Some(params)).await;
    assert!(result.is_ok(), "Filter posts call should succeed");
    let api_result = result.unwrap();

    assert_eq!(api_result.status_code, 200);
    assert!(!api_result.raw.is_empty());

    // Check that we got an array response
    let first_result = api_result.value_at("0");
    assert!(
        first_result.is_some(),
        "Filter should return at least one result"
    );

    if let Some(first_item) = first_result.as_ref() {
        let user_id = first_item.get("userId").and_then(|v| v.as_i64());
        assert_eq!(user_id, Some(1));
    }
}

#[tokio::test]
#[ignore = "requires external network access"]
async fn test_call_with_query_params() {
    // Test: Multiple query parameters
    let params = HashMap::from([
        ("userId".to_string(), "1".to_string()),
        ("id".to_string(), "1".to_string()),
    ]);

    let result = call("JP.filter", Some(params)).await;
    assert!(
        result.is_ok(),
        "Call with multiple query params should succeed"
    );
    let api_result = result.unwrap();

    assert_eq!(api_result.status_code, 200);
    assert!(!api_result.raw.is_empty());
}

#[tokio::test]
async fn test_call_invalid_method() {
    // Test: Non-existent method
    let result = call("JP.nonexistent", None).await;
    assert!(result.is_err(), "Should fail for non-existent method");
    println!("Non-existent method error: {:?}", result.unwrap_err());
}

#[tokio::test]
async fn test_call_invalid_service() {
    // Test: Non-existent service
    let result = call("invalidService.method", None).await;
    assert!(result.is_err(), "Should fail for non-existent service");
    println!("Non-existent service error: {:?}", result.unwrap_err());
}

#[tokio::test]
async fn test_call_wrong_path_params() {
    // Test: Missing path parameters
    let result = call("JP.get", None).await; // Missing post_id
    assert!(result.is_err(), "Should fail for missing path parameters");
    println!("Missing path params error: {:?}", result.unwrap_err());
}
