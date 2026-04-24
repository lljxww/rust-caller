//! `caller` is a configurable Web API calling crate with both instance-based
//! and global entry points.
//!
//! # Choosing An API Style
//!
//! For new code, prefer the instance API built around [`Caller`].
//!
//! The instance API is the better default because:
//!
//! - config is scoped to one concrete caller instance
//! - auth providers are scoped to that instance
//! - the underlying `reqwest::Client` is reused
//! - reload behavior is explicit and instance-local
//!
//! ```no_run
//! use caller::Caller;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), caller::CallerError> {
//! let caller = Caller::from_path("caller.json")?;
//! let result = caller.call("JP.list", None).await?;
//! println!("{}", result.status_code);
//! # Ok(())
//! # }
//! ```
//!
//! The crate-root global API such as [`call`] is still useful for simple apps,
//! quick scripts, and cases where one shared global config is enough.
//!
//! ```no_run
//! use caller::{call, init_config};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), caller::CallerError> {
//! init_config()?;
//! let result = call("JP.list", None).await?;
//! println!("{}", result.status_code);
//! # Ok(())
//! # }
//! ```
//!
//! # Global Config Limitation
//!
//! The global helpers currently assume a default config path of `./caller.json`.
//! That applies to:
//!
//! - [`init_config`]
//! - [`reload_config`]
//! - [`watch_config`]
//! - [`watch_config_with_debounce`]
//!
//! If you want to load JSON/YAML/TOML from arbitrary paths, prefer:
//!
//! - [`Caller::from_path`]
//! - [`ConfigLoader::load_config_from_path`]
//! - [`ConfigLoader::load_config_from_path_with_format`]
//!
//! # Programmatic Config
//!
//! For in-memory config construction, use [`ConfigBuilder`] together with
//! typed helpers such as [`HttpMethod`] and [`ParamType`].
//!
pub mod client;
// Public modules retained for compatibility
pub mod config;
pub(crate) mod core;
pub mod domain;
pub(crate) mod infra;
pub mod openapi;
pub mod params;
pub mod server;
mod shared;

// Stable crate-root API
pub use client::{Caller, CallerBuilder};
pub use config::config_builder::ConfigFormat as BuilderConfigFormat;
pub use config::config_loader::ConfigFormat as ConfigFileFormat;
pub use config::{ApiEndpointBuilder, ConfigBuilder, ConfigLoader, ServiceBuilder, run_config_cli};
pub use domain::auth_registry::AuthRegistry;
pub use domain::auth_trait::{AuthContext, AuthProvider, Authenticator};
pub use domain::builtin_auth::{
    ApiKeyAuth, BasicAuth, BearerAuth, CustomHeaderAuth, DynamicApiKeyAuth, DynamicBearerAuth,
    DynamicHeaderAuth, NoAuth, OAuth2Auth,
};
pub use domain::{
    ApiConfig, ApiResult, AuthConfig, CallerConfig, DownloadResult, HttpMethod, ParamType,
    ResponseBody, RetryConfig, ServiceConfig,
};
pub use domain::{
    CircuitBreakerConfig, CircuitBreakerMiddleware, CircuitBreakerStats, HeaderMiddleware,
    LoggingMiddleware, Middleware, MiddlewareChain, RequestContext, ResponseContext,
    RetryMiddleware, TimingMiddleware, UserAgentMiddleware,
};
pub use openapi::{OpenApiDoc, OpenApiGenerator};
pub use params::*;
pub use server::ServerConfig;
pub use shared::error::{CallerError, ErrorCategory};

#[cfg(feature = "server")]
pub use server::{start_server, start_server_with_caller};

// Main public API
use std::collections::HashMap;
use std::time::Duration;

/// Call an API through the global configuration state.
///
/// This is the simplest entry point, but it rebuilds a temporary [`Caller`]
/// from the current global config snapshot on each invocation.
///
/// Use [`Caller`] directly when you need instance-local auth providers,
/// instance-local config paths, or a long-lived reusable client.
///
/// # Panics
/// Does not panic by design; initialization and lookup failures are returned as
/// [`CallerError`].
///
/// # Examples
/// ```no_run
/// use caller::{call, init_config};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), caller::CallerError> {
/// init_config()?;
/// let result = call("JP.list", None).await?;
/// println!("{}", result.status_code);
/// # Ok(())
/// # }
/// ```
pub async fn call(
    method: &str,
    params: Option<HashMap<String, String>>,
) -> Result<domain::api_result::ApiResult, CallerError> {
    global_caller()?.call(method, params).await
}

/// Call an API through the global configuration state with type-safe parameters.
pub async fn call_params(
    method: &str,
    params: Option<CallParams>,
) -> Result<domain::api_result::ApiResult, CallerError> {
    call(method, params.map(|p| p.to_hashmap())).await
}

/// Main public API function with retry support
///
/// # Arguments
/// * `method` - API method in format "service.api"
/// * `params` - Optional parameters for the request
/// * `retry_config` - Retry configuration
///
/// # Examples
/// ```no_run
/// use caller::{call_with_retry, RetryConfig};
/// use std::collections::HashMap;
/// use std::time::Duration;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), caller::CallerError> {
/// let retry_config = RetryConfig::new()
///     .with_max_retries(3)
///     .with_base_delay(Duration::from_millis(500));
///
/// let result = call_with_retry("jsonplaceholder.posts.list", None, retry_config).await?;
/// # Ok(())
/// # }
/// ```
pub async fn call_with_retry(
    method: &str,
    params: Option<HashMap<String, String>>,
    retry_config: domain::retry_config::RetryConfig,
) -> Result<domain::api_result::ApiResult, CallerError> {
    global_caller()?
        .call_with_retry(method, params, retry_config)
        .await
}

/// Call an API through the global configuration state with type-safe parameters and retry support.
pub async fn call_params_with_retry(
    method: &str,
    params: Option<CallParams>,
    retry_config: domain::retry_config::RetryConfig,
) -> Result<domain::api_result::ApiResult, CallerError> {
    call_with_retry(method, params.map(|p| p.to_hashmap()), retry_config).await
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
    global_caller()?.download(method, params, extension).await
}

/// Download a file through the global configuration state with type-safe parameters.
pub async fn download_params(
    method: &str,
    params: Option<CallParams>,
    extension: Option<String>,
) -> Result<domain::download_result::DownloadResult, CallerError> {
    download(method, params.map(|p| p.to_hashmap()), extension).await
}

/// Initialize the global configuration state from the default config path.
///
/// Current default path is `./caller.json`.
/// For YAML/TOML or arbitrary paths, use [`ConfigLoader`] or [`Caller::from_path`].
pub fn init_config() -> Result<(), CallerError> {
    config::config_loader::ConfigLoader::reload_config()
}

/// Reload the global configuration state from the default config path.
pub fn reload_config() -> Result<(), CallerError> {
    config::config_loader::ConfigLoader::reload_config()
}

/// Start watching the default global config file for changes.
///
/// Changes are automatically reloaded with a default debounce of 500ms.
/// This watch path currently follows the same default path behavior as
/// [`init_config`], which means `./caller.json`.
pub fn watch_config() -> Result<(), CallerError> {
    config::config_loader::ConfigLoader::start_watching(Duration::from_millis(500))
}

/// Start watching the default global config file with a custom debounce duration.
pub fn watch_config_with_debounce(debounce: Duration) -> Result<(), CallerError> {
    config::config_loader::ConfigLoader::start_watching(debounce)
}

/// Stop watching the default global config file.
pub fn stop_watch_config() {
    config::config_loader::ConfigLoader::stop_watching()
}

/// Check whether the default global config file is currently being watched.
pub fn is_watching_config() -> bool {
    config::config_loader::ConfigLoader::is_watching()
}

/// Check whether the global configuration state has been loaded.
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
    Fut:
        std::future::Future<Output = Result<reqwest::RequestBuilder, CallerError>> + Send + 'static,
{
    AuthRegistry::register_closure(name, f)
}

/// Check if an authentication provider is registered
///
/// # Arguments
/// * `name` - name of the authentication provider to check
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
/// * `name` - name of the authentication provider
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
/// * `name` - name of the authentication provider
/// * `f` - Async closure that modifies the request builder
///
/// # Example
/// ```rust
/// use caller::update_auth_closure;
///
/// update_auth_closure("custom", |builder, _ctx| async move {
///     Ok(builder.header("X-Auth-Version", "v2"))
/// }).unwrap();
/// # caller::remove_auth("custom").unwrap();
/// ```
pub fn update_auth_closure<F, Fut>(name: &str, f: F) -> Result<(), CallerError>
where
    F: Fn(reqwest::RequestBuilder, &AuthContext) -> Fut + Send + Sync + 'static,
    Fut:
        std::future::Future<Output = Result<reqwest::RequestBuilder, CallerError>> + Send + 'static,
{
    AuthRegistry::update_closure(name, f)
}

/// Remove an authentication provider from the global registry.
///
/// # Arguments
/// * `name` - name of the authentication provider to remove
///
/// # Returns
/// `true` if the provider was removed, `false` if it didn't exist
pub fn remove_auth(name: &str) -> Result<bool, CallerError> {
    AuthRegistry::remove(name)
}

/// Clear all authentication providers from the global registry.
pub fn clear_auth() -> Result<(), CallerError> {
    AuthRegistry::clear()
}

/// List all authentication provider names from the global registry.
pub fn list_auth() -> Result<Vec<String>, CallerError> {
    AuthRegistry::list()
}

/// Get the number of authentication providers in the global registry.
pub fn auth_count() -> Result<usize, CallerError> {
    AuthRegistry::count()
}

fn global_caller() -> Result<Caller, CallerError> {
    let caller = Caller::from_config(config::config_loader::ConfigLoader::get_full_config()?)?;
    caller.replace_auth_providers(AuthRegistry::snapshot()?)?;
    Ok(caller)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::config_loader::{ConfigLoader, TEST_STATE_LOCK};

    fn auth_config(auth_type: &str) -> CallerConfig {
        CallerConfig {
            authorizations: vec![],
            service_items: vec![ServiceConfig {
                api_name: "GlobalSvc".to_string(),
                authorization_type: Some(auth_type.to_string()),
                base_url: "http://127.0.0.1:9".to_string(),
                timeout: Some(50),
                api_items: vec![ApiConfig {
                    method: "list".to_string(),
                    url: "/items".to_string(),
                    http_method: "GET".to_string(),
                    param_type: "none".to_string(),
                    description: None,
                    need_cache: None,
                    cache_time: None,
                    content_type: None,
                    authorization_type: None,
                    timeout: Some(50),
                    use_new_http_client: None,
                }],
                use_new_http_client: None,
            }],
        }
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn test_global_call_delegates_through_instance_model() {
        let _guard = TEST_STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        ConfigLoader::reset_state_for_test();
        clear_auth().unwrap();
        ConfigLoader::init_with_config(auth_config("token"));

        let err = call("GlobalSvc.list", None)
            .await
            .expect_err("missing global auth should fail");
        assert!(matches!(err, CallerError::UnknownAuthProvider { .. }));

        register_auth("token", NoAuth).unwrap();
        let err = call("GlobalSvc.list", None)
            .await
            .expect_err("configured auth should let request reach network layer");
        assert!(matches!(
            err,
            CallerError::NetworkError(_) | CallerError::HttpError(_)
        ));

        clear_auth().unwrap();
        ConfigLoader::reset_state_for_test();
    }
}
