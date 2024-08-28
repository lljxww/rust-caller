pub mod caller_const;
pub mod caller_context;
pub mod config_loader;
pub mod models;
pub mod utils;

use crate::models::api_result::ApiResult;
use caller_context::CallerContext;
use std::collections::HashMap;

pub trait Callable<T> {
    fn call(method: &str, params: T) -> Result<ApiResult, reqwest::Error>;
}

pub async fn call(
    method: &str,
    params: Option<HashMap<String, String>>,
) -> Result<ApiResult, reqwest::Error> {
    CallerContext::call(method, params).await
}
0