use caller::{CallParams, CallerError, download_params, params};

#[tokio::test]
async fn typed_download_rejects_unknown_endpoint() {
    caller::init_config().unwrap();
    let error = download_params("JP.download", Some(params! { "file_id" => 1 }), None)
        .await
        .expect_err("unknown endpoint must fail before network I/O");
    assert!(matches!(error, CallerError::ApiNotFound { .. }));
}

#[test]
fn typed_download_params_preserve_json_types() {
    let params = params! {
        "id" => u64::MAX,
        "active" => true,
        "tags" => vec!["rust", "http"],
    };
    let json = params.to_json().unwrap();

    assert_eq!(json["id"], serde_json::json!(u64::MAX));
    assert_eq!(json["active"], true);
    assert_eq!(json["tags"], serde_json::json!(["rust", "http"]));
}

#[test]
fn json_params_require_an_object() {
    let error = CallParams::from_json(serde_json::json!([1, 2]))
        .expect_err("top-level CallParams must be an object");
    assert!(matches!(error, CallerError::ParameterError(_)));
}
