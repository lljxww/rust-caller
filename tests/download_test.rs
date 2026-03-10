use caller::download;
use caller::init_config;

#[tokio::test]
async fn test_download_with_auto_detection() {
    // Initialize config if needed
    let _ = init_config();
    
    // This test demonstrates the download API
    // In a real scenario, you would configure an API endpoint that returns downloadable content
    
    // Example: Download with auto-detected file format
    // let result = download("api.download", None, None).await;
    // assert!(result.is_ok(), "Download should succeed");
    // let download_result = result.unwrap();
    
    // Verify download result properties
    // assert_eq!(download_result.status_code, 200);
    // assert!(download_result.size() > 0);
    // assert!(!download_result.file_extension.is_empty());
    
    println!("Download with auto-detection test - configure API endpoint to enable");
}

#[tokio::test]
async fn test_download_with_manual_extension() {
    // This test demonstrates overriding the auto-detected extension
    // let result = download("api.download", None, Some("custom".to_string())).await;
    // assert!(result.is_ok(), "Download should succeed");
    // let download_result = result.unwrap();
    
    // Verify manual extension override
    // assert_eq!(download_result.file_extension, "custom");
    
    println!("Download with manual extension test - configure API endpoint to enable");
}

#[tokio::test]
async fn test_download_with_params() {
    // This test demonstrates downloading with parameters
    // let params = HashMap::from([
    //     ("file_id".to_string(), "12345".to_string()),
    //     ("version".to_string(), "1.0".to_string()),
    // ]);
    // 
    // let result = download("api.download_file", Some(params), None).await;
    // assert!(result.is_ok(), "Download with params should succeed");
    
    println!("Download with parameters test - configure API endpoint to enable");
}

#[tokio::test]
async fn test_download_save_file() {
    // This test demonstrates saving downloaded content to a file
    // let result = download("api.download", None, None).await;
    // assert!(result.is_ok());
    // let download_result = result.unwrap();
    
    // Save to specific directory with auto-generated filename
    // let filename = download_result.save("./test_downloads", "testfile").unwrap();
    // assert!(!filename.is_empty());
    // 
    // Verify file was created
    // let file_path = std::path::Path::new("./test_downloads").join(&filename);
    // assert!(file_path.exists());
    // 
    // Cleanup
    // std::fs::remove_dir_all("./test_downloads").ok();
    
    println!("Download save file test - configure API endpoint to enable");
}

#[tokio::test]
async fn test_download_size_human() {
    // This test demonstrates human-readable file size formatting
    // let result = download("api.download", None, None).await;
    // assert!(result.is_ok());
    // let download_result = result.unwrap();
    // 
    // let size_human = download_result.size_human();
    // assert!(!size_human.is_empty());
    // println!("Downloaded file size: {}", size_human);
    
    println!("Download size human test - configure API endpoint to enable");
}

#[tokio::test]
async fn test_download_content_type_detection() {
    // This test demonstrates content type detection
    // let result = download("api.download_json", None, None).await;
    // assert!(result.is_ok());
    // let download_result = result.unwrap();
    // 
    // Should detect JSON content type
    // assert_eq!(download_result.content_type, "application/json");
    // assert_eq!(download_result.file_extension, "json");
    
    println!("Download content type detection test - configure API endpoint to enable");
}

#[tokio::test]
async fn test_download_invalid_method() {
    // Test: Non-existent method
    let result = download("invalid.service.method", None, None).await;
    assert!(result.is_err(), "Should fail for non-existent method");
    println!("Invalid method error: {:?}", result.unwrap_err());
}

#[tokio::test]
async fn test_download_result_methods() {
    // This test demonstrates DownloadResult helper methods
    // let result = download("api.download_text", None, None).await;
    // assert!(result.is_ok());
    // let download_result = result.unwrap();
    // 
    // Get file size in bytes
    // let size = download_result.size();
    // assert!(size > 0);
    // 
    // Get human-readable size
    // let size_human = download_result.size_human();
    // assert!(!size_human.is_empty());
    // 
    // Try to parse as text
    // if download_result.content_type.starts_with("text/") {
    //     let text = download_result.as_text().unwrap();
    //     assert!(!text.is_empty());
    // }
    
    println!("Download result methods test - configure API endpoint to enable");
}