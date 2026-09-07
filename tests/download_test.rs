use caller::{CallerError, DownloadResult, download};
use reqwest::StatusCode;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn download_result_detects_content_type_and_text() {
    let result = DownloadResult::from_response(
        StatusCode::OK,
        b"hello".to_vec(),
        Some("text/plain; charset=utf-8".to_string()),
    );

    assert_eq!(result.file_extension, "txt");
    assert_eq!(result.size(), 5);
    assert_eq!(result.as_text().unwrap(), "hello");
}

#[test]
fn safe_save_creates_a_new_file() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "caller-download-integration-{}-{nonce}",
        std::process::id()
    ));
    let result = DownloadResult::from_response(
        StatusCode::OK,
        b"content".to_vec(),
        Some("application/pdf".to_string()),
    );

    let filename = result.save(&directory, "report").unwrap();
    assert_eq!(filename, "report.pdf");
    assert_eq!(std::fs::read(directory.join(filename)).unwrap(), b"content");

    std::fs::remove_dir_all(directory).ok();
}

#[test]
fn safe_save_rejects_remote_path_traversal() {
    let result = DownloadResult::from_response(StatusCode::OK, vec![], None)
        .with_filename("../outside.bin".to_string());
    let error = result
        .save(std::env::temp_dir(), "unused")
        .expect_err("unsafe remote filename must fail");
    assert!(matches!(error, CallerError::UnsafeDownloadFilename { .. }));
}

#[tokio::test]
async fn download_rejects_invalid_method_before_network_io() {
    let error = download("invalid.service.method", None, None)
        .await
        .expect_err("invalid method shape must fail");
    assert!(matches!(error, CallerError::InvalidMethodFormat { .. }));
}
