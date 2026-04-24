use serde::{Deserialize, Serialize};

use crate::shared::error::CallerError;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct ApiConfig {
    pub method: String,
    pub url: String,
    pub http_method: String,
    pub param_type: String,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// Parse the configured `http_method` string into a typed [`HttpMethod`].
    pub fn http_method(&self) -> Result<HttpMethod, CallerError> {
        HttpMethod::parse(&self.http_method)
    }

    /// Parse the configured `param_type` string into typed [`ParamType`] values.
    ///
    /// Supports comma-separated combinations such as `path,json`.
    pub fn param_types(&self) -> Result<Vec<ParamType>, CallerError> {
        let param_types: Vec<ParamType> = self
            .param_type
            .split(',')
            .map(ParamType::parse)
            .collect::<Result<_, _>>()?;

        if param_types.is_empty() {
            return Err(CallerError::unsupported_param_type(&self.param_type));
        }

        if param_types.len() > 1 && param_types.contains(&ParamType::None) {
            return Err(CallerError::unsupported_param_type(&self.param_type));
        }

        Ok(param_types)
    }

    /// Check whether this API configuration contains a specific parameter kind.
    pub fn has_param_type(&self, param_type: ParamType) -> Result<bool, CallerError> {
        Ok(self.param_types()?.contains(&param_type))
    }

    /// Validate the protocol-facing configuration fields of this API item.
    pub fn validate(&self) -> Result<(), CallerError> {
        self.http_method()?;
        self.param_types()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn api_with(http_method: &str, param_type: &str) -> ApiConfig {
        ApiConfig {
            method: "list".to_string(),
            url: "/items".to_string(),
            http_method: http_method.to_string(),
            param_type: param_type.to_string(),
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
        assert!(matches!(
            api_with("GET", "none,json").validate(),
            Err(CallerError::UnsupportedParamType { .. })
        ));
    }
}
