use std::{error::Error, fs};

use crate::models::caller_config::CallerConfig;

pub fn load_config() -> Result<CallerConfig, Box<dyn Error>> {
    let config = fs::read_to_string("./caller.json")?;
    let caller_config = serde_json::from_str(&config)?;
    Ok(caller_config)
}
