use crate::models::caller_config::CallerConfig;
use crate::utils::config_loader::load_config;

pub struct CallerContext {
    pub config: CallerConfig,
}

impl CallerContext {
    pub fn build() -> CallerContext {
        CallerContext {
            config: match load_config() {
                Ok(value) => value,
                Err(e) => panic!("{}", e),
            },
        }
    }
}
