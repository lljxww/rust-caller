pub(crate) mod api_config;
pub(crate) mod api_result;
pub(crate) mod auth_config;
pub(crate) mod auth_registry;
pub(crate) mod auth_trait;
pub(crate) mod builtin_auth;
pub(crate) mod caller_config;
pub(crate) mod download_result;
pub(crate) mod middleware;
pub(crate) mod retry_config;
pub(crate) mod service_config;

// Re-export commonly used types
pub use api_config::{ApiConfig, HttpMethod, ParamType};
pub use api_result::{ApiResult, ResponseBody};
pub use auth_config::AuthConfig;
pub use caller_config::CallerConfig;
pub use download_result::DownloadResult;
pub use middleware::{
    CircuitBreakerConfig, CircuitBreakerMiddleware, CircuitBreakerStats, HeaderMiddleware,
    LoggingMiddleware, Middleware, MiddlewareChain, RequestContext, ResponseContext,
    TimingMiddleware, UserAgentMiddleware,
};
pub use retry_config::RetryConfig;
pub use service_config::ServiceConfig;
