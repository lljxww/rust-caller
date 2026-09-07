use crate::core::constants;
use crate::core::context::{
    split_method, substitute_path_parameters, validate_exact_path_parameters,
    validate_path_parameters,
};
use crate::domain::auth_trait::{AuthContext, AuthProvider, Authenticator};
use crate::domain::service_config::join_base_and_endpoint;
use crate::domain::{
    ApiConfig, ApiResult, CallerConfig, DownloadResult, Middleware, MiddlewareChain, ParamType,
    RequestContext, ResponseContext, RetryConfig, ServiceConfig,
};
use crate::params::{CallParams, RequestArgs};
use crate::shared::error::CallerError;
use reqwest::{Method, header};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_MAX_RESPONSE_BODY_BYTES: u64 = 16 * 1024 * 1024;
const DEFAULT_MAX_DOWNLOAD_BYTES: u64 = 256 * 1024 * 1024;

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
    middlewares: MiddlewareChain,
    max_response_body_bytes: u64,
    max_download_bytes: u64,
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
    middlewares: MiddlewareChain,
    max_response_body_bytes: u64,
    max_download_bytes: u64,
}

struct InstanceContext {
    service_name: String,
    api_name: String,
    api_config: ApiConfig,
    http_method: Method,
    url: String,
    path_params: Option<HashMap<String, String>>,
    query_params: Option<Vec<(String, String)>>,
    form_params: Option<Vec<(String, String)>>,
    params: Option<HashMap<String, String>>,
    json_body: Option<serde_json::Value>,
    auth_type: Option<String>,
    service_timeout: Option<u32>,
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
        validate_auth_provider_name(name)?;
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
        validate_auth_provider_name(name)?;
        let provider = AuthProvider::from_closure(f);
        let mut providers = self
            .auth_providers
            .write()
            .map_err(|_| CallerError::lock_poisoned("caller auth registry"))?;
        providers.insert(name.to_string(), provider);
        Ok(())
    }

    /// Check whether an auth provider exists in this instance registry.
    pub fn has_auth(&self, name: &str) -> Result<bool, CallerError> {
        Ok(self
            .auth_providers
            .read()
            .map_err(|_| CallerError::lock_poisoned("caller auth registry"))?
            .contains_key(name))
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
        let context = self.build_context(method, params, None)?;
        self.execute_call(context).await
    }

    /// Execute a configured API call while preserving JSON parameter types.
    pub async fn call_params(
        &self,
        method: &str,
        params: Option<CallParams>,
    ) -> Result<ApiResult, CallerError> {
        let string_params = params.as_ref().map(CallParams::to_hashmap);
        let typed_params = params.as_ref().map(CallParams::to_json).transpose()?;
        let context = self.build_context(method, string_params, typed_params)?;
        self.execute_call(context).await
    }

    /// Execute a configured API call with parameters separated by transport location.
    ///
    /// Prefer this method for combined endpoint types such as `path,json` or
    /// `path,query`. Legacy `call` and `call_params` apply one parameter map to
    /// every configured parameter kind for compatibility.
    pub async fn call_args(
        &self,
        method: &str,
        args: RequestArgs,
    ) -> Result<ApiResult, CallerError> {
        let context = self.build_args_context(method, args)?;
        self.execute_call(context).await
    }

    async fn execute_call(&self, context: InstanceContext) -> Result<ApiResult, CallerError> {
        self.execute_call_attempt(&context).await
    }

    /// Execute a configured API call with retry behavior.
    pub async fn call_with_retry(
        &self,
        method: &str,
        params: Option<HashMap<String, String>>,
        retry_config: RetryConfig,
    ) -> Result<ApiResult, CallerError> {
        let context = self.build_context(method, params, None)?;
        self.execute_call_with_retry(context, retry_config).await
    }

    /// Execute a type-preserving configured API call with retry behavior.
    pub async fn call_params_with_retry(
        &self,
        method: &str,
        params: Option<CallParams>,
        retry_config: RetryConfig,
    ) -> Result<ApiResult, CallerError> {
        let string_params = params.as_ref().map(CallParams::to_hashmap);
        let typed_params = params.as_ref().map(CallParams::to_json).transpose()?;
        let context = self.build_context(method, string_params, typed_params)?;
        self.execute_call_with_retry(context, retry_config).await
    }

    /// Execute a separated-argument call with retry behavior.
    pub async fn call_args_with_retry(
        &self,
        method: &str,
        args: RequestArgs,
        retry_config: RetryConfig,
    ) -> Result<ApiResult, CallerError> {
        let context = self.build_args_context(method, args)?;
        self.execute_call_with_retry(context, retry_config).await
    }

    async fn execute_call_with_retry(
        &self,
        context: InstanceContext,
        retry_config: RetryConfig,
    ) -> Result<ApiResult, CallerError> {
        retry_config.validate()?;
        let mut attempt = 0u32;
        let mut next_delay = Duration::ZERO;

        loop {
            if attempt > 0 {
                tokio::time::sleep(next_delay).await;
            }

            match self.execute_call_attempt(&context).await {
                Ok(result) => {
                    let status = result.status_code.as_u16();

                    if retry_config.should_retry_status(status)
                        && attempt < retry_config.max_retries
                    {
                        next_delay =
                            retry_config.calculate_response_delay(&result.headers, attempt);
                        attempt += 1;
                        continue;
                    }

                    return Ok(result);
                }
                Err(error) => {
                    if retry_config.retry_on_network_error
                        && error.is_network_error()
                        && attempt < retry_config.max_retries
                    {
                        next_delay = retry_config.calculate_delay(attempt);
                        attempt += 1;
                        continue;
                    }
                    return Err(error);
                }
            }
        }
    }

    /// Execute a configured API call and collect the response as downloadable bytes.
    pub async fn download(
        &self,
        method: &str,
        params: Option<HashMap<String, String>>,
        extension: Option<String>,
    ) -> Result<DownloadResult, CallerError> {
        let context = self.build_context(method, params, None)?;
        self.execute_download(context, extension).await
    }

    /// Download a configured response while accepting type-preserving parameters.
    pub async fn download_params(
        &self,
        method: &str,
        params: Option<CallParams>,
        extension: Option<String>,
    ) -> Result<DownloadResult, CallerError> {
        let string_params = params.as_ref().map(CallParams::to_hashmap);
        let typed_params = params.as_ref().map(CallParams::to_json).transpose()?;
        let context = self.build_context(method, string_params, typed_params)?;
        self.execute_download(context, extension).await
    }

    /// Download a configured response with parameters separated by transport location.
    pub async fn download_args(
        &self,
        method: &str,
        args: RequestArgs,
        extension: Option<String>,
    ) -> Result<DownloadResult, CallerError> {
        let context = self.build_args_context(method, args)?;
        self.execute_download(context, extension).await
    }

    async fn execute_download(
        &self,
        context: InstanceContext,
        extension: Option<String>,
    ) -> Result<DownloadResult, CallerError> {
        let (rb, request_context, _) = self.prepare_request(&context).await?;

        let response = match rb.send().await {
            Ok(response) => response,
            Err(error) => {
                let error = CallerError::from(error);
                self.middlewares.on_error(&error, &request_context).await;
                return Err(error);
            }
        };
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
        let content = match read_limited_body(response, self.max_download_bytes).await {
            Ok(content) => content,
            Err(error) => {
                self.middlewares.on_error(&error, &request_context).await;
                return Err(error);
            }
        };

        let mut download_result = DownloadResult::from_response(status_code, content, content_type);

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

    async fn execute_call_attempt(
        &self,
        context: &InstanceContext,
    ) -> Result<ApiResult, CallerError> {
        let (request, request_context, started_at) = self.prepare_request(context).await?;
        let response = match request.send().await {
            Ok(response) => response,
            Err(error) => {
                let error = CallerError::from(error);
                self.middlewares.on_error(&error, &request_context).await;
                return Err(error);
            }
        };

        let status_code = response.status();
        let response_headers = response.headers().clone();
        let body = match read_limited_body(response, self.max_response_body_bytes).await {
            Ok(body) => String::from_utf8_lossy(&body).into_owned(),
            Err(error) => {
                self.middlewares.on_error(&error, &request_context).await;
                return Err(error);
            }
        };

        let mut response_context = ResponseContext::new(
            status_code.as_u16(),
            body,
            request_context,
            duration_millis(started_at.elapsed()),
        );
        response_context.headers = response_headers;
        if let Err(error) = self.middlewares.after_response(&mut response_context).await {
            self.middlewares
                .on_error(&error, &response_context.request)
                .await;
            return Err(error);
        }

        let status_code = reqwest::StatusCode::from_u16(response_context.status_code)
            .map_err(|error| CallerError::parameter_error(error.to_string()))?;
        Ok(ApiResult::build_with_metadata(
            response_context.body,
            status_code,
            response_context.headers,
            Duration::from_millis(response_context.duration_ms),
        ))
    }

    async fn prepare_request(
        &self,
        context: &InstanceContext,
    ) -> Result<(reqwest::RequestBuilder, RequestContext, Instant), CallerError> {
        let mut request_context = RequestContext::new(context.http_method.as_str(), &context.url);
        request_context.params = context.params.clone();
        request_context.path_params = context.path_params.clone();
        request_context.query_params = context.query_params.clone();
        request_context.form_params = context.form_params.clone();
        if context.api_config.has_param_type(ParamType::Json)? {
            request_context.body = context
                .json_body
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?;
        }

        if let Err(error) = self.middlewares.before_request(&mut request_context).await {
            self.middlewares.on_error(&error, &request_context).await;
            return Err(error);
        }

        let started_at = Instant::now();
        let mut request = match self.base_request(context, &request_context) {
            Ok(request) => request,
            Err(error) => {
                self.middlewares.on_error(&error, &request_context).await;
                return Err(error);
            }
        };
        request = match self.apply_params(request, context, &request_context) {
            Ok(request) => request,
            Err(error) => {
                self.middlewares.on_error(&error, &request_context).await;
                return Err(error);
            }
        };
        request = match self.apply_auth(request, context, &request_context).await {
            Ok(request) => request,
            Err(error) => {
                self.middlewares.on_error(&error, &request_context).await;
                return Err(error);
            }
        };

        Ok((request, request_context, started_at))
    }

    fn build_context(
        &self,
        method: &str,
        params: Option<HashMap<String, String>>,
        typed_params: Option<serde_json::Value>,
    ) -> Result<InstanceContext, CallerError> {
        let split_method = split_method(method)?;
        let service_name = &split_method[0];
        let api_name = &split_method[1];

        let (service_config, api_config, base_url) =
            self.get_config_with_base_url(service_name, api_name)?;
        let mut url = join_base_and_endpoint(&base_url, &api_config.url);
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
        let json_body = if api_config.has_param_type(ParamType::Json)? {
            if let Some(typed_params) = typed_params {
                Some(typed_params)
            } else {
                params.as_ref().map(serde_json::to_value).transpose()?
            }
        } else {
            None
        };

        Ok(InstanceContext {
            service_name: service_name.to_string(),
            api_name: api_name.to_string(),
            api_config,
            http_method,
            url,
            path_params: None,
            query_params: None,
            form_params: None,
            params,
            json_body,
            auth_type,
            service_timeout: service_config.timeout,
        })
    }

    fn build_args_context(
        &self,
        method: &str,
        args: RequestArgs,
    ) -> Result<InstanceContext, CallerError> {
        let split_method = split_method(method)?;
        let service_name = &split_method[0];
        let api_name = &split_method[1];
        let (service_config, api_config, base_url) =
            self.get_config_with_base_url(service_name, api_name)?;

        validate_request_args(&api_config, &args)?;

        let path_params = non_empty_scalar_map(args.path_params())?;
        let query_params = collect_scalar_pairs(args.query_params(), args.query_pairs())?;
        let form_params = collect_scalar_pairs(args.form_params(), args.form_pairs())?;
        let mut url = join_base_and_endpoint(&base_url, &api_config.url);
        if api_config.has_param_type(ParamType::Path)? {
            let params = path_params.as_ref().ok_or_else(|| {
                CallerError::parameter_error("Path parameters are required for this API")
            })?;
            validate_exact_path_parameters(&url, params.keys().map(String::as_str).collect())?;
            url = substitute_path_parameters(&url, params)?;
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
            path_params,
            query_params,
            form_params,
            params: None,
            json_body: args.json_body().cloned(),
            auth_type,
            service_timeout: service_config.timeout,
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
        request_context: &RequestContext,
    ) -> Result<reqwest::RequestBuilder, CallerError> {
        let timeout = context
            .api_config
            .timeout
            .or(context.service_timeout)
            .map(|timeout_ms| Duration::from_millis(timeout_ms as u64))
            .unwrap_or(self.default_timeout);
        let method = Method::from_bytes(request_context.method.as_bytes())
            .map_err(|_| CallerError::http_method_not_supported(&request_context.method))?;
        let mut builder = self
            .client
            .request(method, &request_context.url)
            .timeout(timeout);

        if !self.default_headers.is_empty() {
            builder = builder.headers(self.default_headers.clone());
        }

        if let Some(user_agent) = &self.user_agent {
            builder = builder.header(header::USER_AGENT, user_agent.clone());
        } else if !self.default_headers.contains_key(header::USER_AGENT) {
            builder = builder.header(header::USER_AGENT, constants::UA);
        }

        if let Some(content_type) = &context.api_config.content_type {
            let content_type = header::HeaderValue::from_str(content_type).map_err(|error| {
                CallerError::InvalidHeaderValue {
                    name: header::CONTENT_TYPE.as_str().to_string(),
                    message: error.to_string(),
                }
            })?;
            builder = builder.header(header::CONTENT_TYPE, content_type);
        }

        if !request_context.headers.is_empty() {
            builder = builder.headers(request_context.headers.clone());
        }

        if context.api_config.has_param_type(ParamType::Json)?
            && !self.default_headers.contains_key(header::CONTENT_TYPE)
            && !request_context.headers.contains_key(header::CONTENT_TYPE)
            && context.api_config.content_type.is_none()
        {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
        }

        Ok(builder)
    }

    fn apply_params(
        &self,
        mut rb: reqwest::RequestBuilder,
        context: &InstanceContext,
        request_context: &RequestContext,
    ) -> Result<reqwest::RequestBuilder, CallerError> {
        for param_type in context.api_config.param_types()? {
            match param_type {
                ParamType::Query => {
                    if let Some(params) = &request_context.query_params {
                        rb = rb.query(params);
                    } else if let Some(params) = &request_context.params {
                        rb = rb.query(params);
                    }
                }
                ParamType::Json => {
                    if let Some(body) = &request_context.body {
                        rb = rb.body(body.clone());
                    }
                }
                ParamType::Form => {
                    if let Some(params) = &request_context.form_params {
                        rb = rb.form(params);
                    } else if let Some(params) = &request_context.params {
                        rb = rb.form(params);
                    }
                }
                ParamType::Path | ParamType::None => {}
            }
        }

        Ok(rb)
    }

    async fn apply_auth(
        &self,
        builder: reqwest::RequestBuilder,
        context: &InstanceContext,
        request_context: &RequestContext,
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
                request_context.url.clone(),
                context.api_config.http_method.as_str().to_string(),
                merged_auth_params(request_context),
                auth_type.clone(),
            );

            return provider.apply(builder, &auth_context).await;
        }

        Ok(builder)
    }
}

fn non_empty_scalar_map(
    params: &CallParams,
) -> Result<Option<HashMap<String, String>>, CallerError> {
    if params.is_empty() {
        Ok(None)
    } else {
        Ok(Some(params.to_scalar_hashmap()?))
    }
}

fn collect_scalar_pairs(
    params: &CallParams,
    extra: &[(String, String)],
) -> Result<Option<Vec<(String, String)>>, CallerError> {
    let mut values: Vec<(String, String)> = params.to_scalar_hashmap()?.into_iter().collect();
    values.extend_from_slice(extra);
    Ok((!values.is_empty()).then_some(values))
}

fn validate_auth_provider_name(name: &str) -> Result<(), CallerError> {
    if name.trim().is_empty() || name.trim() != name {
        return Err(CallerError::config_error(
            "Authentication provider name must be non-empty and cannot have surrounding whitespace",
        ));
    }
    Ok(())
}

fn validate_request_args(api: &ApiConfig, args: &RequestArgs) -> Result<(), CallerError> {
    let checks = [
        (ParamType::Path, !args.path_params().is_empty()),
        (
            ParamType::Query,
            !args.query_params().is_empty() || !args.query_pairs().is_empty(),
        ),
        (
            ParamType::Form,
            !args.form_params().is_empty() || !args.form_pairs().is_empty(),
        ),
        (ParamType::Json, args.json_body().is_some()),
    ];

    for (param_type, supplied) in checks {
        if supplied && !api.has_param_type(param_type)? {
            return Err(CallerError::parameter_error(format!(
                "Endpoint '{}' does not accept {} parameters",
                api.method,
                param_type.as_str()
            )));
        }
    }

    Ok(())
}

fn merged_auth_params(request: &RequestContext) -> Option<HashMap<String, String>> {
    let mut merged = HashMap::new();
    for params in [request.params.as_ref(), request.path_params.as_ref()]
        .into_iter()
        .flatten()
    {
        merged.extend(params.clone());
    }
    for params in [request.query_params.as_ref(), request.form_params.as_ref()]
        .into_iter()
        .flatten()
    {
        merged.extend(params.iter().cloned());
    }
    (!merged.is_empty()).then_some(merged)
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
            middlewares: MiddlewareChain::new(),
            max_response_body_bytes: DEFAULT_MAX_RESPONSE_BODY_BYTES,
            max_download_bytes: DEFAULT_MAX_DOWNLOAD_BYTES,
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

    /// Add middleware to the request execution chain.
    pub fn middleware<M>(mut self, middleware: M) -> Self
    where
        M: Middleware + 'static,
    {
        self.middlewares = self.middlewares.with(middleware);
        self
    }

    /// Add shared middleware to the request execution chain.
    pub fn middleware_arc(mut self, middleware: Arc<dyn Middleware>) -> Self {
        self.middlewares = self.middlewares.with_arc(middleware);
        self
    }

    /// Replace the request execution middleware chain.
    pub fn middleware_chain(mut self, middlewares: MiddlewareChain) -> Self {
        self.middlewares = middlewares;
        self
    }

    /// Set the maximum buffered body size for ordinary API responses.
    pub fn max_response_body_bytes(mut self, max_bytes: u64) -> Self {
        self.max_response_body_bytes = max_bytes;
        self
    }

    /// Set the maximum buffered body size for downloads.
    pub fn max_download_bytes(mut self, max_bytes: u64) -> Self {
        self.max_download_bytes = max_bytes;
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
        for name in self.auth_providers.keys() {
            validate_auth_provider_name(name)?;
        }
        if self.default_timeout == Some(Duration::ZERO) {
            return Err(CallerError::config_error(
                "Caller default timeout must be greater than zero",
            ));
        }

        let client = match self.client {
            Some(client) => client,
            None => reqwest::Client::builder()
                .build()
                .map_err(CallerError::from)?,
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
            middlewares: self.middlewares,
            max_response_body_bytes: self.max_response_body_bytes,
            max_download_bytes: self.max_download_bytes,
        })
    }
}

impl Default for CallerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_filename_from_disposition(disposition: &str) -> Option<String> {
    let mut regular_filename = None;
    let mut extended_filename = None;

    for parameter in disposition.split(';').skip(1) {
        let Some((name, value)) = parameter.trim().split_once('=') else {
            continue;
        };
        let value = value.trim();

        if name.trim().eq_ignore_ascii_case("filename*") {
            let mut parts = value.splitn(3, '\'');
            let charset = parts.next().unwrap_or_default();
            let _language = parts.next();
            let encoded = parts.next();
            if (charset.is_empty() || charset.eq_ignore_ascii_case("utf-8"))
                && let Some(encoded) = encoded
                && let Ok(decoded) = percent_decode(encoded)
            {
                extended_filename = Some(decoded);
            }
        } else if name.trim().eq_ignore_ascii_case("filename") {
            regular_filename = parse_quoted_header_value(value);
        }
    }

    extended_filename.or(regular_filename)
}

fn parse_quoted_header_value(value: &str) -> Option<String> {
    if let Some(quoted) = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    {
        let mut parsed = String::with_capacity(quoted.len());
        let mut escaped = false;
        for character in quoted.chars() {
            if escaped {
                parsed.push(character);
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else {
                parsed.push(character);
            }
        }
        if escaped {
            return None;
        }
        Some(parsed)
    } else {
        (!value.is_empty()).then(|| value.to_string())
    }
}

fn duration_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

async fn read_limited_body(
    mut response: reqwest::Response,
    limit_bytes: u64,
) -> Result<Vec<u8>, CallerError> {
    if let Some(content_length) = response.content_length()
        && content_length > limit_bytes
    {
        return Err(CallerError::ResponseBodyTooLarge {
            limit_bytes,
            received_bytes: content_length,
        });
    }

    let initial_capacity = response
        .content_length()
        .unwrap_or(0)
        .min(limit_bytes)
        .min(usize::MAX as u64) as usize;
    let mut body = Vec::with_capacity(initial_capacity);

    while let Some(chunk) = response.chunk().await? {
        let received_bytes = (body.len() as u64).saturating_add(chunk.len() as u64);
        if received_bytes > limit_bytes {
            return Err(CallerError::ResponseBodyTooLarge {
                limit_bytes,
                received_bytes,
            });
        }
        body.extend_from_slice(&chunk);
    }

    Ok(body)
}

fn percent_decode(s: &str) -> Result<String, std::string::FromUtf8Error> {
    let input = s.as_bytes();
    let mut bytes = Vec::with_capacity(input.len());
    let mut i = 0;

    while i < input.len() {
        if input[i] == b'%'
            && i + 2 < input.len()
            && let Ok(hex) = std::str::from_utf8(&input[i + 1..i + 3])
            && let Ok(byte) = u8::from_str_radix(hex, 16)
        {
            bytes.push(byte);
            i += 3;
            continue;
        }
        bytes.push(input[i]);
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

    #[test]
    fn content_disposition_prefers_utf8_extended_filename() {
        let disposition =
            "attachment; FILENAME=legacy.txt; filename*=UTF-8''report%20%E4%B8%AD%E6%96%87.pdf";
        assert_eq!(
            extract_filename_from_disposition(disposition).as_deref(),
            Some("report 中文.pdf")
        );
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
        assert!(caller.has_auth("token").unwrap());
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
