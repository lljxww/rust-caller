use std::sync::OnceLock;
use std::{error::Error, fs};

use crate::{
    domain::api_item::ApiItem,
    domain::caller_config::CallerConfig,
    domain::service_item::ServiceItem,
};
use crate::shared::error::CallerError;

pub(crate) struct ConfigLoader {}

static CONFIG: OnceLock<CallerConfig> = OnceLock::new();

impl ConfigLoader {
    pub fn get_config(service_name: &str, api_item: &str) -> Result<(ServiceItem, ApiItem), CallerError> {
        let config = CONFIG.get_or_init(|| {
            match Self::load_config() {
                Ok(value) => value,
                Err(_) => panic!("caller配置文件加载失败，请检查caller.json文件是否存在且格式正确"),
            }
        });

        let service_item = config
            .service_items
            .iter()
            .find(|s| s.api_name == service_name)
            .ok_or_else(|| CallerError::service_not_found(service_name))?;

        let api_item = service_item
            .api_items
            .iter()
            .find(|a| a.method == api_item)
            .ok_or_else(|| CallerError::api_not_found(service_name, api_item))?;

        Ok((service_item.clone(), api_item.clone()))
    }

    pub fn get_config_ref() -> Result<&'static CallerConfig, CallerError> {
        CONFIG.get().ok_or(CallerError::ConfigNotInitialized)
    }

    pub fn is_config_loaded() -> bool {
        CONFIG.get().is_some()
    }

    pub fn force_reload() -> Result<(), CallerError> {
        // Since OnceLock is designed to be initialized only once,
        // we cannot forcefully reload it in this implementation.
        // For real-world usage, consider using a different synchronization primitive
        // or implementing a proper config reloading mechanism.
        eprintln!("Warning: Force reload is not supported with OnceLock. Consider using a different approach for config reloading.");
        Ok(())
    }

    pub(crate) fn load_config() -> Result<CallerConfig, Box<dyn Error>> {
        let config = fs::read_to_string("./caller.json")?;
        let caller_config = serde_json::from_str(&config)?;
        Ok(caller_config)
    }

    pub fn try_get_config(service_name: &str, api_item: &str) -> Result<(ServiceItem, ApiItem), CallerError> {
        Self::get_config(service_name, api_item)
    }
}
