use crate::core::constants;
use crate::core::context::{split_method, substitute_path_parameters, validate_path_parameters};
use crate::domain::{
    ApiConfig, ApiResult, AuthContext, AuthProvider, Authenticator, CallerConfig, DownloadResult,
    ParamType, RetryConfig, ServiceConfig,
};
use crate::shared::error::CallerError;
use reqwest::{Method, header};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;

const DEFAULT_TIMEOUT_MS: u64 = 30_000;

/// Instance-based caller client.
///
/// This is the preferred API for new code because it keeps configuration,
/// authentication providers, and the HTTP client scoped to a concrete instance.
///
/// Compared with the crate-root global functions such as [`crate::call`]:
///
/// - each instance owns its own config snapshot
/// - each instance owns its own auth provider registry
/// - each instance reuses its own `reqwest::Client`
/// - config reload is explicit and instance-local
///
/// # Examples
/// ```no_run
/// use caller::Caller;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), caller::CallerError> {
/// let caller = Caller::from_path("caller.json")?;
/// let result = caller.call("JP.list", None).await?;
/// println!("{}", result.status_code);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Caller {
    config: Arc<RwLock<CallerConfig>>,
    auth_providers: Arc<RwLock<HashMap<String, AuthProvider>>>,
    client: reqwest::Client,
    config_path: Option<PathBuf>,
    default_timeout: Duration,
    default_headers: header::HeaderMap,
    user_agent: Option<header::HeaderValue>,
}

/// Builder for constructing a [`Caller`] instance.
///
/// Use this when you want instance-local defaults such as timeout, headers,
/// user-agent, or pre-registered auth providers.
///
/// # Examples
/// ```no_run
/// use caller::{Caller, HttpMethod, ParamType, NoAuth};
/// use std::collections::HashMap;
/// use std::time::Duration;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), caller::CallerError> {
/// let mut builder = caller::ConfigBuilder::new();
/// builder
///     .service("JP", "https://example.com")
///     .api_typed("list", "/posts", HttpMethod::Get, [ParamType::Query])
///     .build();
/// let config = builder.build();
///
/// let caller = Caller::builder()
///     .config(config)
///     .timeout(Duration::from_secs(5))
///     .auth("demo", NoAuth)
///     .build()?;
///
/// let _ = caller.call("JP.list", Some(HashMap::new())).await;
/// # Ok(())
/// # }
/// ```
pub struct CallerBuilder {
    config: Option<CallerConfig>,
    config_path: Option<PathBuf>,
    client: Option<reqwest::Client>,
    default_timeout: Option<Duration>,
    default_headers: header::HeaderMap,
    user_agent: Option<header::HeaderValue>,
    auth_providers: HashMap<String, AuthProvider>,
}

struct InstanceContext {
    service_name: String,
    api_name: String,
    api_config: ApiConfig,
    http_method: Method,
    url: String,
    params: Option<HashMap<String, String>>,
    auth_type: Option<String>,
}

impl Caller {
    /// Start constructing a [`Caller`] with [`CallerBuilder`].
    pub fn builder() -> CallerBuilder {
        CallerBuilder::new()
    }

    /// Build a [`Caller`] directly from an in-memory [`CallerConfig`].
    ///
    /// The config is validated before the instance is created.
    pub fn from_config(config: CallerConfig) -> Result<Self, CallerError> {
        Self::builder().config(config).build()
    }

    /// Build a [`Caller`] by loading config from a file path.
    ///
    /// Unlike the crate-root global helpers, this can load JSON, YAML, or TOML
    /// from any supported path.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, CallerError> {
        Self::builder().config_path(path).build()
    }

    /// Return a cloned snapshot of the instance configuration.
    pub fn config(&self) -> Result<CallerConfig, CallerError> {
        Ok(self
            .config
            .read()
            .map_err(|_| CallerError::lock_poisoned("caller config"))?
            .clone())
    }

    /// Reload configuration from this instance's `config_path`.
    ///
    /// Returns an error if the instance was not created from a file path.
    pub fn reload_config(&self) -> Result<(), CallerError> {
        let path = self
            .config_path
            .as_ref()
            .ok_or(CallerError::MissingCallerConfigPath)?;
        let config = crate::config::config_loader::ConfigLoader::load_config_from_path(
            path.to_string_lossy().as_ref(),
        )?;
        let mut config_guard = self
            .config
            .write()
            .map_err(|_| CallerError::lock_poisoned("caller config"))?;
        *config_guard = config;
        Ok(())
    }

    /// Register an auth provider on this instance only.
    ///
    /// This does not touch the crate-root global auth registry.
    pub fn register_auth(
        &self,
        name: &str,
        auth: impl Authenticator + 'static,
    ) -> Result<(), CallerError> {
        let provider = AuthProvider::from_trait(auth);
        let mut providers = self
            .auth_providers
            .write()
            .map_err(|_| CallerError::lock_poisoned("caller auth registry"))?;
        providers.insert(name.to_string(), provider);
        Ok(())
    }

    /// Register a closure-based auth provider on this instance only.
    pub fn register_auth_closure<F, Fut>(&self, name: &str, f: F) -> Result<(), CallerError>
    where
        F: Fn(reqwest::RequestBuilder, &AuthContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<reqwest::RequestBuilder, CallerError>>
            + Send
            + 'static,
    {
        let provider = AuthProvider::from_closure(f);
        let mut providers = self
            .auth_providers
            .write()
            .map_err(|_| CallerError::lock_poisoned("caller auth registry"))?;
        providers.insert(name.to_string(), provider);
        Ok(())
    }

    /// Check whether an auth provider exists in this instance registry.
    pub fn has_auth(&self, name: &str) -> bool {
        self.auth_providers
            .read()
            .map(|providers| providers.contains_key(name))
            .unwrap_or(false)
    }

    /// Remove an auth provider from this instance registry.
    pub fn remove_auth(&self, name: &str) -> Result<bool, CallerError> {
        let mut providers = self
            .auth_providers
            .write()
            .map_err(|_| CallerError::lock_poisoned("caller auth registry"))?;
        Ok(providers.remove(name).is_some())
    }

    /// Remove all auth providers from this instance registry.
    pub fn clear_auth(&self) -> Result<(), CallerError> {
        let mut providers = self
            .auth_providers
            .write()
            .map_err(|_| CallerError::lock_poisoned("caller auth registry"))?;
        providers.clear();
        Ok(())
    }

    pub(crate) fn replace_auth_providers(
        &self,
        providers: HashMap<String, AuthProvider>,
    ) -> Result<(), CallerError> {
        let mut auth_providers = self
            .auth_providers
            .write()
            .map_err(|_| CallerError::lock_poisoned("caller auth registry"))?;
        *auth_providers = providers;
        Ok(())
    }

    /// Execute a configured API call.
    ///
    /// `method` must use the `service.api` format.
    pub async fn call(
        &self,
        method: &str,
        params: Option<HashMap<String, String>>,
    ) -> Result<ApiResult, CallerError> {
        let context = self.build_context(method, params)?;
        let mut rb = self.base_request(&context)?;
        rb = self.apply_params(rb, &context)?;
        rb = self.apply_auth(rb, &context).await?;

        let response = rb.send().await?;
        let status_code = response.status();
        let result = response.text().await?;
        ApiResult::build(result, status_code)
    }

    /// Execute a configured API call with retry behavior.
    pub async fn call_with_retry(
        &self,
        method: &str,
        params: Option<HashMap<String, String>>,
        retry_config: RetryConfig,
    ) -> Result<ApiResult, CallerError> {
        let context = self.build_context(method, params)?;
        let mut last_error: Option<CallerError> = None;
        let mut attempt = 0u32;

        while attempt <= retry_config.max_retries {
            if attempt > 0 {
                tokio::time::sleep(retry_config.calculate_delay(attempt - 1)).await;
            }

            let mut rb = self.base_request(&context)?;
            rb = self.apply_params(rb, &context)?;
            rb = self.apply_auth(rb, &context).await?;

            match rb.send().await {
                Ok(response) => {
                    let status_code = response.status();
                    let status = status_code.as_u16();

                    if retry_config.should_retry_status(status)
                        && attempt < retry_config.max_retries
                    {
                        last_error = Some(CallerError::RetryableHttpStatus {
                            status,
                            attempt: attempt + 1,
                            max_retries: retry_config.max_retries,
                        });
                        attempt += 1;
                        continue;
                    }

                    let result = response.text().await?;
                    return ApiResult::build(result, status_code);
                }
                Err(e) => {
                    if retry_config.retry_on_network_error && attempt < retry_config.max_retries {
                        last_error = Some(CallerError::from(e));
                        attempt += 1;
                        continue;
                    }
                    return Err(CallerError::from(e));
                }
            }
        }

        Err(last_error.unwrap_or(CallerError::RetryAttemptsExhausted))
    }

    /// Execute a configured API call and collect the response as downloadable bytes.
    pub async fn download(
        &self,
        method: &str,
        params: Option<HashMap<String, String>>,
        extension: Option<String>,
    ) -> Result<DownloadResult, CallerError> {
        let context = self.build_context(method, params)?;
        let mut rb = self.base_request(&context)?;
        rb = self.apply_params(rb, &context)?;
        rb = self.apply_auth(rb, &context).await?;

        let response = rb.send().await?;
        let status_code = response.status();
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let content_disposition = response
            .headers()
            .get(header::CONTENT_DISPOSITION)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let content = response.bytes().await?.to_vec();

        let mut download_result =
            DownloadResult::from_response(status_code, content, content_type)?;

        if let Some(ext) = extension {
            download_result = download_result.with_extension(&ext);
        }

        if let Some(disposition) = content_disposition
            && let Some(filename) = extract_filename_from_disposition(&disposition)
        {
            download_result = download_result.with_filename(filename);
        }

        Ok(download_result)
    }

    fn build_context(
        &self,
        method: &str,
        params: Option<HashMap<String, String>>,
    ) -> Result<InstanceContext, CallerError> {
        let split_method = split_method(method)?;
        let service_name = &split_method[0];
        let api_name = &split_method[1];

        let (service_config, api_config, base_url) =
            self.get_config_with_base_url(service_name, api_name)?;
        let mut url = format!("{}{}", base_url, api_config.url);
        if api_config.has_param_type(ParamType::Path)? {
            let params_map = params.as_ref().ok_or_else(|| {
                CallerError::parameter_error("Path parameters are required for this API")
            })?;
            validate_path_parameters(&url, params_map.keys().map(|s| s.as_str()).collect())?;
            url = substitute_path_parameters(&url, params_map)?;
        }

        let http_method = api_config.http_method()?.as_reqwest_method();
        let auth_type = api_config
            .authorization_type
            .clone()
            .or_else(|| service_config.authorization_type.clone());

        Ok(InstanceContext {
            service_name: service_name.to_string(),
            api_name: api_name.to_string(),
            api_config,
            http_method,
            url,
            params,
            auth_type,
        })
    }

    fn get_config_with_base_url(
        &self,
        service_name: &str,
        api_name: &str,
    ) -> Result<(ServiceConfig, ApiConfig, String), CallerError> {
        let config = self
            .config
            .read()
            .map_err(|_| CallerError::lock_poisoned("caller config"))?;

        let service_config = config
            .service_items
            .iter()
            .find(|s| s.api_name == service_name)
            .ok_or_else(|| CallerError::service_not_found(service_name))?;

        let api_config = service_config
            .api_items
            .iter()
            .find(|a| a.method == api_name)
            .ok_or_else(|| CallerError::api_not_found(service_name, api_name))?;

        Ok((
            service_config.clone(),
            api_config.clone(),
            service_config.base_url.clone(),
        ))
    }

    fn base_request(
        &self,
        context: &InstanceContext,
    ) -> Result<reqwest::RequestBuilder, CallerError> {
        let timeout = context
            .api_config
            .timeout
            .map(|timeout_ms| Duration::from_millis(timeout_ms as u64))
            .unwrap_or(self.default_timeout);
        let mut builder = self
            .client
            .request(context.http_method.clone(), &context.url)
            .timeout(timeout);

        if !self.default_headers.is_empty() {
            builder = builder.headers(self.default_headers.clone());
        }

        if let Some(user_agent) = &self.user_agent {
            builder = builder.header(header::USER_AGENT, user_agent.clone());
        } else if !self.default_headers.contains_key(header::USER_AGENT) {
            builder = builder.header(header::USER_AGENT, constants::UA);
        }

        if !self.default_headers.contains_key(header::CONTENT_TYPE) {
            builder = builder.header(header::CONTENT_TYPE, constants::DEFAULT_CONTENT_TYPE);
        }

        Ok(builder)
    }

    fn apply_params(
        &self,
        mut rb: reqwest::RequestBuilder,
        context: &InstanceContext,
    ) -> Result<reqwest::RequestBuilder, CallerError> {
        if let Some(params_map) = &context.params {
            for param_type in context.api_config.param_types()? {
                match param_type {
                    ParamType::Query => rb = rb.query(params_map),
                    ParamType::Json => rb = rb.json(params_map),
                    ParamType::Form => rb = rb.form(params_map),
                    ParamType::Path | ParamType::None => {}
                }
            }
        }

        Ok(rb)
    }

    async fn apply_auth(
        &self,
        builder: reqwest::RequestBuilder,
        context: &InstanceContext,
    ) -> Result<reqwest::RequestBuilder, CallerError> {
        if let Some(auth_type) = &context.auth_type {
            let provider = self
                .auth_providers
                .read()
                .map_err(|_| CallerError::lock_poisoned("caller auth registry"))?
                .get(auth_type)
                .cloned()
                .ok_or_else(|| CallerError::unknown_auth_provider(auth_type))?;

            let auth_context = AuthContext::new(
                context.service_name.clone(),
                context.api_name.clone(),
                context.url.clone(),
                context.api_config.http_method.as_str().to_string(),
                context.params.clone(),
                auth_type.clone(),
            );

            return provider.apply(builder, &auth_context).await;
        }

        Ok(builder)
    }
}

impl CallerBuilder {
    /// Create a new empty builder.
    pub fn new() -> Self {
        Self {
            config: None,
            config_path: None,
            client: None,
            default_timeout: None,
            default_headers: header::HeaderMap::new(),
            user_agent: None,
            auth_providers: HashMap::new(),
        }
    }

    /// Set the in-memory configuration for the future [`Caller`].
    pub fn config(mut self, config: CallerConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the config file path for the future [`Caller`].
    pub fn config_path(mut self, path: impl AsRef<Path>) -> Self {
        self.config_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Inject a prebuilt `reqwest::Client`.
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Set the default request timeout for APIs that do not override timeout in config.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = Some(timeout);
        self
    }

    /// Add a default header applied to every request from this instance.
    pub fn default_header(
        mut self,
        key: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Result<Self, CallerError> {
        let header_name = header::HeaderName::from_bytes(key.as_ref().as_bytes()).map_err(|e| {
            CallerError::InvalidHeaderName {
                name: key.as_ref().to_string(),
                message: e.to_string(),
            }
        })?;
        let header_value = header::HeaderValue::from_str(value.as_ref()).map_err(|e| {
            CallerError::InvalidHeaderValue {
                name: key.as_ref().to_string(),
                message: e.to_string(),
            }
        })?;
        self.default_headers.insert(header_name, header_value);
        Ok(self)
    }

    /// Set the default `User-Agent` header for this instance.
    pub fn user_agent(mut self, value: impl AsRef<str>) -> Result<Self, CallerError> {
        let user_agent = header::HeaderValue::from_str(value.as_ref()).map_err(|e| {
            CallerError::InvalidUserAgent {
                value: value.as_ref().to_string(),
                message: e.to_string(),
            }
        })?;
        self.user_agent = Some(user_agent);
        Ok(self)
    }

    /// Pre-register an instance-local auth provider during construction.
    pub fn auth(mut self, name: &str, auth: impl Authenticator + 'static) -> Self {
        self.auth_providers
            .insert(name.to_string(), AuthProvider::from_trait(auth));
        self
    }

    /// Pre-register an instance-local closure-based auth provider during construction.
    pub fn auth_closure<F, Fut>(mut self, name: &str, f: F) -> Self
    where
        F: Fn(reqwest::RequestBuilder, &AuthContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<reqwest::RequestBuilder, CallerError>>
            + Send
            + 'static,
    {
        self.auth_providers
            .insert(name.to_string(), AuthProvider::from_closure(f));
        self
    }

    /// Validate configuration and build the final [`Caller`].
    pub fn build(self) -> Result<Caller, CallerError> {
        let config = match (self.config, self.config_path.as_ref()) {
            (Some(config), _) => config,
            (None, Some(path)) => {
                crate::config::config_loader::ConfigLoader::load_config_from_path(
                    path.to_string_lossy().as_ref(),
                )?
            }
            (None, None) => return Err(CallerError::MissingCallerBuilderConfig),
        };
        config.validate()?;

        let client = match self.client {
            Some(client) => client,
            None => reqwest::Client::builder().build().map_err(|e| {
                CallerError::HttpClientBuildError {
                    message: e.to_string(),
                }
            })?,
        };

        Ok(Caller {
            config: Arc::new(RwLock::new(config)),
            auth_providers: Arc::new(RwLock::new(self.auth_providers)),
            client,
            config_path: self.config_path,
            default_timeout: self
                .default_timeout
                .unwrap_or_else(|| Duration::from_millis(DEFAULT_TIMEOUT_MS)),
            default_headers: self.default_headers,
            user_agent: self.user_agent,
        })
    }
}

impl Default for CallerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_filename_from_disposition(disposition: &str) -> Option<String> {
    if let Some(start) = disposition.find("filename=") {
        let rest = &disposition[start + 9..];

        if let Some(stripped) = rest.strip_prefix('"') {
            if let Some(end) = stripped.find('"') {
                return Some(stripped[..end].to_string());
            }
        } else {
            let end = rest.find(';').unwrap_or(rest.len());
            return Some(rest[..end].trim().to_string());
        }
    }

    if let Some(start) = disposition.find("filename*=") {
        let rest = &disposition[start + 10..];

        if let Some(prefix_end) = rest.find('\'')
            && let Some(encoding_end) = rest[prefix_end + 1..].find('\'')
        {
            let encoded = &rest[prefix_end + encoding_end + 2..];
            if let Ok(decoded) = percent_decode(encoded) {
                return Some(decoded);
            }
        }
    }

    None
}

fn percent_decode(s: &str) -> Result<String, std::string::FromUtf8Error> {
    let mut bytes = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '%' && i + 2 < chars.len() {
            let hex = format!("{}{}", chars[i + 1], chars[i + 2]);
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                bytes.push(byte);
                i += 3;
                continue;
            }
        }
        bytes.push(chars[i] as u8);
        i += 1;
    }

    String::from_utf8(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HttpMethod;
    use crate::NoAuth;

    fn make_config(base_url: &str, auth_type: Option<&str>) -> CallerConfig {
        CallerConfig {
            authorizations: vec![],
            service_items: vec![ServiceConfig {
                api_name: "Svc".to_string(),
                authorization_type: auth_type.map(str::to_string),
                base_url: base_url.to_string(),
                timeout: Some(50),
                api_items: vec![ApiConfig {
                    method: "list".to_string(),
                    url: "/items".to_string(),
                    http_method: HttpMethod::Get,
                    param_type: vec![ParamType::None],
                    description: None,
                    need_cache: None,
                    cache_time: None,
                    content_type: None,
                    authorization_type: None,
                    timeout: Some(50),
                    use_new_http_client: None,
                }],
                use_new_http_client: None,
            }],
        }
    }

    #[test]
    fn test_caller_instances_keep_separate_configs() {
        let caller_a = Caller::from_config(make_config("https://a.example.com", None)).unwrap();
        let caller_b = Caller::from_config(make_config("https://b.example.com", None)).unwrap();

        let config_a = caller_a.config().unwrap();
        let config_b = caller_b.config().unwrap();

        assert_eq!(config_a.service_items[0].base_url, "https://a.example.com");
        assert_eq!(config_b.service_items[0].base_url, "https://b.example.com");
    }

    #[tokio::test]
    async fn test_caller_auth_isolation() {
        let caller_with_auth =
            Caller::from_config(make_config("http://127.0.0.1:9", Some("token"))).unwrap();
        let caller_without_auth =
            Caller::from_config(make_config("http://127.0.0.1:9", Some("token"))).unwrap();

        caller_with_auth.register_auth("token", NoAuth).unwrap();

        let err = caller_without_auth
            .call("Svc.list", None)
            .await
            .expect_err("caller without auth should fail");
        assert!(matches!(err, CallerError::UnknownAuthProvider { .. }));

        let err = caller_with_auth
            .call("Svc.list", None)
            .await
            .expect_err("caller with auth should reach network layer");
        assert!(err.is_network_error());
    }

    #[test]
    fn test_builder_supports_instance_defaults_and_auth() {
        let caller = Caller::builder()
            .config(make_config("https://api.example.com", Some("token")))
            .timeout(Duration::from_secs(2))
            .default_header("x-trace-id", "trace-123")
            .unwrap()
            .user_agent("caller-test/1.0")
            .unwrap()
            .auth("token", NoAuth)
            .build()
            .unwrap();

        assert_eq!(caller.default_timeout, Duration::from_secs(2));
        assert_eq!(
            caller.default_headers.get("x-trace-id").unwrap(),
            "trace-123"
        );
        assert_eq!(
            caller.user_agent.as_ref().unwrap(),
            &header::HeaderValue::from_static("caller-test/1.0")
        );
        assert!(caller.has_auth("token"));
    }

    #[test]
    fn test_builder_rejects_invalid_header_configuration() {
        let err = Caller::builder()
            .config(make_config("https://api.example.com", None))
            .default_header("bad header", "value")
            .err()
            .expect("invalid header name should fail");
        assert!(matches!(err, CallerError::InvalidHeaderName { .. }));

        let err = Caller::builder()
            .config(make_config("https://api.example.com", None))
            .user_agent("bad\r\nagent")
            .err()
            .expect("invalid user-agent should fail");
        assert!(matches!(err, CallerError::InvalidUserAgent { .. }));
    }

    #[test]
    fn test_from_config_rejects_invalid_api_shape() {
        let mut config = make_config("https://api.example.com", None);
        config.service_items[0].api_items[0].param_type = vec![ParamType::None, ParamType::Query];

        let err = Caller::from_config(config)
            .err()
            .expect("invalid config should fail fast");
        assert!(matches!(err, CallerError::UnsupportedParamType { .. }));
    }
}
