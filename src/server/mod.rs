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
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:8080".parse().unwrap(),
            title: "Caller API".to_string(),
            version: "1.0.0".to_string(),
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
}
