//! Configuration builder for dynamically creating and editing caller configurations

use crate::config::ConfigFormat;
use crate::domain::{
    HttpMethod, ParamType, api_config::ApiConfig, caller_config::CallerConfig,
    service_config::ServiceConfig,
};
use crate::shared::error::CallerError;

/// Builder for creating Caller configurations
///
/// This is the recommended entry point for programmatic configuration because it
/// lets new code use typed protocol values such as [`HttpMethod`] and [`ParamType`]
/// while remaining compatible with the current serialized config format.
pub struct ConfigBuilder {
    config: CallerConfig,
}

impl ConfigBuilder {
    /// Create a new empty configuration builder.
    ///
    /// # Examples
    /// ```rust
    /// use caller::ConfigBuilder;
    ///
    /// let builder = ConfigBuilder::new();
    /// assert!(builder.as_config().service_items.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            config: CallerConfig {
                authorizations: vec![],
                service_items: vec![],
            },
        }
    }

    /// Create from existing configuration
    pub fn from_config(config: CallerConfig) -> Self {
        Self { config }
    }

    /// Add a service
    pub fn add_service(&mut self, service: ServiceConfig) -> &mut Self {
        self.config.service_items.push(service);
        self
    }

    /// Add or replace a service by name and return a chained [`ServiceBuilder`].
    pub fn service(&mut self, name: &str, base_url: &str) -> ServiceBuilder<'_> {
        // Remove existing if present
        self.config.service_items.retain(|s| s.api_name != name);

        ServiceBuilder {
            builder: self,
            service: ServiceConfig {
                api_name: name.to_string(),
                base_url: base_url.to_string(),
                authorization_type: None,
                timeout: None,
                api_items: vec![],
                use_new_http_client: None,
            },
        }
    }

    /// Get or create a service by name
    pub fn get_or_create_service(&mut self, name: &str) -> &mut ServiceConfig {
        if !self.config.service_items.iter().any(|s| s.api_name == name) {
            self.config.service_items.push(ServiceConfig {
                api_name: name.to_string(),
                base_url: String::new(),
                authorization_type: None,
                timeout: None,
                api_items: vec![],
                use_new_http_client: None,
            });
        }
        let index = self
            .config
            .service_items
            .iter()
            .position(|service| service.api_name == name)
            .expect("service was present or inserted immediately above");
        &mut self.config.service_items[index]
    }

    /// Remove a service by name
    pub fn remove_service(&mut self, name: &str) -> bool {
        let len = self.config.service_items.len();
        self.config.service_items.retain(|s| s.api_name != name);
        self.config.service_items.len() != len
    }

    /// List all service names
    pub fn list_services(&self) -> Vec<&str> {
        self.config
            .service_items
            .iter()
            .map(|s| s.api_name.as_str())
            .collect()
    }

    /// Get a service by name
    pub fn get_service(&self, name: &str) -> Option<&ServiceConfig> {
        self.config
            .service_items
            .iter()
            .find(|s| s.api_name == name)
    }

    /// Add API to a service
    pub fn add_api(
        &mut self,
        service_name: &str,
        api: ApiConfig,
    ) -> Result<&mut Self, CallerError> {
        let service = self
            .config
            .service_items
            .iter_mut()
            .find(|s| s.api_name == service_name)
            .ok_or_else(|| CallerError::service_not_found(service_name))?;
        service.api_items.push(api);
        Ok(self)
    }

    /// Consume the builder and return the assembled, unchecked [`CallerConfig`].
    ///
    /// Prefer [`Self::build_validated`] at trust boundaries. This method is
    /// retained for callers that assemble a configuration in multiple stages.
    pub fn build(self) -> CallerConfig {
        self.config
    }

    /// Validate and consume the builder.
    pub fn build_validated(self) -> Result<CallerConfig, CallerError> {
        self.config.validate()?;
        Ok(self.config)
    }

    /// Validate the currently assembled configuration.
    pub fn validate(&self) -> Result<(), CallerError> {
        self.config.validate()
    }

    /// Get reference to configuration
    pub fn as_config(&self) -> &CallerConfig {
        &self.config
    }

    /// Export to JSON string
    pub fn to_json(&self) -> Result<String, CallerError> {
        self.validate()?;
        serde_json::to_string_pretty(&self.config)
            .map_err(|e| CallerError::config_serialize_error("json", e.to_string()))
    }

    /// Export to YAML string
    pub fn to_yaml(&self) -> Result<String, CallerError> {
        self.validate()?;
        serde_yaml_ng::to_string(&self.config)
            .map_err(|e| CallerError::config_serialize_error("yaml", e.to_string()))
    }

    /// Export to TOML string
    pub fn to_toml(&self) -> Result<String, CallerError> {
        self.validate()?;
        toml::to_string_pretty(&self.config)
            .map_err(|e| CallerError::config_serialize_error("toml", e.to_string()))
    }

    /// Export to specified format
    pub fn to_format(&self, format: ConfigFormat) -> Result<String, CallerError> {
        match format {
            ConfigFormat::Json => self.to_json(),
            ConfigFormat::Yaml => self.to_yaml(),
            ConfigFormat::Toml => self.to_toml(),
        }
    }

    /// Save to file (format detected from extension)
    pub fn save(&self, path: &str) -> Result<(), CallerError> {
        let format = ConfigFormat::detect_from_path(path)
            .ok_or_else(|| CallerError::unsupported_config_format(path))?;

        let content = self.to_format(format)?;
        std::fs::write(path, content)
            .map_err(|error| CallerError::io(format!("writing config file '{path}'"), error))?;

        Ok(())
    }

    /// Load from file
    pub fn load(path: &str) -> Result<Self, CallerError> {
        let content = std::fs::read_to_string(path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                CallerError::config_file_not_found(path)
            } else {
                CallerError::io(format!("reading config file '{path}'"), error)
            }
        })?;

        let format = ConfigFormat::detect_from_path(path)
            .ok_or_else(|| CallerError::unsupported_config_format(path))?;

        let config: CallerConfig = match format {
            ConfigFormat::Json => serde_json::from_str(&content)
                .map_err(|e| CallerError::config_parse_error(path, "json", e.to_string()))?,
            ConfigFormat::Yaml => serde_yaml_ng::from_str(&content)
                .map_err(|e| CallerError::config_parse_error(path, "yaml", e.to_string()))?,
            ConfigFormat::Toml => toml::from_str(&content)
                .map_err(|e| CallerError::config_parse_error(path, "toml", e.to_string()))?,
        };

        config.validate()?;
        Ok(Self::from_config(config))
    }

    /// Convert configuration format
    pub fn convert(input_path: &str, output_path: &str) -> Result<(), CallerError> {
        let builder = Self::load(input_path)?;
        builder.save(output_path)
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for configuring a single service inside a [`ConfigBuilder`].
pub struct ServiceBuilder<'a> {
    builder: &'a mut ConfigBuilder,
    service: ServiceConfig,
}

/// Typed builder for configuring a single API endpoint.
///
/// This builder avoids raw protocol strings in new code and writes them back
/// into the serialized `ApiConfig` shape only when `build()` is called.
pub struct ApiEndpointBuilder<'a> {
    service_builder: ServiceBuilder<'a>,
    api: ApiConfig,
}

fn collect_param_types<I>(param_types: I) -> Vec<ParamType>
where
    I: IntoIterator<Item = ParamType>,
{
    let values: Vec<ParamType> = param_types.into_iter().collect();
    if values.is_empty() {
        vec![ParamType::None]
    } else {
        values
    }
}

impl<'a> ServiceBuilder<'a> {
    /// Set base URL
    pub fn base_url(mut self, url: &str) -> Self {
        self.service.base_url = url.to_string();
        self
    }

    /// Set auth_config type
    pub fn auth(mut self, auth_type: &str) -> Self {
        self.service.authorization_type = Some(auth_type.to_string());
        self
    }

    /// Set timeout
    pub fn timeout(mut self, timeout_ms: u32) -> Self {
        self.service.timeout = Some(timeout_ms);
        self
    }

    /// Add an API endpoint
    pub fn api(
        mut self,
        method: &str,
        url: &str,
        http_method: &str,
        param_type: &str,
    ) -> Result<Self, CallerError> {
        self.service.api_items.push(ApiConfig {
            method: method.to_string(),
            url: url.to_string(),
            http_method: HttpMethod::parse(http_method)?,
            param_type: ApiConfig::parse_param_types(param_type)?,
            description: None,
            need_cache: None,
            cache_time: None,
            content_type: None,
            authorization_type: None,
            timeout: None,
            use_new_http_client: None,
        });
        Ok(self)
    }

    /// Add an API endpoint using typed HTTP method and parameter kinds.
    ///
    /// # Examples
    /// ```rust
    /// use caller::{ConfigBuilder, HttpMethod, ParamType};
    ///
    /// let mut builder = ConfigBuilder::new();
    /// builder
    ///     .service("JP", "https://example.com")
    ///     .api_typed("list", "/posts", HttpMethod::Get, [ParamType::Query])
    ///     .build();
    ///
    /// let config = builder.build();
    /// assert_eq!(config.service_items[0].api_items[0].http_method, HttpMethod::Get);
    /// ```
    pub fn api_typed<I>(
        self,
        method: &str,
        url: &str,
        http_method: HttpMethod,
        param_types: I,
    ) -> Self
    where
        I: IntoIterator<Item = ParamType>,
    {
        let mut service_builder = self;
        service_builder.service.api_items.push(ApiConfig {
            method: method.to_string(),
            url: url.to_string(),
            http_method,
            param_type: collect_param_types(param_types),
            description: None,
            need_cache: None,
            cache_time: None,
            content_type: None,
            authorization_type: None,
            timeout: None,
            use_new_http_client: None,
        });
        service_builder
    }

    /// Start a typed builder for a single API endpoint.
    ///
    /// This is the most flexible programmatic configuration path.
    pub fn api_endpoint(self, method: &str, url: &str) -> ApiEndpointBuilder<'a> {
        ApiEndpointBuilder {
            service_builder: self,
            api: ApiConfig {
                method: method.to_string(),
                url: url.to_string(),
                http_method: HttpMethod::Get,
                param_type: vec![ParamType::None],
                description: None,
                need_cache: None,
                cache_time: None,
                content_type: None,
                authorization_type: None,
                timeout: None,
                use_new_http_client: None,
            },
        }
    }

    /// Add an API endpoint with full details
    pub fn api_full(mut self, api: ApiConfig) -> Self {
        self.service.api_items.push(api);
        self
    }

    /// Build and add service to configuration
    pub fn build(self) -> &'a mut ConfigBuilder {
        self.builder.config.service_items.push(self.service);
        self.builder
    }
}

impl<'a> ApiEndpointBuilder<'a> {
    /// Set the endpoint HTTP method.
    pub fn http_method(mut self, http_method: HttpMethod) -> Self {
        self.api.http_method = http_method;
        self
    }

    /// Set a single parameter kind such as [`ParamType::Query`].
    pub fn param_type(mut self, param_type: ParamType) -> Self {
        self.api.param_type = vec![param_type];
        self
    }

    /// Set one or more parameter kinds such as `path + json`.
    pub fn param_types<I>(mut self, param_types: I) -> Self
    where
        I: IntoIterator<Item = ParamType>,
    {
        self.api.param_type = collect_param_types(param_types);
        self
    }

    /// Set the human-readable endpoint description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.api.description = Some(description.into());
        self
    }

    /// Override the request content type metadata.
    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.api.content_type = Some(content_type.into());
        self
    }

    /// Override the service-level authentication type for this endpoint.
    pub fn auth(mut self, auth_type: impl Into<String>) -> Self {
        self.api.authorization_type = Some(auth_type.into());
        self
    }

    /// Override the timeout for this endpoint in milliseconds.
    pub fn timeout(mut self, timeout_ms: u32) -> Self {
        self.api.timeout = Some(timeout_ms);
        self
    }

    /// Finalize the endpoint and return to the parent [`ServiceBuilder`].
    pub fn build(mut self) -> ServiceBuilder<'a> {
        self.service_builder.service.api_items.push(self.api);
        self.service_builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let mut builder = ConfigBuilder::new();

        builder
            .service("JP", "https://jsonplaceholder.typicode.com")
            .timeout(30000)
            .api("list", "/posts", "GET", "query")
            .unwrap()
            .api("get", "/posts/{id}", "GET", "path")
            .unwrap()
            .build();

        let services = builder.list_services();
        assert_eq!(services, vec!["JP"]);

        let config = builder.build();
        assert_eq!(config.service_items.len(), 1);
        assert_eq!(config.service_items[0].api_items.len(), 2);
    }

    #[test]
    fn test_format_detection() {
        assert_eq!(
            ConfigFormat::detect_from_path("config.json"),
            Some(ConfigFormat::Json)
        );
        assert_eq!(
            ConfigFormat::detect_from_path("config.yaml"),
            Some(ConfigFormat::Yaml)
        );
        assert_eq!(
            ConfigFormat::detect_from_path("config.yml"),
            Some(ConfigFormat::Yaml)
        );
        assert_eq!(
            ConfigFormat::detect_from_path("config.toml"),
            Some(ConfigFormat::Toml)
        );
        assert_eq!(ConfigFormat::detect_from_path("config.txt"), None);
    }

    #[test]
    fn test_export_formats() {
        let mut builder = ConfigBuilder::new();
        builder
            .service("Test", "https://api.example.com")
            .api("ping", "/ping", "GET", "none")
            .unwrap()
            .build();

        let json = builder.to_json().unwrap();
        assert!(json.contains("\"api_name\": \"Test\""));

        let yaml = builder.to_yaml().unwrap();
        assert!(yaml.contains("api_name: Test"));

        let toml = builder.to_toml().unwrap();
        assert!(toml.contains("api_name = \"Test\""));
    }

    #[test]
    fn test_typed_service_builder_api() {
        let mut builder = ConfigBuilder::new();
        builder
            .service("Typed", "https://api.example.com")
            .api_typed(
                "list",
                "/items",
                HttpMethod::Get,
                [ParamType::Query, ParamType::Json],
            )
            .build();

        let config = builder.build();
        let api = &config.service_items[0].api_items[0];
        assert_eq!(api.http_method, HttpMethod::Get);
        assert_eq!(api.param_types_as_str(), "query,json");
    }

    #[test]
    fn test_typed_api_endpoint_builder() {
        let mut builder = ConfigBuilder::new();
        builder
            .service("Typed", "https://api.example.com")
            .api_endpoint("create", "/items/{id}")
            .http_method(HttpMethod::Post)
            .param_types([ParamType::Path, ParamType::Json])
            .description("Create an item")
            .content_type("application/json")
            .auth("token")
            .timeout(5_000)
            .build()
            .build();

        let config = builder.build();
        let api = &config.service_items[0].api_items[0];
        assert_eq!(api.http_method, HttpMethod::Post);
        assert_eq!(api.param_types_as_str(), "path,json");
        assert_eq!(api.description.as_deref(), Some("Create an item"));
        assert_eq!(api.content_type.as_deref(), Some("application/json"));
        assert_eq!(api.authorization_type.as_deref(), Some("token"));
        assert_eq!(api.timeout, Some(5_000));
    }
}
