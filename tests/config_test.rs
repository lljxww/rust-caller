#[path = "../src/models/mod.rs"]
mod models;

#[path = "../src/utils/mod.rs"]
mod utils;

#[path = "../src/lib.rs"]
mod lib;

use std::{collections::HashMap, fs, str::FromStr};

use caller::models::api_result::ApiResult;
use reqwest::StatusCode;
use tokio::runtime::Builder;
use utils::config_loader::load_config;

#[test]
fn test_load_config() {
    let config = match load_config() {
        Ok(value) => value,
        Err(err) => {
            eprintln!("{}", err);
            panic!("{}", err);
        }
    };

    println!("{}", config);
}

#[test]
fn test_caller_get() {
    async fn test_caller_get_async() {
        let result = lib::call("weibo.hot", None).await.unwrap();
        assert_eq!(1, result.get_as_i64("ok").unwrap());
        assert_eq!(StatusCode::OK, result.status_code);
    }

    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(test_caller_get_async())
}

#[test]
fn test_api_result_get_str() {
    let test_json_str = fs::read_to_string("api_result_test.json").unwrap();
    let result = ApiResult::build(test_json_str, StatusCode::OK);

    assert_eq!(1, result.get_as_i64("ok").unwrap());

    assert_eq!(
        "热",
        result
            .get("data")
            .unwrap()
            .get("hotgov")
            .unwrap()
            .get("small_icon_desc")
            .unwrap()
    );

    assert_eq!(
        "热",
        result.get_as_str("data.hotgov.small_icon_desc").unwrap()
    );

    assert_eq!(
        None,
        result.get_as_i64("data.hotgov.small_icon_desc_nodata")
    );
}

#[test]
fn test_jp_create() {
    async fn test_jp_create_async() {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert(String::from_str("k"), String::from_str("v"));
        let result = lib::call("JP.create", Some(params)).await.unwrap();
    }

    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(test_jp_create_async())
}
