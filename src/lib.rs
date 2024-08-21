use std::collections::HashMap;

use caller_context::CallerContext;

pub mod caller_const;
pub mod caller_context;
pub mod config_loader;
pub mod http_method;
pub mod models;
pub mod utils;

pub async fn get(
    method: &str,
    params: Option<HashMap<String, String>>,
) -> Result<String, reqwest::Error> {
    CallerContext::call(method, params).await
}
