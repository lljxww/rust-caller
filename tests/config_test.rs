#[path = "../src/models/mod.rs"]
mod models;

#[path = "../src/utils/mod.rs"]
mod utils;

#[path = "../src/lib.rs"]
mod lib;

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
        println!("{}", result.raw);

        assert_eq!("1", result.get("ok").unwrap_or("-1".to_string()));
    }

    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(test_caller_get_async())
}
