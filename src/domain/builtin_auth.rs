//! Built-in authentication implementations
//!
//! This module provides ready-to-use authentication implementations for common scenarios:
//! - Bearer token authentication
//! - Basic authentication
//! - API key authentication
//! - Dynamic authentication from environment variables or callbacks

use async_trait::async_trait;
use reqwest::{RequestBuilder, header};
use std::fmt;
use std::sync::{Arc, RwLock};

use super::auth_trait::{AuthContext, Authenticator};
use crate::shared::error::CallerError;

type HeaderList = Vec<(String, String)>;
type HeaderProvider = dyn Fn() -> Result<HeaderList, CallerError> + Send + Sync;

// ============================================================================
// Bearer Token Authentication
// ============================================================================

/// Bearer token authentication
#[derive(Clone)]
pub struct BearerAuth {
    token: String,
}

impl fmt::Debug for BearerAuth {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BearerAuth")
            .field("token", &"[REDACTED]")
            .finish()
    }
}

impl BearerAuth {
    /// Create a provider from an in-memory bearer token.
    pub fn new(token: String) -> Self {
        Self { token }
    }

    /// Create from environment variable
    pub fn from_env(var_name: &str) -> Result<Self, CallerError> {
        let token =
            std::env::var(var_name).map_err(|_| CallerError::MissingAuthEnvironmentVariable {
                name: var_name.to_string(),
            })?;
        Ok(Self { token })
    }

    /// Replace the bearer token.
    pub fn set_token(&mut self, token: String) {
        self.token = token;
    }

    /// Borrow the current token.
    ///
    /// Avoid including this value in logs or diagnostics.
    pub fn token(&self) -> &str {
        &self.token
    }
}

#[async_trait]
impl Authenticator for BearerAuth {
    async fn authenticate(
        &self,
        builder: RequestBuilder,
        _context: &AuthContext,
    ) -> Result<RequestBuilder, CallerError> {
        Ok(builder.bearer_auth(&self.token))
    }
}

/// Dynamic Bearer token with runtime callback
pub struct DynamicBearerAuth {
    token_provider: Arc<dyn Fn() -> Result<String, CallerError> + Send + Sync>,
}

impl DynamicBearerAuth {
    /// Create with a token provider callback
    pub fn new<F>(provider: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        Self::try_new(move || Ok(provider()))
    }

    /// Create with a fallible token provider callback.
    pub fn try_new<F>(provider: F) -> Self
    where
        F: Fn() -> Result<String, CallerError> + Send + Sync + 'static,
    {
        Self {
            token_provider: Arc::new(provider),
        }
    }

    /// Create from environment variable (read at request time)
    pub fn from_env(var_name: &str) -> Self {
        let var_name = var_name.to_string();
        Self::try_new(move || {
            std::env::var(&var_name).map_err(|_| CallerError::MissingAuthEnvironmentVariable {
                name: var_name.clone(),
            })
        })
    }

    /// Create from shared mutable state (for token refresh)
    pub fn from_shared(token: Arc<RwLock<String>>) -> Self {
        Self::try_new(move || {
            token
                .read()
                .map(|token| token.clone())
                .map_err(|_| CallerError::lock_poisoned("dynamic bearer token"))
        })
    }
}

#[async_trait]
impl Authenticator for DynamicBearerAuth {
    async fn authenticate(
        &self,
        builder: RequestBuilder,
        _context: &AuthContext,
    ) -> Result<RequestBuilder, CallerError> {
        let token = (self.token_provider)()?;
        Ok(builder.bearer_auth(token))
    }
}

// ============================================================================
// Basic Authentication
// ============================================================================

/// HTTP Basic authentication
#[derive(Clone)]
pub struct BasicAuth {
    username: String,
    password: String,
}

impl fmt::Debug for BasicAuth {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BasicAuth")
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

impl BasicAuth {
    /// Create a provider from an in-memory username and password.
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    /// Read a username and password from environment variables immediately.
    pub fn from_env(user_var: &str, pass_var: &str) -> Result<Self, CallerError> {
        let username =
            std::env::var(user_var).map_err(|_| CallerError::MissingAuthEnvironmentVariable {
                name: user_var.to_string(),
            })?;
        let password =
            std::env::var(pass_var).map_err(|_| CallerError::MissingAuthEnvironmentVariable {
                name: pass_var.to_string(),
            })?;
        Ok(Self { username, password })
    }

    /// Replace the username and password.
    pub fn set_credentials(&mut self, username: String, password: String) {
        self.username = username;
        self.password = password;
    }
}

#[async_trait]
impl Authenticator for BasicAuth {
    async fn authenticate(
        &self,
        builder: RequestBuilder,
        _context: &AuthContext,
    ) -> Result<RequestBuilder, CallerError> {
        Ok(builder.basic_auth(&self.username, Some(&self.password)))
    }
}

// ============================================================================
// API Key Authentication
// ============================================================================

/// API Key authentication via header
#[derive(Clone)]
pub struct ApiKeyAuth {
    header_name: String,
    api_key: String,
}

impl fmt::Debug for ApiKeyAuth {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApiKeyAuth")
            .field("header_name", &self.header_name)
            .field("api_key", &"[REDACTED]")
            .finish()
    }
}

impl ApiKeyAuth {
    /// Create a provider that writes an API key into the named header.
    pub fn new(header_name: String, api_key: String) -> Self {
        Self {
            header_name,
            api_key,
        }
    }

    /// Read an API key from an environment variable immediately.
    pub fn from_env(header_name: &str, var_name: &str) -> Result<Self, CallerError> {
        let api_key =
            std::env::var(var_name).map_err(|_| CallerError::MissingAuthEnvironmentVariable {
                name: var_name.to_string(),
            })?;
        Ok(Self {
            header_name: header_name.to_string(),
            api_key,
        })
    }

    /// Replace the API key.
    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = api_key;
    }
}

#[async_trait]
impl Authenticator for ApiKeyAuth {
    async fn authenticate(
        &self,
        builder: RequestBuilder,
        _context: &AuthContext,
    ) -> Result<RequestBuilder, CallerError> {
        Ok(builder.header(&self.header_name, &self.api_key))
    }
}

/// Dynamic API Key with runtime callback
pub struct DynamicApiKeyAuth {
    header_name: String,
    key_provider: Arc<dyn Fn() -> Result<String, CallerError> + Send + Sync>,
}

impl DynamicApiKeyAuth {
    /// Create a dynamic API-key provider from an infallible callback.
    pub fn new<F>(header_name: &str, provider: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        Self::try_new(header_name, move || Ok(provider()))
    }

    /// Create with a fallible API key provider callback.
    pub fn try_new<F>(header_name: &str, provider: F) -> Self
    where
        F: Fn() -> Result<String, CallerError> + Send + Sync + 'static,
    {
        Self {
            header_name: header_name.to_string(),
            key_provider: Arc::new(provider),
        }
    }

    /// Read an API key from an environment variable on every request attempt.
    pub fn from_env(header_name: &str, var_name: &str) -> Self {
        let var_name = var_name.to_string();
        Self::try_new(header_name, move || {
            std::env::var(&var_name).map_err(|_| CallerError::MissingAuthEnvironmentVariable {
                name: var_name.clone(),
            })
        })
    }

    /// Read an API key from shared mutable state on every request attempt.
    pub fn from_shared(header_name: &str, key: Arc<RwLock<String>>) -> Self {
        Self::try_new(header_name, move || {
            key.read()
                .map(|key| key.clone())
                .map_err(|_| CallerError::lock_poisoned("dynamic API key"))
        })
    }
}

#[async_trait]
impl Authenticator for DynamicApiKeyAuth {
    async fn authenticate(
        &self,
        builder: RequestBuilder,
        _context: &AuthContext,
    ) -> Result<RequestBuilder, CallerError> {
        let key = (self.key_provider)()?;
        Ok(builder.header(&self.header_name, key))
    }
}

// ============================================================================
// OAuth2 Authentication
// ============================================================================

/// OAuth 2.0 token authentication
#[derive(Clone)]
pub struct OAuth2Auth {
    token: String,
    prefix: String,
}

impl fmt::Debug for OAuth2Auth {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OAuth2Auth")
            .field("token", &"[REDACTED]")
            .field("prefix", &self.prefix)
            .finish()
    }
}

impl OAuth2Auth {
    /// Create an OAuth 2 provider using the standard `Bearer` prefix.
    pub fn new(token: String) -> Self {
        Self {
            token,
            prefix: "Bearer".to_string(),
        }
    }

    /// Read an OAuth 2 token from an environment variable immediately.
    pub fn from_env(var_name: &str) -> Result<Self, CallerError> {
        let token =
            std::env::var(var_name).map_err(|_| CallerError::MissingAuthEnvironmentVariable {
                name: var_name.to_string(),
            })?;
        Ok(Self::new(token))
    }

    /// Create a token provider with a non-default authorization scheme prefix.
    pub fn with_prefix(token: String, prefix: String) -> Self {
        Self { token, prefix }
    }

    /// Replace the OAuth 2 access token.
    pub fn set_token(&mut self, token: String) {
        self.token = token;
    }
}

#[async_trait]
impl Authenticator for OAuth2Auth {
    async fn authenticate(
        &self,
        builder: RequestBuilder,
        _context: &AuthContext,
    ) -> Result<RequestBuilder, CallerError> {
        let header_value = format!("{} {}", self.prefix, self.token);
        let header_value = header::HeaderValue::from_str(&header_value).map_err(|error| {
            CallerError::InvalidHeaderValue {
                name: header::AUTHORIZATION.as_str().to_string(),
                message: error.to_string(),
            }
        })?;
        Ok(builder.header(header::AUTHORIZATION, header_value))
    }
}

// ============================================================================
// Custom Header Authentication
// ============================================================================

/// Custom header authentication
#[derive(Clone, Default)]
pub struct CustomHeaderAuth {
    headers: Vec<(String, String)>,
}

impl fmt::Debug for CustomHeaderAuth {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let header_names: Vec<&str> = self.headers.iter().map(|(name, _)| name.as_str()).collect();
        formatter
            .debug_struct("CustomHeaderAuth")
            .field("header_names", &header_names)
            .field("values", &"[REDACTED]")
            .finish()
    }
}

impl CustomHeaderAuth {
    /// Create an empty fixed-header provider.
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
        }
    }

    /// Append a header that will be applied during authentication.
    pub fn add_header(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.headers.push((name.into(), value.into()));
    }

    /// Remove all configured authentication headers.
    pub fn clear(&mut self) {
        self.headers.clear();
    }
}

#[async_trait]
impl Authenticator for CustomHeaderAuth {
    async fn authenticate(
        &self,
        mut builder: RequestBuilder,
        _context: &AuthContext,
    ) -> Result<RequestBuilder, CallerError> {
        for (name, value) in &self.headers {
            builder = builder.header(name, value);
        }
        Ok(builder)
    }
}

/// Dynamic custom headers with runtime callbacks
pub struct DynamicHeaderAuth {
    headers_provider: Arc<HeaderProvider>,
}

impl DynamicHeaderAuth {
    /// Create a dynamic header provider from an infallible callback.
    pub fn new<F>(provider: F) -> Self
    where
        F: Fn() -> Vec<(String, String)> + Send + Sync + 'static,
    {
        Self::try_new(move || Ok(provider()))
    }

    /// Create with a fallible header provider callback.
    pub fn try_new<F>(provider: F) -> Self
    where
        F: Fn() -> Result<Vec<(String, String)>, CallerError> + Send + Sync + 'static,
    {
        Self {
            headers_provider: Arc::new(provider),
        }
    }

    /// Read authentication headers from shared mutable state for every attempt.
    pub fn from_shared(headers: Arc<RwLock<Vec<(String, String)>>>) -> Self {
        Self::try_new(move || {
            headers
                .read()
                .map(|headers| headers.clone())
                .map_err(|_| CallerError::lock_poisoned("dynamic authentication headers"))
        })
    }
}

#[async_trait]
impl Authenticator for DynamicHeaderAuth {
    async fn authenticate(
        &self,
        mut builder: RequestBuilder,
        _context: &AuthContext,
    ) -> Result<RequestBuilder, CallerError> {
        let headers = (self.headers_provider)()?;
        for (name, value) in headers {
            builder = builder.header(name, value);
        }
        Ok(builder)
    }
}

// ============================================================================
// No Authentication
// ============================================================================

/// No-op authentication (for testing or optional auth)
#[derive(Debug, Clone, Default)]
pub struct NoAuth;

impl NoAuth {
    /// Create a no-op provider.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Authenticator for NoAuth {
    async fn authenticate(
        &self,
        builder: RequestBuilder,
        _context: &AuthContext,
    ) -> Result<RequestBuilder, CallerError> {
        Ok(builder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bearer_auth() {
        let auth = BearerAuth::new("test-token".to_string());
        assert_eq!(auth.token(), "test-token");
    }

    #[test]
    fn test_basic_auth() {
        let auth = BasicAuth::new("user".to_string(), "pass".to_string());
        assert_eq!(auth.username, "user");
        assert_eq!(auth.password, "pass");
    }

    #[test]
    fn test_api_key_auth() {
        let auth = ApiKeyAuth::new("X-API-Key".to_string(), "key123".to_string());
        assert_eq!(auth.header_name, "X-API-Key");
        assert_eq!(auth.api_key, "key123");
    }

    #[test]
    fn test_oauth2_auth() {
        let auth = OAuth2Auth::new("token123".to_string());
        assert_eq!(auth.token, "token123");
        assert_eq!(auth.prefix, "Bearer");

        let auth_custom = OAuth2Auth::with_prefix("token123".to_string(), "OAuth".to_string());
        assert_eq!(auth_custom.prefix, "OAuth");
    }

    #[test]
    fn test_custom_header_auth() {
        let mut auth = CustomHeaderAuth::new();
        auth.add_header("X-Key1", "value1");
        auth.add_header("X-Key2", "value2");
        assert_eq!(auth.headers.len(), 2);
    }

    #[test]
    fn test_dynamic_bearer_from_shared() {
        let token = Arc::new(RwLock::new("initial".to_string()));
        let auth = DynamicBearerAuth::from_shared(token.clone());

        // Update token
        *token.write().unwrap() = "updated".to_string();
        // Token will be "updated" on next request
        let _ = auth;
    }
}
