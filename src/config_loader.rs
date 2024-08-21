use std::sync::OnceLock;

use crate::{
    models::{api_item::ApiItem, caller_config::CallerConfig, service_item::ServiceItem},
    utils::config_loader::load_config,
};

pub struct ConfigLoader {}

static CONFIG: OnceLock<CallerConfig> = OnceLock::new();

impl ConfigLoader {
    pub fn get_config(service_name: &str, api_item: &str) -> (ServiceItem, ApiItem) {
        let config = CONFIG.get_or_init(|| match load_config() {
            Ok(value) => value,
            Err(e) => panic!("caller配置文件加载失败: {}", e),
        });

        let service_item = config
            .service_items
            .iter()
            .find(|s| s.api_name == service_name)
            .unwrap();
        let api_item = service_item
            .api_items
            .iter()
            .find(|a| a.method == api_item)
            .unwrap();

        (service_item.clone(), api_item.clone())
    }

    pub fn get_config_ref() -> &'static CallerConfig {
        CONFIG.get().unwrap()
    }
}
