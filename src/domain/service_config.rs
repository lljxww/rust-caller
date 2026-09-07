use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::api_config::ApiConfig;
use crate::shared::error::CallerError;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
/// Configuration shared by a named collection of API endpoints.
pub struct ServiceConfig {
    /// Service name used as the first component of `service.method`.
    pub api_name: String,
    /// Default runtime authentication provider name for endpoints in this service.
    pub authorization_type: Option<String>,
    /// HTTP(S) base URL without query, fragment, or embedded credentials.
    pub base_url: String,
    /// Default request timeout in milliseconds for this service.
    pub timeout: Option<u32>,
    /// API endpoints belonging to the service.
    pub api_items: Vec<ApiConfig>,
    /// Legacy client-selection flag accepted for compatibility; currently ignored.
    pub use_new_http_client: Option<bool>,
}

impl ServiceConfig {
    /// Validate the service and all nested API endpoint configurations.
    pub fn validate(&self) -> Result<(), CallerError> {
        if self.api_name.trim().is_empty()
            || self.api_name.trim() != self.api_name
            || self.api_name.contains('.')
        {
            return Err(CallerError::config_error(format!(
                "Service name must be non-empty and cannot contain '.': '{}'",
                self.api_name
            )));
        }
        if self.timeout == Some(0) {
            return Err(CallerError::config_error(format!(
                "Timeout for service '{}' must be greater than zero",
                self.api_name
            )));
        }
        validate_base_url(&self.base_url)?;
        validate_auth_reference(
            self.authorization_type.as_deref(),
            "service",
            &self.api_name,
        )?;

        let mut api_names = HashSet::new();

        for api in &self.api_items {
            if !api_names.insert(api.method.as_str()) {
                return Err(CallerError::config_error(format!(
                    "Duplicate api method '{}' in service '{}'",
                    api.method, self.api_name
                )));
            }
            api.validate()?;
            validate_http_url(&join_base_and_endpoint(&self.base_url, &api.url))?;
        }

        Ok(())
    }
}

pub(crate) fn join_base_and_endpoint(base_url: &str, endpoint: &str) -> String {
    if endpoint.is_empty() {
        base_url.trim_end_matches('/').to_string()
    } else {
        format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            endpoint.trim_start_matches('/')
        )
    }
}

fn validate_http_url(url: &str) -> Result<(), CallerError> {
    let parsed = reqwest::Url::parse(url).map_err(|e| CallerError::InvalidUrl {
        url: url.to_string(),
        message: e.to_string(),
    })?;

    match parsed.scheme() {
        "http" | "https" => {
            if !parsed.username().is_empty() || parsed.password().is_some() {
                return Err(CallerError::config_error(format!(
                    "Credentials must not be embedded in URL '{url}'"
                )));
            }
            if parsed.fragment().is_some() {
                return Err(CallerError::config_error(format!(
                    "URL fragments are not sent in HTTP requests: '{url}'"
                )));
            }
            Ok(())
        }
        scheme => Err(CallerError::UnsupportedUrlScheme {
            url: url.to_string(),
            scheme: scheme.to_string(),
        }),
    }
}

fn validate_base_url(url: &str) -> Result<(), CallerError> {
    validate_http_url(url)?;
    let parsed = reqwest::Url::parse(url).map_err(|error| CallerError::InvalidUrl {
        url: url.to_string(),
        message: error.to_string(),
    })?;
    if parsed.query().is_some() {
        return Err(CallerError::config_error(format!(
            "Service base URL cannot contain a query string: '{url}'"
        )));
    }
    Ok(())
}

pub(crate) fn validate_auth_reference(
    value: Option<&str>,
    owner_kind: &str,
    owner_name: &str,
) -> Result<(), CallerError> {
    if let Some(value) = value
        && (value.trim().is_empty() || value.trim() != value)
    {
        return Err(CallerError::config_error(format!(
            "Authorization type for {owner_kind} '{owner_name}' must be non-empty and cannot have surrounding whitespace"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HttpMethod, ParamType};

    fn api(url: &str) -> ApiConfig {
        ApiConfig {
            method: "list".to_string(),
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
        }
    }

    fn service(base_url: &str, api_items: Vec<ApiConfig>) -> ServiceConfig {
        ServiceConfig {
            api_name: "Svc".to_string(),
            authorization_type: None,
            base_url: base_url.to_string(),
            timeout: None,
            api_items,
            use_new_http_client: None,
        }
    }

    #[test]
    fn validates_http_service_and_api_urls() {
        assert!(
            service("https://api.example.com", vec![api("/items")])
                .validate()
                .is_ok()
        );
    }

    #[test]
    fn rejects_invalid_base_url() {
        let err = service("not a url", vec![])
            .validate()
            .expect_err("invalid base URL should fail validation");
        assert!(matches!(err, CallerError::InvalidUrl { .. }));
    }

    #[test]
    fn rejects_non_http_base_url() {
        let err = service("file:///tmp/api", vec![])
            .validate()
            .expect_err("non-http URL should fail validation");
        assert!(matches!(err, CallerError::UnsupportedUrlScheme { .. }));
    }

    #[test]
    fn rejects_invalid_resolved_api_url() {
        let err = service("https://api.example.com", vec![api(":bad")])
            .validate()
            .expect_err("invalid resolved API URL should fail validation");
        assert!(matches!(err, CallerError::InvalidUrl { .. }));
    }

    #[test]
    fn rejects_unsafe_base_url_components() {
        assert!(
            service("https://user:secret@api.example.com", vec![])
                .validate()
                .is_err()
        );
        assert!(
            service("https://api.example.com?tenant=1", vec![])
                .validate()
                .is_err()
        );
        assert!(
            service("https://api.example.com#fragment", vec![])
                .validate()
                .is_err()
        );
    }

    #[test]
    fn joins_base_and_endpoint_with_exactly_one_separator() {
        assert_eq!(
            join_base_and_endpoint("https://api.example.com/", "/items"),
            "https://api.example.com/items"
        );
        assert_eq!(
            join_base_and_endpoint("https://api.example.com", "items"),
            "https://api.example.com/items"
        );
        assert_eq!(
            join_base_and_endpoint("https://api.example.com/", ""),
            "https://api.example.com"
        );
    }
}
