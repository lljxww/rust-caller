pub mod api_config;
pub mod api_result;
pub mod auth_config;
pub mod auth_registry;
pub mod auth_trait;
pub mod builtin_auth;
pub mod caller_config;
pub mod download_result;
pub mod middleware;
pub mod retry_config;
pub mod service_config;

// Re-export commonly used types
pub use api_config::{ApiConfig, HttpMethod, ParamType};
pub use api_result::{ApiResult, ResponseBody};
pub use auth_config::AuthConfig;
pub use auth_registry::AuthRegistry;
pub use auth_trait::{AuthContext, AuthProvider, Authenticator};
pub use builtin_auth::{
    ApiKeyAuth, BasicAuth, BearerAuth, CustomHeaderAuth, DynamicApiKeyAuth, DynamicBearerAuth,
    DynamicHeaderAuth, NoAuth, OAuth2Auth,
};
pub use caller_config::CallerConfig;
pub use download_result::DownloadResult;
pub use middleware::{
    CircuitBreakerConfig, CircuitBreakerMiddleware, CircuitBreakerStats, HeaderMiddleware,
    LoggingMiddleware, Middleware, MiddlewareChain, RequestContext, ResponseContext,
    RetryMiddleware, TimingMiddleware, UserAgentMiddleware,
};
pub use retry_config::RetryConfig;
pub use service_config::ServiceConfig;
