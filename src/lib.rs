// Core modules
pub mod config;
pub mod core;
pub mod domain;
pub mod infra;
pub mod openapi;
pub mod params;
pub mod server;
pub mod shared;

// Re-export public APIs for convenience
pub use config::*;
pub use domain::*;
pub use params::*;

// Main public API
use std::collections::HashMap;
use std::time::Duration;

/// Main error type for the caller library
pub use shared::error::CallerError;

// Re-export authentication types
pub use domain::auth_registry::AuthRegistry;
pub use domain::auth_trait::{AuthContext, AuthProvider, Authenticator};
pub use domain::builtin_auth::{
    ApiKeyAuth, BasicAuth, BearerAuth, CustomHeaderAuth, DynamicApiKeyAuth, DynamicBearerAuth,
    DynamicHeaderAuth, NoAuth, OAuth2Auth,
};

// Re-export OpenAPI types
pub use openapi::{OpenApiDoc, OpenApiGenerator};
pub use server::ServerConfig;

// Re-export server types when server feature is enabled
#[cfg(feature = "server")]
pub use server::start_server;

/// Main public API function
/// 
/// 传统方式：接受 HashMap<String, String> 参数
/// 
/// # Arguments
/// * `method` - API method in format "service.api"
/// * `params` - Optional parameters (HashMap)
///
/// # Examples
/// ```no_run
/// # use caller::call;
/// # use std::collections::HashMap;
/// # #[tokio::main]
/// # async fn main() -> Result<(), caller::CallerError> {
/// let mut params = HashMap::new();
/// params.insert("id".to_string(), "1".to_string());
/// call("api.get", Some(params)).await?;
/// # Ok(())
/// # }
/// ```
/// 
/// 对于类型安全的参数，请使用 [`call_params`] 函数。
pub async fn call(
    method: &str,
    params: Option<HashMap<String, String>>,
) -> Result<domain::api_result::ApiResult, CallerError> {
    core::context::CallerContext::call(method, params).await
}

/// Main public API function with type-safe parameters
/// 
/// # Arguments
/// * `method` - API method in format "service.api"
/// * `params` - Optional type-safe parameters using [`params!`] macro
/// 
/// # Examples
/// ```no_run
/// # use caller::{call_params, params};
/// # #[tokio::main]
/// # async fn main() -> Result<(), caller::CallerError> {
/// // Using the params! macro
/// let result = call_params("api.get", Some(params! {
///     "id" => 1
/// })).await?;
/// 
/// // Using builder pattern
/// let params = params!()
///     .add("id", 1)
///     .add("name", "Alice")
///     .add("active", true);
/// let result = call_params("api.create", Some(params)).await?;
/// # Ok(())
/// # }
/// ```
pub async fn call_params(
    method: &str,
    params: Option<CallParams>,
) -> Result<domain::api_result::ApiResult, CallerError> {
    let hashmap = params.map(|p| p.to_hashmap());
    core::context::CallerContext::call(method, hashmap).await
}

/// Main public API function with retry support
/// 
/// 传统方式：接受 HashMap<String, String> 参数
/// 
/// # Arguments
/// * `method` - API method in format "service.api"
/// * `params` - Optional parameters (HashMap)
/// * `retry_config` - Retry configuration
///
/// # Examples
/// ```no_run
/// # use caller::{call_with_retry, RetryConfig};
/// # use std::collections::HashMap;
/// # #[tokio::main]
/// # async fn main() -> Result<(), caller::CallerError> {
/// let mut params = HashMap::new();
/// params.insert("id".to_string(), "1".to_string());
/// let retry_config = RetryConfig::new();
/// call_with_retry("api.get", Some(params), retry_config).await?;
/// # Ok(())
/// # }
/// ```
/// 
/// 对于类型安全的参数，请使用 [`call_params_with_retry`] 函数。
pub async fn call_with_retry(
    method: &str,
    params: Option<HashMap<String, String>>,
    retry_config: domain::retry_config::RetryConfig,
) -> Result<domain::api_result::ApiResult, CallerError> {
    core::context::CallerContext::call_with_retry(method, params, retry_config).await
}

/// Main public API function with type-safe parameters and retry support
/// 
/// # Arguments
/// * `method` - API method in format "service.api"
/// * `params` - Optional type-safe parameters using [`params!`] macro
/// * `retry_config` - Retry configuration
/// 
/// # Examples
/// ```no_run
/// # use caller::{call_params_with_retry, params, RetryConfig};
/// # use std::time::Duration;
/// # #[tokio::main]
/// # async fn main() -> Result<(), caller::CallerError> {
/// let retry_config = RetryConfig::new()
///     .with_max_retries(3)
///     .with_base_delay(Duration::from_millis(500));
/// 
/// let result = call_params_with_retry(
///     "api.get",
///     Some(params! { "id" => 1 }),
///     retry_config
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub async fn call_params_with_retry(
    method: &str,
    params: Option<CallParams>,
    retry_config: domain::retry_config::RetryConfig,
) -> Result<domain::api_result::ApiResult, CallerError> {
    let hashmap = params.map(|p| p.to_hashmap());
    core::context::CallerContext::call_with_retry(method, hashmap, retry_config).await
}

/// Download file from API endpoint
/// 
/// 传统方式：接受 HashMap<String, String> 参数
/// 
/// # Arguments
/// * `method` - API method in format "service.api"
/// * `params` - Optional parameters (HashMap)
/// * `extension` - Optional file extension to override auto-detected extension
/// 
/// # Returns
/// Returns a DownloadResult containing the downloaded content and metadata
/// 
/// # Examples
/// ```no_run
/// # use caller::download;
/// # use std::collections::HashMap;
/// # #[tokio::main]
/// # async fn main() -> Result<(), caller::CallerError> {
/// let mut params = HashMap::new();
/// params.insert("file_id".to_string(), "123".to_string());
/// let result = download("api.download", Some(params), None).await?;
/// # Ok(())
/// # }
/// ```
/// 
/// 对于类型安全的参数，请使用 [`download_params`] 函数。
pub async fn download(
    method: &str,
    params: Option<HashMap<String, String>>,
    extension: Option<String>,
) -> Result<domain::download_result::DownloadResult, CallerError> {
    core::context::CallerContext::download(method, params, extension).await
}

/// Download file from API endpoint with type-safe parameters
/// 
/// # Arguments
/// * `method` - API method in format "service.api"
/// * `params` - Optional type-safe parameters using [`params!`] macro
/// * `extension` - Optional file extension to override auto-detected extension
/// 
/// # Returns
/// Returns a DownloadResult containing the downloaded content and metadata
/// 
/// # Examples
/// ```no_run
/// # use caller::{download_params, params};
/// # #[tokio::main]
/// # async fn main() -> Result<(), caller::CallerError> {
/// let result = download_params(
///     "api.download",
///     Some(params! { "file_id" => 123 }),
///     Some("pdf".to_string())
/// ).await?;
/// 
/// result.save("./downloads", "myfile")?;
/// # Ok(())
/// # }
/// ```
pub async fn download_params(
    method: &str,
    params: Option<CallParams>,
    extension: Option<String>,
) -> Result<domain::download_result::DownloadResult, CallerError> {
    let hashmap = params.map(|p| p.to_hashmap());
    core::context::CallerContext::download(method, hashmap, extension).await
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

// ============================================================================
// Authentication API
// ============================================================================

/// Register an authenticator implementation
/// 
/// # Arguments
/// * `name` - Unique name for this authentication provider
/// * `auth` - The authenticator implementation
/// 
/// # Example
/// ```rust
/// use caller::{register_auth, BearerAuth};
/// 
/// // Register a Bearer token authenticator
/// let auth = BearerAuth::new("my-api-token".to_string());
/// register_auth("my_bearer", auth).unwrap();
/// ```
pub fn register_auth(name: &str, auth: impl Authenticator + 'static) -> Result<(), CallerError> {
    AuthRegistry::register(name, auth)
}

/// Register a closure-based authentication provider
/// 
/// This provides maximum flexibility for custom authentication scenarios
/// that don't fit the trait-based approach.
/// 
/// # Arguments
/// * `name` - Unique name for this authentication provider
/// * `f` - Async closure that modifies the request builder
/// 
/// # Example
/// ```rust
/// use caller::register_auth_closure;
/// use reqwest::RequestBuilder;
/// use caller::AuthContext;
/// 
/// // Register a custom authentication closure
/// register_auth_closure("custom_auth", |builder: RequestBuilder, ctx: &AuthContext| async move {
///     // Full access to request builder and context
///     Ok(builder
///         .header("X-Api-Key", "my-key")
///         .header("X-Request-Id", "12345"))
/// }).unwrap();
/// # caller::clear_auth().unwrap();
/// ```
pub fn register_auth_closure<F, Fut>(name: &str, f: F) -> Result<(), CallerError>
where
    F: Fn(reqwest::RequestBuilder, &AuthContext) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<reqwest::RequestBuilder, CallerError>> + Send + 'static,
{
    AuthRegistry::register_closure(name, f)
}

/// Check if an authentication provider is registered
/// 
/// # Arguments
/// * `name` - Name of the authentication provider to check
/// 
/// # Example
/// ```rust
/// use caller::{register_auth, has_auth, BearerAuth};
/// 
/// let auth = BearerAuth::new("token".to_string());
/// register_auth("my_auth", auth).unwrap();
/// 
/// assert!(has_auth("my_auth"));
/// ```
pub fn has_auth(name: &str) -> bool {
    AuthRegistry::contains(name)
}

/// Update an existing authenticator
/// 
/// This replaces the authenticator if it exists, or adds it if it doesn't.
/// Useful for token refresh scenarios.
/// 
/// # Arguments
/// * `name` - Name of the authentication provider
/// * `auth` - New authenticator implementation
/// 
/// # Example
/// ```rust
/// use caller::{register_auth, update_auth, BearerAuth};
/// 
/// // Initial registration
/// register_auth("github", BearerAuth::new("old-token".to_string())).unwrap();
/// 
/// // Later, update with new token
/// update_auth("github", BearerAuth::new("new-token".to_string())).unwrap();
/// ```
pub fn update_auth(name: &str, auth: impl Authenticator + 'static) -> Result<(), CallerError> {
    AuthRegistry::update(name, auth)
}

/// Update a closure-based authentication provider
/// 
/// # Arguments
/// * `name` - Name of the authentication provider
/// * `f` - Async closure that modifies the request builder
pub fn update_auth_closure<F, Fut>(name: &str, f: F) -> Result<(), CallerError>
where
    F: Fn(reqwest::RequestBuilder, &AuthContext) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<reqwest::RequestBuilder, CallerError>> + Send + 'static,
{
    AuthRegistry::update_closure(name, f)
}

/// Remove an authentication provider
/// 
/// # Arguments
/// * `name` - Name of the authentication provider to remove
/// 
/// # Returns
/// `true` if the provider was removed, `false` if it didn't exist
pub fn remove_auth(name: &str) -> Result<bool, CallerError> {
    AuthRegistry::remove(name)
}

/// Clear all registered authentication providers
pub fn clear_auth() -> Result<(), CallerError> {
    AuthRegistry::clear()
}

/// List all registered authentication provider names
pub fn list_auth() -> Result<Vec<String>, CallerError> {
    AuthRegistry::list()
}

/// Get the number of registered authentication providers
pub fn auth_count() -> Result<usize, CallerError> {
    AuthRegistry::count()
}
