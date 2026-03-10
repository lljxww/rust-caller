use thiserror::Error;

/// Custom error types for the caller library
#[derive(Debug, Error)]
pub enum CallerError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("API call error: {0}")]
    ApiError(String),

    #[error("HTTP request error: {0}")]
    HttpError(String),

    #[error("JSON parse error: {0}")]
    JsonError(String),

    #[error("Invalid method format: must be 'service.api', got: {0}")]
    InvalidMethodFormat(String),

    #[error("Parameter error: {0}")]
    ParameterError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("API not found: service='{0}', method='{1}'")]
    ApiNotFound(String, String),

    #[error("HTTP method not supported: {0}")]
    HttpMethodNotSupported(String),

    #[error("Configuration not initialized")]
    ConfigNotInitialized,

    #[error("URL parameter not found: {0}")]
    UrlParameterNotFound(String),

    #[error("Invalid URL format: {0}")]
    InvalidUrlFormat(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

impl From<reqwest::Error> for CallerError {
    fn from(err: reqwest::Error) -> Self {
        // Try to extract more specific information from the HTTP error
        if err.is_timeout() {
            CallerError::NetworkError("Request timeout".to_string())
        } else if err.is_redirect() {
            CallerError::HttpError(format!("Too many redirects: {}", err))
        } else if err.is_connect() {
            CallerError::NetworkError(format!("Connection error: {}", err))
        } else {
            CallerError::HttpError(err.to_string())
        }
    }
}

impl From<serde_json::Error> for CallerError {
    fn from(err: serde_json::Error) -> Self {
        CallerError::JsonError(format!(
            "JSON parsing failed at line {}: {}",
            err.line(),
            err
        ))
    }
}

impl From<std::io::Error> for CallerError {
    fn from(err: std::io::Error) -> Self {
        CallerError::IoError(err.to_string())
    }
}

impl From<notify::Error> for CallerError {
    fn from(err: notify::Error) -> Self {
        CallerError::ConfigError(format!("File watch error: {}", err))
    }
}

impl From<String> for CallerError {
    fn from(err: String) -> Self {
        CallerError::ApiError(err)
    }
}

impl From<&str> for CallerError {
    fn from(err: &str) -> Self {
        CallerError::ApiError(err.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for CallerError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        CallerError::ConfigError(err.to_string())
    }
}

impl CallerError {
    /// Creates a new configuration error
    pub fn config_error<S: Into<String>>(message: S) -> Self {
        CallerError::ConfigError(message.into())
    }

    /// Creates a new API error
    pub fn api_error<S: Into<String>>(message: S) -> Self {
        CallerError::ApiError(message.into())
    }

    /// Creates a new method format error
    pub fn invalid_method_format<S: Into<String>>(method: S) -> Self {
        CallerError::InvalidMethodFormat(method.into())
    }

    /// Creates a new service not found error
    pub fn service_not_found<S: Into<String>>(service_name: S) -> Self {
        CallerError::ServiceNotFound(service_name.into())
    }

    /// Creates a new API not found error
    pub fn api_not_found<S: Into<String>>(service_name: S, method_name: S) -> Self {
        CallerError::ApiNotFound(service_name.into(), method_name.into())
    }

    /// Creates a new HTTP method not supported error
    pub fn http_method_not_supported<S: Into<String>>(method: S) -> Self {
        CallerError::HttpMethodNotSupported(method.into())
    }

    /// Creates a new parameter error
    pub fn parameter_error<S: Into<String>>(message: S) -> Self {
        CallerError::ParameterError(message.into())
    }

    /// Creates a new URL parameter not found error
    pub fn url_parameter_not_found<S: Into<String>>(param_name: S) -> Self {
        CallerError::UrlParameterNotFound(param_name.into())
    }

    /// Creates a new authentication error
    pub fn authentication_error<S: Into<String>>(message: S) -> Self {
        CallerError::AuthenticationError(message.into())
    }

    /// Returns true if this is a configuration error
    pub fn is_config_error(&self) -> bool {
        matches!(self, CallerError::ConfigError(_))
    }

    /// Returns true if this is a network-related error
    pub fn is_network_error(&self) -> bool {
        matches!(
            self,
            CallerError::NetworkError(_) | CallerError::HttpError(_)
        )
    }

    /// Returns true if this is a method format error
    pub fn is_method_error(&self) -> bool {
        matches!(
            self,
            CallerError::InvalidMethodFormat(_) | CallerError::HttpMethodNotSupported(_)
        )
    }
}
