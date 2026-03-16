//! Configuration builder for dynamically creating and editing caller configurations

use crate::domain::{api_item::ApiItem, caller_config::CallerConfig, service_item::ServiceItem};
use crate::shared::error::CallerError;

/// Builder for creating Caller configurations
pub struct ConfigBuilder {
    config: CallerConfig,
}

impl ConfigBuilder {
    /// Create a new empty configuration builder
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
    pub fn add_service(&mut self, service: ServiceItem) -> &mut Self {
        self.config.service_items.push(service);
        self
    }

    /// Add or update a service by name
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
    pub fn remove_service(&mut self, name: &str) -> bool {
        let len = self.config.service_items.len();
        self.config.service_items.retain(|s| s.api_name != name);
        self.config.service_items.len() != len
    }

    /// List all service names
    pub fn list_services(&self) -> Vec<&str> {
        self.config.service_items.iter().map(|s| s.api_name.as_str()).collect()
    }

    /// Get a service by name
    pub fn get_service(&self, name: &str) -> Option<&ServiceItem> {
        self.config.service_items.iter().find(|s| s.api_name == name)
    }

    /// Add API to a service
    pub fn add_api(&mut self, service_name: &str, api: ApiItem) -> Result<&mut Self, CallerError> {
        let service = self.config.service_items.iter_mut()
            .find(|s| s.api_name == service_name)
            .ok_or_else(|| CallerError::ServiceNotFound(service_name.to_string()))?;
        service.api_items.push(api);
        Ok(self)
    }

    /// Build the configuration
    pub fn build(self) -> CallerConfig {
        self.config
    }

    /// Get reference to configuration
    pub fn as_config(&self) -> &CallerConfig {
        &self.config
    }

    /// Export to JSON string
    pub fn to_json(&self) -> Result<String, CallerError> {
        serde_json::to_string_pretty(&self.config)
            .map_err(|e| CallerError::JsonError(format!("Failed to serialize: {}", e)))
    }

    /// Export to YAML string
    pub fn to_yaml(&self) -> Result<String, CallerError> {
        serde_yaml::to_string(&self.config)
            .map_err(|e| CallerError::JsonError(format!("Failed to serialize: {}", e)))
    }

    /// Export to TOML string
    pub fn to_toml(&self) -> Result<String, CallerError> {
        toml::to_string_pretty(&self.config)
            .map_err(|e| CallerError::JsonError(format!("Failed to serialize: {}", e)))
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
            .ok_or_else(|| CallerError::ConfigError(format!("Unknown file format: {}", path)))?;
        
        let content = self.to_format(format)?;
        std::fs::write(path, content)
            .map_err(|e| CallerError::IoError(format!("Failed to write file: {}", e)))?;
        
        Ok(())
    }

    /// Load from file
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

/// Configuration file format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Json,
    Yaml,
    Toml,
}

impl ConfigFormat {
    /// Detect format from file path
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

    /// Get file extension
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
        }
    }

    /// Get all supported formats
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
    /// Set base URL
    pub fn base_url(mut self, url: &str) -> Self {
        self.service.base_url = url.to_string();
        self
    }

    /// Set authorization type
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
    pub fn api_full(mut self, api: ApiItem) -> Self {
        self.service.api_items.push(api);
        self
    }

    /// Build and add service to configuration
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
