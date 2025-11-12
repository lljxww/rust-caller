// Core modules
pub mod core;
pub mod config;
pub mod domain;
pub mod infra;
pub mod shared;

// Re-export public APIs for convenience
pub use core::*;
pub use domain::*;
pub use config::*;

// Main public API
use std::collections::HashMap;

/// Main error type for the caller library
pub use shared::error::CallerError;

/// Main public API trait
pub trait Callable<T> {
    fn call(method: &str, params: T) -> Result<domain::api_result::ApiResult, CallerError>;
}

/// Main public API function
pub async fn call(
    method: &str,
    params: Option<HashMap<String, String>>,
) -> Result<domain::api_result::ApiResult, CallerError> {
    core::context::CallerContext::call(method, params).await
}