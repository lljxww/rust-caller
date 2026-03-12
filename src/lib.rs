// Core modules
pub mod config;
pub mod core;
pub mod domain;
pub mod infra;
pub mod shared;

// Re-export public APIs for convenience
pub use config::*;
pub use domain::*;

// Main public API
use std::collections::HashMap;
use std::time::Duration;

/// Main error type for the caller library
pub use shared::error::CallerError;

/// Main public API function
pub async fn call(
    method: &str,
    params: Option<HashMap<String, String>>,
) -> Result<domain::api_result::ApiResult, CallerError> {
    core::context::CallerContext::call(method, params).await
}

/// Download file from API endpoint
/// 
/// # Arguments
/// * `method` - API method in format "service.api"
/// * `params` - Optional parameters for the request
/// * `extension` - Optional file extension to override auto-detected extension
/// 
/// # Returns
/// Returns a DownloadResult containing the downloaded content and metadata
/// 
/// # Examples
/// ```no_run
/// use caller::download;
/// 
/// # #[tokio::main]
/// # async fn main() -> Result<(), caller::CallerError> {
/// // Download with auto-detected file format
/// let result = download("api.download", None, None).await?;
/// 
/// // Download with specified file extension
/// let result = download("api.download", None, Some("pdf".to_string())).await?;
/// 
/// // Save the downloaded file
/// result.save("./downloads", "myfile")?;
/// # Ok(())
/// # }
/// ```
pub async fn download(
    method: &str,
    params: Option<HashMap<String, String>>,
    extension: Option<String>,
) -> Result<domain::download_result::DownloadResult, CallerError> {
    core::context::CallerContext::download(method, params, extension).await
}

/// Initialize the configuration by loading from file
pub fn init_config() -> Result<(), CallerError> {
    config::config_loader::ConfigLoader::load_config()
        .map_err(|e| CallerError::ConfigError(e.to_string()))?;
    Ok(())
}

/// Manually reload the configuration from file
pub fn reload_config() -> Result<(), CallerError> {
    config::config_loader::ConfigLoader::reload_config()
}

/// Start watching the config file for changes.
/// Changes will be automatically reloaded with a default debounce of 500ms.
pub fn watch_config() -> Result<(), CallerError> {
    config::config_loader::ConfigLoader::start_watching(Duration::from_millis(500))
}

/// Start watching the config file with a custom debounce duration
pub fn watch_config_with_debounce(debounce: Duration) -> Result<(), CallerError> {
    config::config_loader::ConfigLoader::start_watching(debounce)
}

/// Stop watching the config file
pub fn stop_watch_config() {
    config::config_loader::ConfigLoader::stop_watching()
}

/// Check if the config is currently being watched
pub fn is_watching_config() -> bool {
    config::config_loader::ConfigLoader::is_watching()
}

/// Check if the configuration is loaded
pub fn is_config_loaded() -> bool {
    config::config_loader::ConfigLoader::is_config_loaded()
}
