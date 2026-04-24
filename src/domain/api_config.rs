use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashSet;

use crate::shared::error::CallerError;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct ApiConfig {
    pub method: String,
    pub url: String,
    #[serde(
        deserialize_with = "deserialize_http_method",
        serialize_with = "serialize_http_method"
    )]
    pub http_method: HttpMethod,
    #[serde(
        deserialize_with = "deserialize_param_types",
        serialize_with = "serialize_param_types"
    )]
    pub param_type: Vec<ParamType>,
    pub description: Option<String>,
    pub need_cache: Option<bool>,
    pub cache_time: Option<u32>,
    pub content_type: Option<String>,
    pub authorization_type: Option<String>,
    pub timeout: Option<u32>,
    pub use_new_http_client: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

impl HttpMethod {
    pub fn parse(value: &str) -> Result<Self, CallerError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "get" => Ok(Self::Get),
            "post" => Ok(Self::Post),
            "put" => Ok(Self::Put),
            "delete" => Ok(Self::Delete),
            "patch" => Ok(Self::Patch),
            _ => Err(CallerError::http_method_not_supported(value)),
        }
    }

    pub fn as_reqwest_method(self) -> reqwest::Method {
        match self {
            Self::Get => reqwest::Method::GET,
            Self::Post => reqwest::Method::POST,
            Self::Put => reqwest::Method::PUT,
            Self::Delete => reqwest::Method::DELETE,
            Self::Patch => reqwest::Method::PATCH,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
        }
    }

    pub fn as_openapi_key(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Post => "post",
            Self::Put => "put",
            Self::Delete => "delete",
            Self::Patch => "patch",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub fn parse_param_types(value: &str) -> Result<Vec<ParamType>, CallerError> {
        let param_types: Vec<ParamType> = value
            .split(',')
            .map(ParamType::parse)
            .collect::<Result<_, _>>()?;

        validate_param_types(&param_types, value)?;
        Ok(param_types)
    }

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
        self.http_method()?;
        self.param_types()?;
        self.validate_url_path()?;
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
