//! OpenAPI generator from Caller configuration

use super::types::*;
use crate::config::config_loader::ConfigLoader;
use crate::domain::{api_item::ApiItem, caller_config::CallerConfig, service_item::ServiceItem};
use crate::shared::error::CallerError;
use std::collections::HashMap;

/// Generator for OpenAPI specifications from Caller configuration
pub struct OpenApiGenerator {
    config: CallerConfig,
    title: String,
    version: String,
    description: Option<String>,
    /// When true, generate paths that route through caller proxy
    proxy_mode: bool,
    /// Proxy server URL (used when proxy_mode is true)
    proxy_url: String,
}

impl OpenApiGenerator {
    /// Create a new OpenAPI generator from Caller configuration
    /// 
    /// # Arguments
    /// * `config` - CallerConfig instance containing API definitions
    /// 
    /// # Returns
    /// A new OpenApiGenerator instance with default settings
    /// 
    /// # Example
    /// ```
    /// use caller::{OpenApiGenerator, CallerConfig};
    /// 
    /// let config = CallerConfig {
    ///     authorizations: vec![],
    ///     service_items: vec![],
    /// };
    /// let generator = OpenApiGenerator::new(config);
    /// ```
    pub fn new(config: CallerConfig) -> Self {
        Self {
            config,
            title: "Caller API".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            proxy_mode: false,
            proxy_url: "http://127.0.0.1:8080".to_string(),
        }
    }

    /// Create generator from currently loaded configuration file
    /// 
    /// This is a convenience method that loads the configuration
    /// that was previously loaded via ConfigLoader.
    /// 
    /// # Returns
    /// Ok(OpenApiGenerator) if config is loaded, Err otherwise
    /// 
    /// # Errors
    /// Returns CallerError if configuration is not loaded
    pub fn from_config_file() -> Result<Self, CallerError> {
        let config = ConfigLoader::get_full_config()?;
        Ok(Self::new(config))
    }

    /// Set the API title for OpenAPI documentation
    /// 
    /// # Arguments
    /// * `title` - API title
    /// 
    /// # Returns
    /// Self for method chaining
    /// 
    /// # Example
    /// ```
    /// use caller::{OpenApiGenerator, CallerConfig};
    /// 
    /// let generator = OpenApiGenerator::new(CallerConfig {
    ///     authorizations: vec![],
    ///     service_items: vec![],
    /// })
    /// .title("My REST API");
    /// ```
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Set the API version for OpenAPI documentation
    /// 
    /// # Arguments
    /// * `version` - API version string (e.g., "1.0.0")
    /// 
    /// # Returns
    /// Self for method chaining
    pub fn version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    /// Set the API description for OpenAPI documentation
    /// 
    /// # Arguments
    /// * `description` - API description text
    /// 
    /// # Returns
    /// Self for method chaining
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Enable or disable proxy mode
    /// 
    /// When enabled, all API paths in the OpenAPI spec will route through
    /// the caller proxy server. This allows Swagger UI "Try it out" to work
    /// through the caller, enabling authentication and other features.
    /// 
    /// # Arguments
    /// * `enabled` - Whether to enable proxy mode
    /// 
    /// # Returns
    /// Self for method chaining
    /// 
    /// # Example
    /// ```
    /// use caller::{OpenApiGenerator, CallerConfig};
    /// 
    /// let generator = OpenApiGenerator::new(CallerConfig {
    ///     authorizations: vec![],
    ///     service_items: vec![],
    /// })
    /// .proxy_mode(true)
    /// .proxy_url("http://localhost:8080");
    /// ```
    pub fn proxy_mode(mut self, enabled: bool) -> Self {
        self.proxy_mode = enabled;
        self
    }

    /// Set the proxy server URL for proxy mode
    /// 
    /// # Arguments
    /// * `url` - Proxy server base URL
    /// 
    /// # Returns
    /// Self for method chaining
    pub fn proxy_url(mut self, url: &str) -> Self {
        self.proxy_url = url.to_string();
        self
    }

    /// Generate the complete OpenAPI 3.0.3 document
    /// 
    /// This method processes all services and API items from the configuration
    /// and creates a complete OpenAPI specification.
    /// 
    /// # Returns
    /// An OpenApiDoc containing the complete OpenAPI specification
    /// 
    /// # Example
    /// ```
    /// use caller::{OpenApiGenerator, CallerConfig};
    /// 
    /// let config = CallerConfig {
    ///     authorizations: vec![],
    ///     service_items: vec![],
    /// };
    /// let doc = OpenApiGenerator::new(config).generate();
    /// assert_eq!(doc.openapi, "3.0.3");
    /// ```
    pub fn generate(&self) -> OpenApiDoc {
        let mut paths = HashMap::new();
        let mut servers = Vec::new();
        let mut tags = Vec::new();
        let mut security_schemes = HashMap::new();

        if self.proxy_mode {
            // In proxy mode, use local server
            servers.push(Server {
                url: self.proxy_url.clone(),
                description: Some("Caller Proxy Server".to_string()),
                variables: None,
            });
        }

        // Generate servers and tags from services
        for service in &self.config.service_items {
            // Add original server (for reference, or as primary in non-proxy mode)
            if !self.proxy_mode {
                servers.push(Server {
                    url: service.base_url.clone(),
                    description: Some(format!("{} API Server", service.api_name)),
                    variables: None,
                });
            }

            // Add tag
            tags.push(Tag {
                name: service.api_name.clone(),
                description: None,
                external_docs: None,
            });

            // Process API items
            for api_item in &service.api_items {
                let path = if self.proxy_mode {
                    // In proxy mode, generate proxy paths
                    format!("/proxy/{}/{}", service.api_name, api_item.method)
                } else {
                    self.normalize_path(&api_item.url)
                };
                let operation = self.create_operation(service, api_item);

                // Get or create path item
                let path_item = paths.entry(path.clone()).or_insert_with(|| PathItem {
                    summary: None,
                    description: None,
                    get: None,
                    post: None,
                    put: None,
                    delete: None,
                    patch: None,
                    parameters: Vec::new(),
                    servers: None,
                });

                // Add operation to path item based on HTTP method
                match api_item.http_method.to_lowercase().as_str() {
                    "get" => path_item.get = Some(operation),
                    "post" => path_item.post = Some(operation),
                    "put" => path_item.put = Some(operation),
                    "delete" => path_item.delete = Some(operation),
                    "patch" => path_item.patch = Some(operation),
                    _ => {}
                }
            }

            // Add security scheme if service has auth
            if let Some(auth_type) = &service.authorization_type {
                security_schemes.insert(
                    auth_type.clone(),
                    SecurityScheme::bearer().description(&format!("{} authentication", auth_type)),
                );
            }
        }

        OpenApiDoc {
            openapi: "3.0.3".to_string(),
            info: Info {
                title: self.title.clone(),
                version: self.version.clone(),
                description: self.description.clone(),
                contact: None,
                license: None,
            },
            servers,
            paths,
            components: if security_schemes.is_empty() {
                None
            } else {
                Some(Components {
                    schemas: None,
                    security_schemes: Some(security_schemes),
                    parameters: None,
                    request_bodies: None,
                    responses: None,
                })
            },
            tags,
        }
    }

    /// Normalize path by replacing {param} with OpenAPI path format
    fn normalize_path(&self, path: &str) -> String {
        // Path parameters are already in {param} format which matches OpenAPI
        path.to_string()
    }

    /// Create OpenAPI operation from API item
    fn create_operation(&self, service: &ServiceItem, api_item: &ApiItem) -> Operation {
        let mut parameters = Vec::new();
        let mut request_body = None;

        // Parse parameter types
        let param_type_lower = api_item.param_type.to_lowercase();
        let param_types: Vec<&str> = param_type_lower.split(',').collect();

        // Extract path parameters from URL
        let path_params = self.extract_path_params(&api_item.url);
        for param in path_params {
            parameters.push(Parameter {
                name: param.clone(),
                location: "path".to_string(),
                description: Some(format!("Path parameter: {}", param)),
                required: Some(true),
                schema: Some(Schema::string()),
                example: None,
            });
        }

        // Add query parameters if query type
        if param_types.contains(&"query") {
            // Generic query parameter support
            parameters.push(Parameter {
                name: "params".to_string(),
                location: "query".to_string(),
                description: Some("Query parameters".to_string()),
                required: Some(false),
                schema: Some(Schema::object()),
                example: None,
            });
        }

        // Add request body if json or form type
        if param_types.contains(&"json") {
            request_body = Some(RequestBody {
                description: Some("Request body".to_string()),
                content: HashMap::from([(
                    "application/json".to_string(),
                    MediaType {
                        schema: Some(Schema::object().description("Request body as JSON object")),
                        example: Some(serde_json::json!({"key": "value"})),
                        examples: None,
                    },
                )]),
                required: Some(true),
            });
        } else if param_types.contains(&"form") {
            request_body = Some(RequestBody {
                description: Some("Form data".to_string()),
                content: HashMap::from([(
                    "application/x-www-form-urlencoded".to_string(),
                    MediaType {
                        schema: Some(Schema::object().description("Form data")),
                        example: None,
                        examples: None,
                    },
                )]),
                required: Some(true),
            });
        }

        // Determine auth type (API item takes precedence over service)
        let auth_type = api_item
            .authorization_type
            .as_ref()
            .or(service.authorization_type.as_ref());

        let security = if let Some(auth) = auth_type {
            vec![HashMap::from([(auth.clone(), Vec::<String>::new())])]
        } else {
            Vec::new()
        };

        // Default response
        let responses = HashMap::from([(
            "200".to_string(),
            Response {
                description: "Successful response".to_string(),
                headers: None,
                content: Some(HashMap::from([(
                    "application/json".to_string(),
                    MediaType {
                        schema: Some(Schema::object()),
                        example: None,
                        examples: None,
                    },
                )])),
            },
        )]);

        Operation {
            summary: api_item.description.clone(),
            description: api_item.description.clone(),
            operation_id: Some(format!("{}_{}", service.api_name, api_item.method)),
            tags: vec![service.api_name.clone()],
            parameters,
            request_body,
            responses,
            security,
            deprecated: None,
        }
    }

    /// Extract path parameters from URL template
    fn extract_path_params(&self, url: &str) -> Vec<String> {
        let mut params = Vec::new();
        let mut current_param = String::new();
        let mut in_param = false;

        for c in url.chars() {
            match c {
                '{' => {
                    in_param = true;
                    current_param.clear();
                }
                '}' => {
                    if in_param && !current_param.is_empty() {
                        params.push(current_param.clone());
                    }
                    in_param = false;
                }
                _ if in_param => current_param.push(c),
                _ => {}
            }
        }

        params
    }

    /// Generate OpenAPI specification as JSON string
    /// 
    /// This method generates the complete OpenAPI document and serializes
    /// it to a pretty-printed JSON string suitable for writing to a file.
    /// 
    /// # Returns
    /// Ok(JSON string) if successful, Err if serialization fails
    /// 
    /// # Example
    /// ```
    /// use caller::{OpenApiGenerator, CallerConfig};
    /// 
    /// let config = CallerConfig {
    ///     authorizations: vec![],
    ///     service_items: vec![],
    /// };
    /// let json = OpenApiGenerator::new(config).to_json().unwrap();
    /// assert!(json.contains("\"openapi\""));
    /// ```
    pub fn to_json(&self) -> Result<String, CallerError> {
        let doc = self.generate();
        serde_json::to_string_pretty(&doc)
            .map_err(|e| CallerError::JsonError(format!("Failed to serialize OpenAPI: {}", e)))
    }

    /// Generate OpenAPI specification as YAML string
    /// 
    /// This method generates the complete OpenAPI document and serializes
    /// it to a YAML string suitable for writing to a file.
    /// 
    /// # Returns
    /// Ok(YAML string) if successful, Err if serialization fails
    /// 
    /// # Example
    /// ```
    /// use caller::{OpenApiGenerator, CallerConfig};
    /// 
    /// let config = CallerConfig {
    ///     authorizations: vec![],
    ///     service_items: vec![],
    /// };
    /// let yaml = OpenApiGenerator::new(config).to_yaml().unwrap();
    /// assert!(yaml.contains("openapi:"));
    /// ```
    pub fn to_yaml(&self) -> Result<String, CallerError> {
        let doc = self.generate();
        serde_yaml::to_string(&doc)
            .map_err(|e| CallerError::JsonError(format!("Failed to serialize OpenAPI: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::api_item::ApiItem;
    use crate::domain::caller_config::CallerConfig;
    use crate::domain::service_item::ServiceItem;

    fn create_test_config() -> CallerConfig {
        CallerConfig {
            authorizations: vec![],
            service_items: vec![ServiceItem {
                api_name: "test".to_string(),
                authorization_type: None,
                base_url: "https://api.example.com".to_string(),
                timeout: None,
                api_items: vec![
                    ApiItem {
                        method: "list".to_string(),
                        url: "/items".to_string(),
                        http_method: "GET".to_string(),
                        param_type: "query".to_string(),
                        description: Some("List all items".to_string()),
                        need_cache: None,
                        cache_time: None,
                        content_type: None,
                        authorization_type: None,
                        timeout: None,
                        use_new_http_client: None,
                    },
                    ApiItem {
                        method: "get".to_string(),
                        url: "/items/{id}".to_string(),
                        http_method: "GET".to_string(),
                        param_type: "path".to_string(),
                        description: Some("Get item by ID".to_string()),
                        need_cache: None,
                        cache_time: None,
                        content_type: None,
                        authorization_type: None,
                        timeout: None,
                        use_new_http_client: None,
                    },
                    ApiItem {
                        method: "create".to_string(),
                        url: "/items".to_string(),
                        http_method: "POST".to_string(),
                        param_type: "json".to_string(),
                        description: Some("Create new item".to_string()),
                        need_cache: None,
                        cache_time: None,
                        content_type: Some("application/json".to_string()),
                        authorization_type: Some("bearer".to_string()),
                        timeout: None,
                        use_new_http_client: None,
                    },
                ],
                use_new_http_client: None,
            }],
        }
    }

    #[test]
    fn test_generate_openapi() {
        let config = create_test_config();
        let generator = OpenApiGenerator::new(config)
            .title("Test API")
            .version("1.0.0");

        let doc = generator.generate();

        assert_eq!(doc.openapi, "3.0.3");
        assert_eq!(doc.info.title, "Test API");
        assert!(!doc.paths.is_empty());
    }

    #[test]
    fn test_to_json() {
        let config = create_test_config();
        let generator = OpenApiGenerator::new(config);
        let json = generator.to_json().unwrap();
        assert!(json.contains("\"openapi\""));
        assert!(json.contains("\"paths\""));
    }

    #[test]
    fn test_extract_path_params() {
        let config = create_test_config();
        let generator = OpenApiGenerator::new(config);

        let params = generator.extract_path_params("/items/{id}");
        assert_eq!(params, vec!["id"]);

        let params = generator.extract_path_params("/users/{user_id}/posts/{post_id}");
        assert_eq!(params, vec!["user_id", "post_id"]);

        let params = generator.extract_path_params("/items");
        assert!(params.is_empty());
    }
}
