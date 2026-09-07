use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Broad error classification suitable for metrics and policy decisions.
pub enum ErrorCategory {
    /// Invalid, missing, or inaccessible configuration.
    Config,
    /// A failure during endpoint lookup, I/O, HTTP transport, or HTTP status handling.
    Runtime,
    /// Invalid request or response data at the HTTP protocol boundary.
    Protocol,
    /// Authentication failure or an intentional request rejection.
    Security,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Classification of the underlying `reqwest` transport failure.
pub enum TransportErrorKind {
    /// The request exceeded a configured timeout.
    Timeout,
    /// Redirect handling failed, for example because a redirect limit was reached.
    Redirect,
    /// Establishing the upstream connection failed.
    Connect,
    /// The HTTP request could not be constructed.
    Builder,
    /// Another error occurred while sending or receiving the request.
    Request,
}

impl fmt::Display for TransportErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Timeout => "timeout",
            Self::Redirect => "redirect",
            Self::Connect => "connect",
            Self::Builder => "builder",
            Self::Request => "request",
        };
        formatter.write_str(value)
    }
}

/// Custom error types for the caller library
#[derive(Debug, Error)]
pub enum CallerError {
    /// Generic configuration validation failure.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// The requested configuration file does not exist.
    #[error("Configuration file not found: {path}")]
    ConfigFileNotFound {
        /// Requested filesystem path.
        path: String,
    },

    /// The configuration file extension does not select a supported format.
    #[error("Unsupported config file format: {path}")]
    UnsupportedConfigFormat {
        /// Requested filesystem path.
        path: String,
    },

    /// JSON, YAML, or TOML configuration parsing failed.
    #[error("Failed to parse {format} config at {path}: {message}")]
    ConfigParseError {
        /// Configuration source path or diagnostic source name.
        path: String,
        /// Detected or explicitly selected configuration format.
        format: String,
        /// Parser diagnostic.
        message: String,
    },

    /// Serializing configuration to a selected format failed.
    #[error("Failed to serialize config as {format}: {message}")]
    ConfigSerializeError {
        /// Target configuration format.
        format: String,
        /// Serializer diagnostic.
        message: String,
    },

    /// Starting or operating the configuration file watcher failed.
    #[error("Failed to watch config file at {path}: {message}")]
    ConfigWatchError {
        /// Watched path, or `<unknown>` when the source did not provide one.
        path: String,
        /// Watcher diagnostic.
        message: String,
    },

    /// Sending or receiving an HTTP request failed before a response was available.
    #[error("HTTP transport error ({kind}): {source}")]
    HttpTransport {
        /// Stable high-level failure kind.
        kind: TransportErrorKind,
        /// Original transport error with its source chain intact.
        #[source]
        source: reqwest::Error,
    },

    /// A received 4xx or 5xx response was converted with `error_for_status`.
    #[error("HTTP response status {status}: {body_preview}")]
    HttpStatus {
        /// HTTP status code.
        status: u16,
        /// Bounded response-body preview for diagnostics.
        body_preview: String,
    },

    /// JSON parsing failed.
    #[error("JSON parse error: {0}")]
    JsonError(String),

    /// A lookup key did not use the required `service.api` format.
    #[error("Invalid method format: must be 'service.api', got: {method}")]
    InvalidMethodFormat {
        /// Invalid lookup key.
        method: String,
    },

    /// Request parameters could not be represented at the configured location.
    #[error("Parameter error: {0}")]
    ParameterError(String),

    /// An HTTP header name was syntactically invalid.
    #[error("Invalid header name '{name}': {message}")]
    InvalidHeaderName {
        /// Rejected header name.
        name: String,
        /// Parser diagnostic.
        message: String,
    },

    /// An HTTP header value was syntactically invalid.
    #[error("Invalid header value for '{name}': {message}")]
    InvalidHeaderValue {
        /// Header whose value was rejected.
        name: String,
        /// Parser diagnostic.
        message: String,
    },

    /// A configured user-agent could not be represented as a header value.
    #[error("Invalid user-agent '{value}': {message}")]
    InvalidUserAgent {
        /// Rejected user-agent string.
        value: String,
        /// Parser diagnostic.
        message: String,
    },

    /// A filesystem or stream operation failed.
    #[error("I/O error while {context}: {source}")]
    Io {
        /// Operation being performed when the failure occurred.
        context: String,
        /// Original I/O error.
        #[source]
        source: std::io::Error,
    },

    /// A shared lock was poisoned by a panic in another thread.
    #[error("Lock poisoned: {resource}")]
    LockPoisoned {
        /// Shared resource protected by the poisoned lock.
        resource: String,
    },

    /// No configured service matched the lookup key.
    #[error("Service not found: {service}")]
    ServiceNotFound {
        /// Missing service name.
        service: String,
    },

    /// No configured endpoint matched the method within a known service.
    #[error("API not found: service='{service}', method='{method}'")]
    ApiNotFound {
        /// Known service name.
        service: String,
        /// Missing API method name.
        method: String,
    },

    /// The configured HTTP method is not supported by this crate.
    #[error("HTTP method not supported: {method}")]
    HttpMethodNotSupported {
        /// Rejected HTTP method name.
        method: String,
    },

    /// A global helper was used before global configuration initialization.
    #[error("Configuration not initialized")]
    ConfigNotInitialized,

    /// Reload was requested for a `Caller` built from in-memory configuration.
    #[error("Caller instance has no config path")]
    MissingCallerConfigPath,

    /// `CallerBuilder` was not given either a path or an in-memory configuration.
    #[error("CallerBuilder requires either config or config_path")]
    MissingCallerBuilderConfig,

    /// A required URL path placeholder had no corresponding parameter.
    #[error("Missing path parameter: {name}")]
    MissingPathParameter {
        /// Missing placeholder name.
        name: String,
    },

    /// A configured or constructed URL could not be parsed or validated.
    #[error("Invalid URL '{url}': {message}")]
    InvalidUrl {
        /// Rejected URL.
        url: String,
        /// Validation diagnostic.
        message: String,
    },

    /// A configured URL used a scheme other than HTTP or HTTPS.
    #[error("Unsupported URL scheme '{scheme}' in {url}")]
    UnsupportedUrlScheme {
        /// Rejected URL.
        url: String,
        /// Unsupported scheme.
        scheme: String,
    },

    /// A runtime authentication provider failed to produce valid credentials.
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    /// An authentication provider's required environment variable was absent.
    #[error("Missing authentication environment variable: {name}")]
    MissingAuthEnvironmentVariable {
        /// Missing environment variable name.
        name: String,
    },

    /// Configuration referenced an authentication provider that was not registered.
    #[error("Authentication provider not registered: {name}")]
    UnknownAuthProvider {
        /// Missing provider name.
        name: String,
    },

    /// A configuration value named an unsupported or inconsistent parameter type.
    #[error("Unsupported parameter type: {value}")]
    UnsupportedParamType {
        /// Rejected parameter-type representation.
        value: String,
    },

    /// Middleware intentionally blocked a request.
    #[error("Request blocked: {0}")]
    RequestError(String),

    /// Response bytes could not be decoded using the required text encoding.
    #[error("Failed to decode text response: {message}")]
    TextDecodingError {
        /// Decoder diagnostic.
        message: String,
    },

    /// A server-suggested automatic download filename failed safety validation.
    #[error("Unsafe download filename: {filename}")]
    UnsafeDownloadFilename {
        /// Rejected filename.
        filename: String,
    },

    /// A response exceeded the configured in-memory body limit.
    #[error(
        "Response body exceeded the configured limit of {limit_bytes} bytes after receiving {received_bytes} bytes"
    )]
    ResponseBodyTooLarge {
        /// Configured maximum body size.
        limit_bytes: u64,
        /// Declared or actually received body size at detection time.
        received_bytes: u64,
    },
}

impl From<reqwest::Error> for CallerError {
    fn from(err: reqwest::Error) -> Self {
        let kind = if err.is_timeout() {
            TransportErrorKind::Timeout
        } else if err.is_redirect() {
            TransportErrorKind::Redirect
        } else if err.is_connect() {
            TransportErrorKind::Connect
        } else if err.is_builder() {
            TransportErrorKind::Builder
        } else {
            TransportErrorKind::Request
        };
        CallerError::HttpTransport { kind, source: err }
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
        CallerError::Io {
            context: "performing I/O".to_string(),
            source: err,
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

impl CallerError {
    /// Creates a new configuration error
    pub fn config_error<S: Into<String>>(message: S) -> Self {
        CallerError::ConfigError(message.into())
    }

    /// Create a missing configuration file error.
    pub fn config_file_not_found<S: Into<String>>(path: S) -> Self {
        CallerError::ConfigFileNotFound { path: path.into() }
    }

    /// Create an unsupported configuration format error.
    pub fn unsupported_config_format<S: Into<String>>(path: S) -> Self {
        CallerError::UnsupportedConfigFormat { path: path.into() }
    }

    /// Create a configuration parser error with source context.
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

    /// Create a configuration serialization error.
    pub fn config_serialize_error<S1: Into<String>, S2: Into<String>>(
        format: S1,
        message: S2,
    ) -> Self {
        CallerError::ConfigSerializeError {
            format: format.into(),
            message: message.into(),
        }
    }

    /// Create a configuration watcher error.
    pub fn config_watch_error<S1: Into<String>, S2: Into<String>>(path: S1, message: S2) -> Self {
        CallerError::ConfigWatchError {
            path: path.into(),
            message: message.into(),
        }
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

    /// Create a poisoned-lock error naming the shared resource.
    pub fn lock_poisoned<S: Into<String>>(resource: S) -> Self {
        CallerError::LockPoisoned {
            resource: resource.into(),
        }
    }

    /// Wrap an I/O error while retaining the operation context and source.
    pub fn io<S: Into<String>>(context: S, source: std::io::Error) -> Self {
        CallerError::Io {
            context: context.into(),
            source,
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

    /// Create a response text decoding error.
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
            | CallerError::ParameterError(_)
            | CallerError::InvalidHeaderName { .. }
            | CallerError::InvalidHeaderValue { .. }
            | CallerError::InvalidUserAgent { .. }
            | CallerError::HttpMethodNotSupported { .. }
            | CallerError::MissingPathParameter { .. }
            | CallerError::InvalidUrl { .. }
            | CallerError::UnsupportedUrlScheme { .. }
            | CallerError::UnsupportedParamType { .. }
            | CallerError::TextDecodingError { .. }
            | CallerError::UnsafeDownloadFilename { .. }
            | CallerError::ResponseBodyTooLarge { .. } => ErrorCategory::Protocol,
            CallerError::AuthenticationError(_)
            | CallerError::MissingAuthEnvironmentVariable { .. }
            | CallerError::UnknownAuthProvider { .. }
            | CallerError::RequestError(_) => ErrorCategory::Security,
            CallerError::Io { .. }
            | CallerError::ServiceNotFound { .. }
            | CallerError::ApiNotFound { .. }
            | CallerError::HttpTransport { .. }
            | CallerError::HttpStatus { .. } => ErrorCategory::Runtime,
        }
    }

    /// Returns true if this is a configuration error
    pub fn is_config_error(&self) -> bool {
        self.category() == ErrorCategory::Config
    }

    /// Returns true if this is a network-related error
    pub fn is_network_error(&self) -> bool {
        matches!(self, CallerError::HttpTransport { .. })
    }

    /// Returns true if this is a method format error
    pub fn is_method_error(&self) -> bool {
        matches!(
            self,
            CallerError::InvalidMethodFormat { .. } | CallerError::HttpMethodNotSupported { .. }
        )
    }
}
