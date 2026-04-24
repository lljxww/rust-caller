use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::api_config::ApiConfig;
use crate::shared::error::CallerError;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct ServiceConfig {
    pub api_name: String,
    pub authorization_type: Option<String>,
    pub base_url: String,
    pub timeout: Option<u32>,
    pub api_items: Vec<ApiConfig>,
    pub use_new_http_client: Option<bool>,
}

impl ServiceConfig {
    pub fn validate(&self) -> Result<(), CallerError> {
        validate_http_url(&self.base_url)?;

        let mut api_names = HashSet::new();

        for api in &self.api_items {
            if !api_names.insert(api.method.as_str()) {
                return Err(CallerError::config_error(format!(
                    "Duplicate api method '{}' in service '{}'",
                    api.method, self.api_name
                )));
            }
            api.validate()?;
            validate_http_url(&format!("{}{}", self.base_url, api.url))?;
        }

        Ok(())
    }
}

fn validate_http_url(url: &str) -> Result<(), CallerError> {
    let parsed =
        reqwest::Url::parse(url).map_err(|e| CallerError::InvalidUrlFormat(e.to_string()))?;

    match parsed.scheme() {
        "http" | "https" => Ok(()),
        scheme => Err(CallerError::InvalidUrlFormat(format!(
            "Unsupported URL scheme '{}': {}",
            scheme, url
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn api(url: &str) -> ApiConfig {
        ApiConfig {
            method: "list".to_string(),
            url: url.to_string(),
            http_method: "GET".to_string(),
            param_type: "none".to_string(),
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
        assert!(matches!(err, CallerError::InvalidUrlFormat(_)));
    }

    #[test]
    fn rejects_non_http_base_url() {
        let err = service("file:///tmp/api", vec![])
            .validate()
            .expect_err("non-http URL should fail validation");
        assert!(matches!(err, CallerError::InvalidUrlFormat(_)));
    }

    #[test]
    fn rejects_invalid_resolved_api_url() {
        let err = service("https://api.example.com", vec![api(":bad")])
            .validate()
            .expect_err("invalid resolved API URL should fail validation");
        assert!(matches!(err, CallerError::InvalidUrlFormat(_)));
    }
}
