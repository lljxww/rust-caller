use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Config,
    Runtime,
    Protocol,
    Security,
}

/// Custom error types for the caller library
#[derive(Debug, Error)]
pub enum CallerError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Configuration file not found: {path}")]
    ConfigFileNotFound { path: String },

    #[error("Unsupported config file format: {path}")]
    UnsupportedConfigFormat { path: String },

    #[error("Failed to parse {format} config at {path}: {message}")]
    ConfigParseError {
        path: String,
        format: String,
        message: String,
    },

    #[error("Failed to serialize config as {format}: {message}")]
    ConfigSerializeError { format: String, message: String },

    #[error("Failed to watch config file at {path}: {message}")]
    ConfigWatchError { path: String, message: String },

    #[error("API call error: {0}")]
    ApiError(String),

    #[error("HTTP request error: {0}")]
    HttpError(String),

    #[error("Request timed out")]
    RequestTimeout,

    #[error("Too many redirects: {message}")]
    TooManyRedirects { message: String },

    #[error("Connection error: {message}")]
    ConnectionError { message: String },

    #[error("Failed to create HTTP client: {message}")]
    HttpClientBuildError { message: String },

    #[error("HTTP {status} is retryable (attempt {attempt}/{max_retries})")]
    RetryableHttpStatus {
        status: u16,
        attempt: u32,
        max_retries: u32,
    },

    #[error("All retry attempts exhausted")]
    RetryAttemptsExhausted,

    #[error("JSON parse error: {0}")]
    JsonError(String),

    #[error("Invalid method format: must be 'service.api', got: {method}")]
    InvalidMethodFormat { method: String },

    #[error("Parameter error: {0}")]
    ParameterError(String),

    #[error("Invalid header name '{name}': {message}")]
    InvalidHeaderName { name: String, message: String },

    #[error("Invalid header value for '{name}': {message}")]
    InvalidHeaderValue { name: String, message: String },

    #[error("Invalid user-agent '{value}': {message}")]
    InvalidUserAgent { value: String, message: String },

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Lock poisoned: {resource}")]
    LockPoisoned { resource: String },

    #[error("Service not found: {service}")]
    ServiceNotFound { service: String },

    #[error("API not found: service='{service}', method='{method}'")]
    ApiNotFound { service: String, method: String },

    #[error("HTTP method not supported: {method}")]
    HttpMethodNotSupported { method: String },

    #[error("Configuration not initialized")]
    ConfigNotInitialized,

    #[error("Caller instance has no config path")]
    MissingCallerConfigPath,

    #[error("CallerBuilder requires either config or config_path")]
    MissingCallerBuilderConfig,

    #[error("Missing path parameter: {name}")]
    MissingPathParameter { name: String },

    #[error("Invalid URL format: {0}")]
    InvalidUrlFormat(String),

    #[error("Invalid URL '{url}': {message}")]
    InvalidUrl { url: String, message: String },

    #[error("Unsupported URL scheme '{scheme}' in {url}")]
    UnsupportedUrlScheme { url: String, scheme: String },

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("Missing authentication environment variable: {name}")]
    MissingAuthEnvironmentVariable { name: String },

    #[error("Authentication provider not registered: {name}")]
    UnknownAuthProvider { name: String },

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Unsupported parameter type: {value}")]
    UnsupportedParamType { value: String },

    #[error("Request blocked: {0}")]
    RequestError(String),

    #[error("Failed to decode text response: {message}")]
    TextDecodingError { message: String },
}

impl From<reqwest::Error> for CallerError {
    fn from(err: reqwest::Error) -> Self {
        // Try to extract more specific information from the HTTP error
        if err.is_timeout() {
            CallerError::RequestTimeout
        } else if err.is_redirect() {
            CallerError::TooManyRedirects {
                message: err.to_string(),
            }
        } else if err.is_connect() {
            CallerError::ConnectionError {
                message: err.to_string(),
            }
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
        if err.kind() == std::io::ErrorKind::NotFound {
            CallerError::IoError(format!("Not found: {}", err))
        } else {
            CallerError::IoError(err.to_string())
        }
    }
}

impl From<notify::Error> for CallerError {
    fn from(err: notify::Error) -> Self {
        CallerError::ConfigWatchError {
            path: "<unknown>".to_string(),
            message: err.to_string(),
        }
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

    pub fn config_file_not_found<S: Into<String>>(path: S) -> Self {
        CallerError::ConfigFileNotFound { path: path.into() }
    }

    pub fn unsupported_config_format<S: Into<String>>(path: S) -> Self {
        CallerError::UnsupportedConfigFormat { path: path.into() }
    }

    pub fn config_parse_error<S1: Into<String>, S2: Into<String>, S3: Into<String>>(
        path: S1,
        format: S2,
        message: S3,
    ) -> Self {
        CallerError::ConfigParseError {
            path: path.into(),
            format: format.into(),
            message: message.into(),
        }
    }

    pub fn config_serialize_error<S1: Into<String>, S2: Into<String>>(
        format: S1,
        message: S2,
    ) -> Self {
        CallerError::ConfigSerializeError {
            format: format.into(),
            message: message.into(),
        }
    }

    pub fn config_watch_error<S1: Into<String>, S2: Into<String>>(path: S1, message: S2) -> Self {
        CallerError::ConfigWatchError {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Creates a new API error
    pub fn api_error<S: Into<String>>(message: S) -> Self {
        CallerError::ApiError(message.into())
    }

    /// Creates a new method format error
    pub fn invalid_method_format<S: Into<String>>(method: S) -> Self {
        CallerError::InvalidMethodFormat {
            method: method.into(),
        }
    }

    /// Creates a new service not found error
    pub fn service_not_found<S: Into<String>>(service_name: S) -> Self {
        CallerError::ServiceNotFound {
            service: service_name.into(),
        }
    }

    /// Creates a new API not found error
    pub fn api_not_found<S1: Into<String>, S2: Into<String>>(
        service_name: S1,
        method_name: S2,
    ) -> Self {
        CallerError::ApiNotFound {
            service: service_name.into(),
            method: method_name.into(),
        }
    }

    /// Creates a new HTTP method not supported error
    pub fn http_method_not_supported<S: Into<String>>(method: S) -> Self {
        CallerError::HttpMethodNotSupported {
            method: method.into(),
        }
    }

    /// Creates a new parameter error
    pub fn parameter_error<S: Into<String>>(message: S) -> Self {
        CallerError::ParameterError(message.into())
    }

    /// Creates a new URL parameter not found error
    pub fn url_parameter_not_found<S: Into<String>>(param_name: S) -> Self {
        CallerError::MissingPathParameter {
            name: param_name.into(),
        }
    }

    /// Creates a new authentication error
    pub fn authentication_error<S: Into<String>>(message: S) -> Self {
        CallerError::AuthenticationError(message.into())
    }

    pub fn lock_poisoned<S: Into<String>>(resource: S) -> Self {
        CallerError::LockPoisoned {
            resource: resource.into(),
        }
    }

    /// Creates an unknown authentication provider error
    pub fn unknown_auth_provider<S: Into<String>>(name: S) -> Self {
        CallerError::UnknownAuthProvider { name: name.into() }
    }

    /// Creates an unsupported parameter type error
    pub fn unsupported_param_type<S: Into<String>>(value: S) -> Self {
        CallerError::UnsupportedParamType {
            value: value.into(),
        }
    }

    pub fn text_decoding_error<S: Into<String>>(message: S) -> Self {
        CallerError::TextDecodingError {
            message: message.into(),
        }
    }

    /// Returns the broad category of the error
    pub fn category(&self) -> ErrorCategory {
        match self {
            CallerError::ConfigError(_)
            | CallerError::ConfigFileNotFound { .. }
            | CallerError::UnsupportedConfigFormat { .. }
            | CallerError::ConfigParseError { .. }
            | CallerError::ConfigSerializeError { .. }
            | CallerError::ConfigWatchError { .. }
            | CallerError::LockPoisoned { .. }
            | CallerError::ConfigNotInitialized
            | CallerError::MissingCallerConfigPath
            | CallerError::MissingCallerBuilderConfig
            | CallerError::InvalidMethodFormat { .. } => ErrorCategory::Config,
            CallerError::JsonError(_)
            | CallerError::HttpMethodNotSupported { .. }
            | CallerError::MissingPathParameter { .. }
            | CallerError::InvalidUrlFormat(_)
            | CallerError::InvalidUrl { .. }
            | CallerError::UnsupportedUrlScheme { .. }
            | CallerError::UnsupportedParamType { .. }
            | CallerError::TextDecodingError { .. } => ErrorCategory::Protocol,
            CallerError::AuthenticationError(_)
            | CallerError::MissingAuthEnvironmentVariable { .. }
            | CallerError::UnknownAuthProvider { .. }
            | CallerError::RequestError(_) => ErrorCategory::Security,
            CallerError::ApiError(_)
            | CallerError::ParameterError(_)
            | CallerError::InvalidHeaderName { .. }
            | CallerError::InvalidHeaderValue { .. }
            | CallerError::InvalidUserAgent { .. }
            | CallerError::IoError(_)
            | CallerError::ServiceNotFound { .. }
            | CallerError::ApiNotFound { .. }
            | CallerError::NetworkError(_)
            | CallerError::RequestTimeout
            | CallerError::TooManyRedirects { .. }
            | CallerError::ConnectionError { .. }
            | CallerError::HttpClientBuildError { .. }
            | CallerError::RetryableHttpStatus { .. }
            | CallerError::RetryAttemptsExhausted
            | CallerError::SerializationError(_)
            | CallerError::HttpError(_) => ErrorCategory::Runtime,
        }
    }

    /// Returns true if this is a configuration error
    pub fn is_config_error(&self) -> bool {
        self.category() == ErrorCategory::Config
    }

    /// Returns true if this is a network-related error
    pub fn is_network_error(&self) -> bool {
        matches!(
            self,
            CallerError::NetworkError(_)
                | CallerError::RequestTimeout
                | CallerError::TooManyRedirects { .. }
                | CallerError::ConnectionError { .. }
                | CallerError::HttpClientBuildError { .. }
                | CallerError::RetryableHttpStatus { .. }
                | CallerError::RetryAttemptsExhausted
                | CallerError::HttpError(_)
        )
    }

    /// Returns true if this is a method format error
    pub fn is_method_error(&self) -> bool {
        matches!(
            self,
            CallerError::InvalidMethodFormat { .. } | CallerError::HttpMethodNotSupported { .. }
        )
    }
}
