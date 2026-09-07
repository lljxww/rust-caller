use async_trait::async_trait;
use reqwest::RequestBuilder;
use std::collections::HashMap;
use std::sync::Arc;

use crate::shared::error::CallerError;

/// Authentication trait that users can implement for custom authentication
///
/// # Example
/// ```rust
/// use caller::{AuthContext, Authenticator};
/// use reqwest::RequestBuilder;
/// use caller::CallerError;
/// use async_trait::async_trait;
///
/// pub struct BearerAuth {
///     token: String,
/// }
///
/// impl BearerAuth {
///     pub fn new(token: String) -> Self {
///         Self { token }
///     }
/// }
///
/// #[async_trait]
/// impl Authenticator for BearerAuth {
///     async fn authenticate(&self, builder: RequestBuilder, _context: &AuthContext) -> Result<RequestBuilder, CallerError> {
///         Ok(builder.bearer_auth(&self.token))
///     }
/// }
/// ```
#[async_trait]
pub trait Authenticator: Send + Sync {
    /// Apply authentication to the request builder
    ///
    /// # Arguments
    /// * `builder` - The request builder to modify
    /// * `context` - Authentication context with request details
    ///
    /// # Returns
    /// Modified request builder with authentication applied
    async fn authenticate(
        &self,
        builder: RequestBuilder,
        context: &AuthContext,
    ) -> Result<RequestBuilder, CallerError>;
}

/// Context information for authentication
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// Service name (e.g., "GitHub_API")
    pub service_name: String,
    /// API method name (e.g., "get_user")
    pub api_name: String,
    /// Full method path (e.g., "GitHub_API.get_user")
    pub method: String,
    /// Request URL
    pub url: String,
    /// HTTP method (GET, POST, etc.)
    pub http_method: String,
    /// Request parameters
    pub params: Option<HashMap<String, String>>,
    /// AuthConfig type name from config
    pub auth_type: String,
}

impl AuthContext {
    /// Build authentication context for a configured service endpoint.
    pub fn new(
        service_name: String,
        api_name: String,
        url: String,
        http_method: String,
        params: Option<HashMap<String, String>>,
        auth_type: String,
    ) -> Self {
        let method = format!("{}.{}", service_name, api_name);
        Self {
            service_name,
            api_name,
            method,
            url,
            http_method,
            params,
            auth_type,
        }
    }
}

/// Custom authentication function type for maximum flexibility
///
/// This allows users to provide a closure that has full access to modify the request
///
/// # Arguments
/// * `builder` - The request builder to modify
/// * `context` - Authentication context with request details
///
/// # Returns
/// Modified request builder or error
pub(crate) type AuthFn = dyn Fn(
        RequestBuilder,
        &AuthContext,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<RequestBuilder, CallerError>> + Send>,
    > + Send
    + Sync;

/// Wrapper to support both trait and closure based authentication
#[derive(Clone)]
pub enum AuthProvider {
    /// Trait-based authentication (can be cached and reused)
    Trait(Arc<dyn Authenticator>),
    /// Closure-based authentication for one-off custom auth
    Closure(Arc<AuthFn>),
}

impl AuthProvider {
    /// Create from an Authenticator trait implementation
    pub fn from_trait(auth: impl Authenticator + 'static) -> Self {
        Self::Trait(Arc::new(auth))
    }

    /// Create from a closure
    pub fn from_closure<F, Fut>(f: F) -> Self
    where
        F: Fn(RequestBuilder, &AuthContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<RequestBuilder, CallerError>> + Send + 'static,
    {
        Self::Closure(Arc::new(move |builder, ctx| Box::pin(f(builder, ctx))))
    }

    /// Apply authentication to request builder
    pub async fn apply(
        &self,
        builder: RequestBuilder,
        context: &AuthContext,
    ) -> Result<RequestBuilder, CallerError> {
        match self {
            AuthProvider::Trait(auth) => auth.authenticate(builder, context).await,
            AuthProvider::Closure(f) => f(builder, context).await,
        }
    }
}
