use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashSet;

use crate::domain::service_config::validate_auth_reference;
use crate::shared::error::CallerError;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
/// Configuration for one named API endpoint within a service.
pub struct ApiConfig {
    /// API method name used in the `service.method` lookup key.
    pub method: String,
    /// Endpoint path, either empty for the service root or beginning with `/`.
    pub url: String,
    #[serde(
        deserialize_with = "deserialize_http_method",
        serialize_with = "serialize_http_method"
    )]
    /// HTTP method used to call the endpoint.
    pub http_method: HttpMethod,
    #[serde(
        deserialize_with = "deserialize_param_types",
        serialize_with = "serialize_param_types"
    )]
    /// Locations in which this endpoint accepts request parameters.
    pub param_type: Vec<ParamType>,
    /// Human-readable endpoint description used by generated OpenAPI documents.
    pub description: Option<String>,
    /// Legacy cache flag accepted for configuration compatibility; currently ignored.
    pub need_cache: Option<bool>,
    /// Legacy cache duration accepted for configuration compatibility; currently ignored.
    pub cache_time: Option<u32>,
    /// Explicit request content type, if the endpoint requires one.
    pub content_type: Option<String>,
    /// Name of the runtime authentication provider to apply.
    pub authorization_type: Option<String>,
    /// Endpoint request timeout in milliseconds, overriding service and caller defaults.
    pub timeout: Option<u32>,
    /// Legacy client-selection flag accepted for compatibility; currently ignored.
    pub use_new_http_client: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// HTTP methods supported by configured endpoints.
pub enum HttpMethod {
    /// `GET`
    Get,
    /// `POST`
    Post,
    /// `PUT`
    Put,
    /// `DELETE`
    Delete,
    /// `PATCH`
    Patch,
    /// `HEAD`
    Head,
    /// `OPTIONS`
    Options,
}

impl HttpMethod {
    /// Parse an HTTP method name case-insensitively.
    pub fn parse(value: &str) -> Result<Self, CallerError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "get" => Ok(Self::Get),
            "post" => Ok(Self::Post),
            "put" => Ok(Self::Put),
            "delete" => Ok(Self::Delete),
            "patch" => Ok(Self::Patch),
            "head" => Ok(Self::Head),
            "options" => Ok(Self::Options),
            _ => Err(CallerError::http_method_not_supported(value)),
        }
    }

    /// Convert this value to the corresponding `reqwest` method.
    pub fn as_reqwest_method(self) -> reqwest::Method {
        match self {
            Self::Get => reqwest::Method::GET,
            Self::Post => reqwest::Method::POST,
            Self::Put => reqwest::Method::PUT,
            Self::Delete => reqwest::Method::DELETE,
            Self::Patch => reqwest::Method::PATCH,
            Self::Head => reqwest::Method::HEAD,
            Self::Options => reqwest::Method::OPTIONS,
        }
    }

    /// Return the uppercase HTTP method name used in configuration serialization.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
        }
    }

    /// Return the lowercase key used by an OpenAPI path item.
    pub fn as_openapi_key(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Post => "post",
            Self::Put => "put",
            Self::Delete => "delete",
            Self::Patch => "patch",
            Self::Head => "head",
            Self::Options => "options",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Supported request parameter locations.
pub enum ParamType {
    /// No request parameters.
    None,
    /// URL query parameters.
    Query,
    /// URL path parameters like `/items/{id}`.
    Path,
    /// JSON request body.
    Json,
    /// Form request body.
    Form,
}

impl ParamType {
    /// Parse a configured parameter location case-insensitively.
    pub fn parse(value: &str) -> Result<Self, CallerError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "none" => Ok(Self::None),
            "query" => Ok(Self::Query),
            "path" => Ok(Self::Path),
            "json" => Ok(Self::Json),
            "form" => Ok(Self::Form),
            _ => Err(CallerError::unsupported_param_type(value.trim())),
        }
    }

    /// Return the lowercase representation used in configuration files.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Query => "query",
            Self::Path => "path",
            Self::Json => "json",
            Self::Form => "form",
        }
    }
}

impl ApiConfig {
    /// Parse and validate a comma-separated list of parameter locations.
    pub fn parse_param_types(value: &str) -> Result<Vec<ParamType>, CallerError> {
        let param_types: Vec<ParamType> = value
            .split(',')
            .map(ParamType::parse)
            .collect::<Result<_, _>>()?;

        validate_param_types(&param_types, value)?;
        Ok(param_types)
    }

    /// Serialize the configured parameter locations as a comma-separated string.
    pub fn param_types_as_str(&self) -> String {
        param_types_to_string(&self.param_type)
    }

    /// Return the configured typed [`HttpMethod`].
    pub fn http_method(&self) -> Result<HttpMethod, CallerError> {
        Ok(self.http_method)
    }

    /// Return the configured typed [`ParamType`] values.
    ///
    /// Supports comma-separated combinations such as `path,json`.
    pub fn param_types(&self) -> Result<Vec<ParamType>, CallerError> {
        validate_param_types(&self.param_type, &self.param_types_as_str())?;
        Ok(self.param_type.clone())
    }

    /// Check whether this API configuration contains a specific parameter kind.
    pub fn has_param_type(&self, param_type: ParamType) -> Result<bool, CallerError> {
        Ok(self.param_types()?.contains(&param_type))
    }

    /// Validate the protocol-facing configuration fields of this API item.
    pub fn validate(&self) -> Result<(), CallerError> {
        if self.method.trim().is_empty()
            || self.method.trim() != self.method
            || self.method.contains('.')
        {
            return Err(CallerError::config_error(format!(
                "API method name must be non-empty and cannot contain '.': '{}'",
                self.method
            )));
        }
        self.http_method()?;
        let param_types = self.param_types()?;
        self.validate_url_path()?;
        self.validate_path_parameter_shape(&param_types)?;

        if param_types.contains(&ParamType::Json) && param_types.contains(&ParamType::Form) {
            return Err(CallerError::unsupported_param_type(
                self.param_types_as_str(),
            ));
        }

        if self.timeout == Some(0) {
            return Err(CallerError::config_error(format!(
                "Timeout for API method '{}' must be greater than zero",
                self.method
            )));
        }

        validate_auth_reference(self.authorization_type.as_deref(), "API", &self.method)?;

        if let Some(content_type) = &self.content_type {
            if content_type.trim().is_empty() || content_type.trim() != content_type {
                return Err(CallerError::config_error(format!(
                    "Content type for API method '{}' must be non-empty and cannot have surrounding whitespace",
                    self.method
                )));
            }
            reqwest::header::HeaderValue::from_str(content_type).map_err(|error| {
                CallerError::InvalidHeaderValue {
                    name: reqwest::header::CONTENT_TYPE.as_str().to_string(),
                    message: error.to_string(),
                }
            })?;
        }
        Ok(())
    }

    fn validate_path_parameter_shape(&self, param_types: &[ParamType]) -> Result<(), CallerError> {
        let mut depth = 0_u8;
        let mut placeholder_count = 0_usize;
        let mut placeholder_has_content = false;

        for character in self.url.chars() {
            match character {
                '{' if depth == 0 => {
                    depth = 1;
                    placeholder_has_content = false;
                }
                '}' if depth == 0 => {
                    return Err(CallerError::InvalidUrl {
                        url: self.url.clone(),
                        message: "unbalanced path parameter braces".to_string(),
                    });
                }
                '{' => {
                    return Err(CallerError::InvalidUrl {
                        url: self.url.clone(),
                        message: "nested path parameter braces are not supported".to_string(),
                    });
                }
                '}' => {
                    if !placeholder_has_content {
                        return Err(CallerError::InvalidUrl {
                            url: self.url.clone(),
                            message: "path parameter name cannot be empty".to_string(),
                        });
                    }
                    depth = 0;
                    placeholder_count += 1;
                }
                _ if depth == 1 => placeholder_has_content = true,
                _ => {}
            }
        }

        if depth != 0 {
            return Err(CallerError::InvalidUrl {
                url: self.url.clone(),
                message: "unbalanced path parameter braces".to_string(),
            });
        }

        let declares_path = param_types.contains(&ParamType::Path);
        if declares_path != (placeholder_count > 0) {
            return Err(CallerError::config_error(format!(
                "API method '{}' must declare path parameters exactly when its URL contains placeholders",
                self.method
            )));
        }

        Ok(())
    }

    fn validate_url_path(&self) -> Result<(), CallerError> {
        if self.url.is_empty() || self.url.starts_with('/') {
            return Ok(());
        }

        Err(CallerError::InvalidUrl {
            url: self.url.clone(),
            message: format!(
                "API url for method '{}' must be empty or start with '/'",
                self.method
            ),
        })
    }
}

fn validate_param_types(param_types: &[ParamType], raw_value: &str) -> Result<(), CallerError> {
    if param_types.is_empty() {
        return Err(CallerError::unsupported_param_type(raw_value));
    }

    if param_types.len() > 1 && param_types.contains(&ParamType::None) {
        return Err(CallerError::unsupported_param_type(raw_value));
    }

    let mut unique_param_types = HashSet::new();
    for param_type in param_types {
        if !unique_param_types.insert(param_type) {
            return Err(CallerError::unsupported_param_type(raw_value));
        }
    }

    Ok(())
}

fn param_types_to_string(param_types: &[ParamType]) -> String {
    let values: Vec<&'static str> = param_types
        .iter()
        .map(|param_type| param_type.as_str())
        .collect();
    if values.is_empty() {
        ParamType::None.as_str().to_string()
    } else {
        values.join(",")
    }
}

fn deserialize_http_method<'de, D>(deserializer: D) -> Result<HttpMethod, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    HttpMethod::parse(&value).map_err(serde::de::Error::custom)
}

fn serialize_http_method<S>(http_method: &HttpMethod, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(http_method.as_str())
}

fn deserialize_param_types<'de, D>(deserializer: D) -> Result<Vec<ParamType>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    ApiConfig::parse_param_types(&value).map_err(serde::de::Error::custom)
}

fn serialize_param_types<S>(param_types: &[ParamType], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&param_types_to_string(param_types))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn api_with(http_method: &str, param_type: &str) -> ApiConfig {
        ApiConfig {
            method: "list".to_string(),
            url: "/items".to_string(),
            http_method: HttpMethod::parse(http_method).unwrap(),
            param_type: ApiConfig::parse_param_types(param_type).unwrap(),
            description: None,
            need_cache: None,
            cache_time: None,
            content_type: None,
            authorization_type: None,
            timeout: None,
            use_new_http_client: None,
        }
    }

    #[test]
    fn parses_http_method_case_insensitively() {
        assert_eq!(
            api_with("GET", "none").http_method().unwrap(),
            HttpMethod::Get
        );
        assert_eq!(
            api_with("patch", "none").http_method().unwrap(),
            HttpMethod::Patch
        );
    }

    #[test]
    fn parses_combined_param_types_with_whitespace() {
        assert_eq!(
            api_with("GET", "path, json").param_types().unwrap(),
            vec![ParamType::Path, ParamType::Json]
        );
    }

    #[test]
    fn rejects_invalid_param_type_combinations() {
        let mut api = api_with("GET", "none");
        api.param_type = vec![ParamType::None, ParamType::Json];

        assert!(matches!(
            api.validate(),
            Err(CallerError::UnsupportedParamType { .. })
        ));
    }

    #[test]
    fn rejects_duplicate_param_type_combinations() {
        let mut api = api_with("GET", "none");
        api.param_type = vec![ParamType::Query, ParamType::Query];

        assert!(matches!(
            api.validate(),
            Err(CallerError::UnsupportedParamType { .. })
        ));
    }

    #[test]
    fn validates_endpoint_url_shape() {
        assert!(api_with("GET", "none").validate().is_ok());

        let mut root_endpoint = api_with("GET", "none");
        root_endpoint.url.clear();
        assert!(root_endpoint.validate().is_ok());
    }

    #[test]
    fn rejects_endpoint_url_without_leading_slash() {
        let mut api = api_with("GET", "none");
        api.url = "items".to_string();

        assert!(matches!(
            api.validate(),
            Err(CallerError::InvalidUrl { .. })
        ));
    }
}
