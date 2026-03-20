//! Configuration builder for dynamically creating and editing caller configurations

use crate::domain::{api_item::ApiItem, caller_config::CallerConfig, service_item::ServiceItem};
use crate::shared::error::CallerError;

/// Builder for creating Caller configurations
pub struct ConfigBuilder {
    config: CallerConfig,
}

impl ConfigBuilder {
    /// Create a new empty configuration builder
    /// 
    /// # Returns
    /// A new ConfigBuilder instance with no services or authorizations
    /// 
    /// # Example
    /// ```
    /// use caller::ConfigBuilder;
    /// 
    /// let builder = ConfigBuilder::new();
    /// assert_eq!(builder.list_services().len(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            config: CallerConfig {
                authorizations: vec![],
                service_items: vec![],
            },
        }
    }

    /// Create a builder from an existing CallerConfig
    /// 
    /// # Arguments
    /// * `config` - An existing CallerConfig instance
    /// 
    /// # Returns
    /// A new ConfigBuilder instance initialized with the provided config
    /// 
    /// # Example
    /// ```
    /// use caller::{ConfigBuilder, CallerConfig};
    /// 
    /// let config = CallerConfig {
    ///     authorizations: vec![],
    ///     service_items: vec![],
    /// };
    /// let builder = ConfigBuilder::from_config(config);
    /// ```
    pub fn from_config(config: CallerConfig) -> Self {
        Self { config }
    }

    /// Add a service to the configuration
    /// 
    /// # Arguments
    /// * `service` - ServiceItem to add
    /// 
    /// # Returns
    /// Mutable reference to self for method chaining
    pub fn add_service(&mut self, service: ServiceItem) -> &mut Self {
        self.config.service_items.push(service);
        self
    }

    /// Add or update a service by name (returns a ServiceBuilder for chaining)
    /// 
    /// If a service with the same name exists, it will be replaced.
    /// This is the recommended way to add services as it provides a fluent API.
    /// 
    /// # Arguments
    /// * `name` - Service name
    /// * `base_url` - Base URL for the service
    /// 
    /// # Returns
    /// A ServiceBuilder for configuring the service
    /// 
    /// # Example
    /// ```
    /// use caller::ConfigBuilder;
    /// 
    /// let mut builder = ConfigBuilder::new();
    /// builder
    ///     .service("api", "https://api.example.com")
    ///     .timeout(30000)
    ///     .api("list", "/items", "GET", "query")
    ///     .build();
    /// ```
    pub fn service(&mut self, name: &str, base_url: &str) -> ServiceBuilder<'_> {
        // Remove existing if present
        self.config.service_items.retain(|s| s.api_name != name);
        
        ServiceBuilder {
            builder: self,
            service: ServiceItem {
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
    /// 
    /// If the service doesn't exist, it will be created with an empty base URL.
    /// 
    /// # Arguments
    /// * `name` - Service name
    /// 
    /// # Returns
    /// Mutable reference to the ServiceItem
    /// 
    /// # Example
    /// ```
    /// use caller::ConfigBuilder;
    /// 
    /// let mut builder = ConfigBuilder::new();
    /// let service = builder.get_or_create_service("api");
    /// service.base_url = "https://api.example.com".to_string();
    /// ```
    pub fn get_or_create_service(&mut self, name: &str) -> &mut ServiceItem {
        if !self.config.service_items.iter().any(|s| s.api_name == name) {
            self.config.service_items.push(ServiceItem {
                api_name: name.to_string(),
                base_url: String::new(),
                authorization_type: None,
                timeout: None,
                api_items: vec![],
                use_new_http_client: None,
            });
        }
        self.config.service_items.iter_mut().find(|s| s.api_name == name).unwrap()
    }

    /// Remove a service by name
    /// 
    /// # Arguments
    /// * `name` - Service name to remove
    /// 
    /// # Returns
    /// true if the service was removed, false if it didn't exist
    pub fn remove_service(&mut self, name: &str) -> bool {
        let len = self.config.service_items.len();
        self.config.service_items.retain(|s| s.api_name != name);
        self.config.service_items.len() != len
    }

    /// List all service names
    /// 
    /// # Returns
    /// A vector of service name strings
    pub fn list_services(&self) -> Vec<&str> {
        self.config.service_items.iter().map(|s| s.api_name.as_str()).collect()
    }

    /// Get a service by name
    /// 
    /// # Arguments
    /// * `name` - Service name to look up
    /// 
    /// # Returns
    /// Some(&ServiceItem) if found, None otherwise
    pub fn get_service(&self, name: &str) -> Option<&ServiceItem> {
        self.config.service_items.iter().find(|s| s.api_name == name)
    }

    /// Add an API endpoint to a service
    /// 
    /// # Arguments
    /// * `service_name` - Name of the service to add the API to
    /// * `api` - ApiItem to add
    /// 
    /// # Returns
    /// Ok(&mut Self) if successful, Err if service not found
    /// 
    /// # Example
    /// ```
    /// # use caller::{ConfigBuilder, ApiItem};
    /// # let mut builder = ConfigBuilder::new();
    /// # builder.service("api", "https://api.example.com").build();
    /// # let api = ApiItem {
    /// #     method: "list".to_string(),
    /// #     url: "/items".to_string(),
    /// #     http_method: "GET".to_string(),
    /// #     param_type: "query".to_string(),
    /// #     description: Some("List all items".to_string()),
    /// #     need_cache: None,
    /// #     cache_time: None,
    /// #     content_type: None,
    /// #     authorization_type: None,
    /// #     timeout: None,
    /// #     use_new_http_client: None,
    /// # };
    /// # builder.add_api("api", api).unwrap();
    /// ```
    pub fn add_api(&mut self, service_name: &str, api: ApiItem) -> Result<&mut Self, CallerError> {
        let service = self.config.service_items.iter_mut()
            .find(|s| s.api_name == service_name)
            .ok_or_else(|| CallerError::ServiceNotFound(service_name.to_string()))?;
        service.api_items.push(api);
        Ok(self)
    }

    /// Build and return the CallerConfig
    /// 
    /// # Returns
    /// The constructed CallerConfig instance
    pub fn build(self) -> CallerConfig {
        self.config
    }

    /// Get a reference to the current configuration
    /// 
    /// # Returns
    /// Reference to the CallerConfig
    pub fn as_config(&self) -> &CallerConfig {
        &self.config
    }

    /// Export configuration to JSON string
    /// 
    /// # Returns
    /// Ok(JSON string) if successful, Err otherwise
    pub fn to_json(&self) -> Result<String, CallerError> {
        serde_json::to_string_pretty(&self.config)
            .map_err(|e| CallerError::JsonError(format!("Failed to serialize: {}", e)))
    }

    /// Export configuration to YAML string
    /// 
    /// # Returns
    /// Ok(YAML string) if successful, Err otherwise
    pub fn to_yaml(&self) -> Result<String, CallerError> {
        serde_yaml::to_string(&self.config)
            .map_err(|e| CallerError::JsonError(format!("Failed to serialize: {}", e)))
    }

    /// Export configuration to TOML string
    /// 
    /// # Returns
    /// Ok(TOML string) if successful, Err otherwise
    pub fn to_toml(&self) -> Result<String, CallerError> {
        toml::to_string_pretty(&self.config)
            .map_err(|e| CallerError::JsonError(format!("Failed to serialize: {}", e)))
    }

    /// Export configuration to the specified format
    /// 
    /// # Arguments
    /// * `format` - ConfigFormat to export to
    /// 
    /// # Returns
    /// Ok(string representation) if successful, Err otherwise
    pub fn to_format(&self, format: ConfigFormat) -> Result<String, CallerError> {
        match format {
            ConfigFormat::Json => self.to_json(),
            ConfigFormat::Yaml => self.to_yaml(),
            ConfigFormat::Toml => self.to_toml(),
        }
    }

    /// Save configuration to a file
    /// 
    /// The file format is automatically detected from the file extension
    /// (.json, .yaml, .yml, .toml)
    /// 
    /// # Arguments
    /// * `path` - File path to save to
    /// 
    /// # Returns
    /// Ok(()) if successful, Err otherwise
    /// 
    /// # Example
    /// ```no_run
    /// use caller::ConfigBuilder;
    /// 
    /// let mut builder = ConfigBuilder::new();
    /// builder
    ///     .service("api", "https://api.example.com")
    ///     .api("ping", "/ping", "GET", "none")
    ///     .build();
    /// 
    /// builder.save("config.json").unwrap();
    /// ```
    pub fn save(&self, path: &str) -> Result<(), CallerError> {
        let format = ConfigFormat::detect_from_path(path)
            .ok_or_else(|| CallerError::ConfigError(format!("Unknown file format: {}", path)))?;
        
        let content = self.to_format(format)?;
        std::fs::write(path, content)
            .map_err(|e| CallerError::IoError(format!("Failed to write file: {}", e)))?;
        
        Ok(())
    }

    /// Load configuration from a file
    /// 
    /// The file format is automatically detected from the file extension
    /// (.json, .yaml, .yml, .toml)
    /// 
    /// # Arguments
    /// * `path` - File path to load from
    /// 
    /// # Returns
    /// Ok(ConfigBuilder) if successful, Err otherwise
    /// 
    /// # Example
    /// ```no_run
    /// use caller::ConfigBuilder;
    /// 
    /// let builder = ConfigBuilder::load("config.json").unwrap();
    /// let services = builder.list_services();
    /// ```
    pub fn load(path: &str) -> Result<Self, CallerError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| CallerError::IoError(format!("Failed to read file: {}", e)))?;
        
        let format = ConfigFormat::detect_from_path(path)
            .ok_or_else(|| CallerError::ConfigError(format!("Unknown file format: {}", path)))?;
        
        let config: CallerConfig = match format {
            ConfigFormat::Json => serde_json::from_str(&content)
                .map_err(|e| CallerError::JsonError(format!("Invalid JSON: {}", e)))?,
            ConfigFormat::Yaml => serde_yaml::from_str(&content)
                .map_err(|e| CallerError::JsonError(format!("Invalid YAML: {}", e)))?,
            ConfigFormat::Toml => toml::from_str(&content)
                .map_err(|e| CallerError::JsonError(format!("Invalid TOML: {}", e)))?,
        };
        
        Ok(Self::from_config(config))
    }

    /// Convert configuration from one format to another
    /// 
    /// This is a convenience method for loading and saving in a single operation.
    /// 
    /// # Arguments
    /// * `input_path` - Path to the input configuration file
    /// * `output_path` - Path to save the converted configuration
    /// 
    /// # Returns
    /// Ok(()) if successful, Err otherwise
    /// 
    /// # Example
    /// ```no_run
    /// use caller::ConfigBuilder;
    /// 
    /// // Convert JSON to YAML
    /// ConfigBuilder::convert("config.json", "config.yaml").unwrap();
    /// ```
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

/// Configuration file format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Json,
    Yaml,
    Toml,
}

impl ConfigFormat {
    /// Detect configuration format from file path extension
    /// 
    /// # Arguments
    /// * `path` - File path to analyze
    /// 
    /// # Returns
    /// Some(ConfigFormat) if the extension is recognized, None otherwise
    /// 
    /// # Supported Extensions
    /// - `.json` → ConfigFormat::Json
    /// - `.yaml`, `.yml` → ConfigFormat::Yaml
    /// - `.toml` → ConfigFormat::Toml
    /// 
    /// # Example
    /// ```
    /// use caller::ConfigFormat;
    /// 
    /// assert_eq!(ConfigFormat::detect_from_path("config.json"), Some(ConfigFormat::Json));
    /// assert_eq!(ConfigFormat::detect_from_path("config.yaml"), Some(ConfigFormat::Yaml));
    /// assert_eq!(ConfigFormat::detect_from_path("config.toml"), Some(ConfigFormat::Toml));
    /// ```
    pub fn detect_from_path(path: &str) -> Option<Self> {
        let path_lower = path.to_lowercase();
        if path_lower.ends_with(".json") {
            Some(Self::Json)
        } else if path_lower.ends_with(".yaml") || path_lower.ends_with(".yml") {
            Some(Self::Yaml)
        } else if path_lower.ends_with(".toml") {
            Some(Self::Toml)
        } else {
            None
        }
    }

    /// Get the file extension for this format
    /// 
    /// # Returns
    /// A static string slice with the file extension (without dot)
    /// 
    /// # Example
    /// ```
    /// use caller::ConfigFormat;
    /// 
    /// assert_eq!(ConfigFormat::Json.extension(), "json");
    /// assert_eq!(ConfigFormat::Yaml.extension(), "yaml");
    /// assert_eq!(ConfigFormat::Toml.extension(), "toml");
    /// ```
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
        }
    }

    /// Get all supported configuration formats
    /// 
    /// # Returns
    /// A slice containing all supported formats
    /// 
    /// # Example
    /// ```
    /// use caller::ConfigFormat;
    /// 
    /// let formats = ConfigFormat::all();
    /// assert_eq!(formats.len(), 3);
    /// ```
    pub fn all() -> &'static [Self] {
        &[Self::Json, Self::Yaml, Self::Toml]
    }
}

/// Builder for creating service items
pub struct ServiceBuilder<'a> {
    builder: &'a mut ConfigBuilder,
    service: ServiceItem,
}

impl<'a> ServiceBuilder<'a> {
    /// Set the base URL for the service
    /// 
    /// # Arguments
    /// * `url` - Base URL for all API endpoints in this service
    /// 
    /// # Returns
    /// Self for method chaining
    /// 
    /// # Example
    /// ```
    /// use caller::ConfigBuilder;
    /// 
    /// let mut builder = ConfigBuilder::new();
    /// builder
    ///     .service("api", "https://example.com")
    ///     .base_url("https://api.example.com")
    ///     .build();
    /// ```
    pub fn base_url(mut self, url: &str) -> Self {
        self.service.base_url = url.to_string();
        self
    }

    /// Set the authorization type for the service
    /// 
    /// # Arguments
    /// * `auth_type` - Name of the authorization type (must match a registered auth provider)
    /// 
    /// # Returns
    /// Self for method chaining
    /// 
    /// # Example
    /// ```
    /// use caller::ConfigBuilder;
    /// 
    /// let mut builder = ConfigBuilder::new();
    /// builder
    ///     .service("api", "https://api.example.com")
    ///     .auth("bearer")
    ///     .build();
    /// ```
    pub fn auth(mut self, auth_type: &str) -> Self {
        self.service.authorization_type = Some(auth_type.to_string());
        self
    }

    /// Set the timeout for the service (in milliseconds)
    /// 
    /// # Arguments
    /// * `timeout_ms` - Timeout in milliseconds
    /// 
    /// # Returns
    /// Self for method chaining
    /// 
    /// # Example
    /// ```
    /// use caller::ConfigBuilder;
    /// 
    /// let mut builder = ConfigBuilder::new();
    /// builder
    ///     .service("api", "https://api.example.com")
    ///     .timeout(30000)  // 30 seconds
    ///     .build();
    /// ```
    pub fn timeout(mut self, timeout_ms: u32) -> Self {
        self.service.timeout = Some(timeout_ms);
        self
    }

    /// Add an API endpoint to the service
    /// 
    /// # Arguments
    /// * `method` - Method name (used in call API as "service.method")
    /// * `url` - URL path (can contain path parameters like "/users/{id}")
    /// * `http_method` - HTTP method (GET, POST, PUT, DELETE, PATCH)
    /// * `param_type` - Parameter type (query, path, json, form, none, or comma-separated)
    /// 
    /// # Returns
    /// Self for method chaining
    /// 
    /// # Example
    /// ```
    /// use caller::ConfigBuilder;
    /// 
    /// let mut builder = ConfigBuilder::new();
    /// builder
    ///     .service("api", "https://api.example.com")
    ///     .api("list", "/items", "GET", "query")
    ///     .api("get", "/items/{id}", "GET", "path")
    ///     .api("create", "/items", "POST", "json")
    ///     .build();
    /// ```
    pub fn api(mut self, method: &str, url: &str, http_method: &str, param_type: &str) -> Self {
        self.service.api_items.push(ApiItem {
            method: method.to_string(),
            url: url.to_string(),
            http_method: http_method.to_string(),
            param_type: param_type.to_string(),
            description: None,
            need_cache: None,
            cache_time: None,
            content_type: None,
            authorization_type: None,
            timeout: None,
            use_new_http_client: None,
        });
        self
    }

    /// Add an API endpoint with full details
    /// 
    /// Use this when you need to set additional properties like description,
    /// cache settings, or content type.
    /// 
    /// # Arguments
    /// * `api` - Complete ApiItem with all fields configured
    /// 
    /// # Returns
    /// Self for method chaining
    /// 
    /// # Example
    /// ```
    /// # use caller::{ConfigBuilder, ApiItem};
    /// # let mut builder = ConfigBuilder::new();
    /// # let api = ApiItem {
    /// #     method: "list".to_string(),
    /// #     url: "/items".to_string(),
    /// #     http_method: "GET".to_string(),
    /// #     param_type: "query".to_string(),
    /// #     description: Some("List all items".to_string()),
    /// #     need_cache: None,
    /// #     cache_time: None,
    /// #     content_type: None,
    /// #     authorization_type: None,
    /// #     timeout: None,
    /// #     use_new_http_client: None,
    /// # };
    /// # builder
    /// #     .service("api", "https://api.example.com")
    /// #     .api_full(api)
    /// #     .build();
    /// ```
    pub fn api_full(mut self, api: ApiItem) -> Self {
        self.service.api_items.push(api);
        self
    }

    /// Build and add the service to the configuration
    /// 
    /// This finalizes the service configuration and adds it to the builder.
    /// 
    /// # Returns
    /// Mutable reference to the parent ConfigBuilder
    /// 
    /// # Example
    /// ```
    /// use caller::ConfigBuilder;
    /// 
    /// let mut builder = ConfigBuilder::new();
    /// builder
    ///     .service("api", "https://api.example.com")
    ///     .timeout(30000)
    ///     .api("ping", "/ping", "GET", "none")
    ///     .build();  // Adds service to configuration and returns builder
    /// 
    /// // Can chain another service
    /// builder
    ///     .service("api2", "https://api2.example.com")
    ///     .build();
    /// ```
    pub fn build(self) -> &'a mut ConfigBuilder {
        self.builder.config.service_items.push(self.service);
        self.builder
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
            .api("get", "/posts/{id}", "GET", "path")
            .build();
        
        let services = builder.list_services();
        assert_eq!(services, vec!["JP"]);
        
        let config = builder.build();
        assert_eq!(config.service_items.len(), 1);
        assert_eq!(config.service_items[0].api_items.len(), 2);
    }

    #[test]
    fn test_format_detection() {
        assert_eq!(ConfigFormat::detect_from_path("config.json"), Some(ConfigFormat::Json));
        assert_eq!(ConfigFormat::detect_from_path("config.yaml"), Some(ConfigFormat::Yaml));
        assert_eq!(ConfigFormat::detect_from_path("config.yml"), Some(ConfigFormat::Yaml));
        assert_eq!(ConfigFormat::detect_from_path("config.toml"), Some(ConfigFormat::Toml));
        assert_eq!(ConfigFormat::detect_from_path("config.txt"), None);
    }

    #[test]
    fn test_export_formats() {
        let mut builder = ConfigBuilder::new();
        builder
            .service("Test", "https://api.example.com")
            .api("ping", "/ping", "GET", "none")
            .build();
        
        let json = builder.to_json().unwrap();
        assert!(json.contains("\"ApiName\": \"Test\""));
        
        let yaml = builder.to_yaml().unwrap();
        assert!(yaml.contains("ApiName: Test"));
        
        let toml = builder.to_toml().unwrap();
        assert!(toml.contains("ApiName = \"Test\""));
    }
}
