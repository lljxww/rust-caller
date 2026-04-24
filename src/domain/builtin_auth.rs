//! Built-in authentication implementations
//!
//! This module provides ready-to-use authentication implementations for common scenarios:
//! - Bearer token authentication
//! - Basic authentication
//! - API key authentication
//! - Dynamic authentication from environment variables or callbacks

use async_trait::async_trait;
use reqwest::RequestBuilder;
use std::sync::{Arc, RwLock};

use super::auth_trait::{AuthContext, Authenticator};
use crate::shared::error::CallerError;

// ============================================================================
// Bearer Token Authentication
// ============================================================================

/// Bearer token authentication
#[derive(Debug, Clone)]
pub struct BearerAuth {
    token: String,
}

impl BearerAuth {
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

    pub fn set_token(&mut self, token: String) {
        self.token = token;
    }

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
    token_provider: Arc<dyn Fn() -> String + Send + Sync>,
}

impl DynamicBearerAuth {
    /// Create with a token provider callback
    pub fn new<F>(provider: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        Self {
            token_provider: Arc::new(provider),
        }
    }

    /// Create from environment variable (read at request time)
    pub fn from_env(var_name: &str) -> Self {
        let var_name = var_name.to_string();
        Self::new(move || std::env::var(&var_name).unwrap_or_default())
    }

    /// Create from shared mutable state (for token refresh)
    pub fn from_shared(token: Arc<RwLock<String>>) -> Self {
        Self::new(move || token.read().map(|t| t.clone()).unwrap_or_default())
    }
}

#[async_trait]
impl Authenticator for DynamicBearerAuth {
    async fn authenticate(
        &self,
        builder: RequestBuilder,
        _context: &AuthContext,
    ) -> Result<RequestBuilder, CallerError> {
        let token = (self.token_provider)();
        Ok(builder.bearer_auth(token))
    }
}

// ============================================================================
// Basic Authentication
// ============================================================================

/// HTTP Basic authentication
#[derive(Debug, Clone)]
pub struct BasicAuth {
    username: String,
    password: String,
}

impl BasicAuth {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

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
#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    header_name: String,
    api_key: String,
}

impl ApiKeyAuth {
    pub fn new(header_name: String, api_key: String) -> Self {
        Self {
            header_name,
            api_key,
        }
    }

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
    key_provider: Arc<dyn Fn() -> String + Send + Sync>,
}

impl DynamicApiKeyAuth {
    pub fn new<F>(header_name: &str, provider: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        Self {
            header_name: header_name.to_string(),
            key_provider: Arc::new(provider),
        }
    }

    pub fn from_env(header_name: &str, var_name: &str) -> Self {
        let var_name = var_name.to_string();
        Self::new(header_name, move || {
            std::env::var(&var_name).unwrap_or_default()
        })
    }

    pub fn from_shared(header_name: &str, key: Arc<RwLock<String>>) -> Self {
        Self::new(header_name, move || {
            key.read().map(|k| k.clone()).unwrap_or_default()
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
        let key = (self.key_provider)();
        Ok(builder.header(&self.header_name, key))
    }
}

// ============================================================================
// OAuth2 Authentication
// ============================================================================

/// OAuth 2.0 token authentication
#[derive(Debug, Clone)]
pub struct OAuth2Auth {
    token: String,
    prefix: String,
}

impl OAuth2Auth {
    pub fn new(token: String) -> Self {
        Self {
            token,
            prefix: "Bearer".to_string(),
        }
    }

    pub fn from_env(var_name: &str) -> Result<Self, CallerError> {
        let token =
            std::env::var(var_name).map_err(|_| CallerError::MissingAuthEnvironmentVariable {
                name: var_name.to_string(),
            })?;
        Ok(Self::new(token))
    }

    pub fn with_prefix(token: String, prefix: String) -> Self {
        Self { token, prefix }
    }

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
        Ok(builder.header("AuthConfig", header_value))
    }
}

// ============================================================================
// Custom Header Authentication
// ============================================================================

/// Custom header authentication
#[derive(Debug, Clone, Default)]
pub struct CustomHeaderAuth {
    headers: Vec<(String, String)>,
}

impl CustomHeaderAuth {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
        }
    }

    pub fn add_header(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.headers.push((name.into(), value.into()));
    }

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
    headers_provider: Arc<dyn Fn() -> Vec<(String, String)> + Send + Sync>,
}

impl DynamicHeaderAuth {
    pub fn new<F>(provider: F) -> Self
    where
        F: Fn() -> Vec<(String, String)> + Send + Sync + 'static,
    {
        Self {
            headers_provider: Arc::new(provider),
        }
    }

    pub fn from_shared(headers: Arc<RwLock<Vec<(String, String)>>>) -> Self {
        Self::new(move || headers.read().map(|h| h.clone()).unwrap_or_default())
    }
}

#[async_trait]
impl Authenticator for DynamicHeaderAuth {
    async fn authenticate(
        &self,
        mut builder: RequestBuilder,
        _context: &AuthContext,
    ) -> Result<RequestBuilder, CallerError> {
        let headers = (self.headers_provider)();
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
