//! HTTP server for Swagger UI and API proxy
//!
//! This module provides a built-in HTTP server that serves:
//! - OpenAPI specification
//! - Swagger UI
//! - API proxy for testing requests

#[cfg(feature = "server")]
mod server_impl;

#[cfg(feature = "server")]
pub use server_impl::*;

use crate::CallerError;
use std::net::SocketAddr;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server listen address
    pub addr: SocketAddr,
    /// API title for Swagger UI
    pub title: String,
    /// API version
    pub version: String,
    /// Whether API proxy routes are enabled.
    pub enable_proxy: bool,
    /// Allow binding to a non-loopback address.
    pub allow_remote: bool,
    /// Allow any browser origin through CORS.
    pub allow_any_origin: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:8080".parse().unwrap(),
            title: "Caller API".to_string(),
            version: "1.0.0".to_string(),
            enable_proxy: true,
            allow_remote: false,
            allow_any_origin: false,
        }
    }
}

impl ServerConfig {
    /// Create a new server configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set listen address
    pub fn addr(mut self, addr: &str) -> Result<Self, std::net::AddrParseError> {
        self.addr = addr.parse()?;
        Ok(self)
    }

    /// Set API title
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Set API version
    pub fn version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    /// Enable or disable the HTTP proxy routes.
    pub fn enable_proxy(mut self, enabled: bool) -> Self {
        self.enable_proxy = enabled;
        self
    }

    /// Explicitly allow binding to non-loopback interfaces.
    ///
    /// Enabling this can expose registered upstream credentials to remote
    /// callers through the proxy. Add an authentication layer before using it
    /// outside a trusted development environment.
    pub fn allow_remote(mut self, allowed: bool) -> Self {
        self.allow_remote = allowed;
        self
    }

    /// Explicitly allow all browser origins through CORS.
    pub fn allow_any_origin(mut self, allowed: bool) -> Self {
        self.allow_any_origin = allowed;
        self
    }

    /// Validate server safety and display metadata before binding a socket.
    pub fn validate(&self) -> Result<(), CallerError> {
        if self.title.trim().is_empty() {
            return Err(CallerError::config_error("Server title cannot be empty"));
        }
        if self.version.trim().is_empty() {
            return Err(CallerError::config_error("Server version cannot be empty"));
        }
        if !self.addr.ip().is_loopback() && !self.allow_remote {
            return Err(CallerError::config_error(format!(
                "Refusing to bind development proxy to non-loopback address {} without allow_remote(true)",
                self.addr
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_bind_requires_explicit_opt_in() {
        let config = ServerConfig::new().addr("0.0.0.0:8080").unwrap();
        assert!(config.validate().is_err());
        assert!(config.allow_remote(true).validate().is_ok());
    }
}
