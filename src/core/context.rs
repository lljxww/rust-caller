use crate::shared::error::CallerError;
use std::collections::HashMap;

pub(crate) fn split_method(method: &str) -> Result<Vec<String>, CallerError> {
    if !method.contains('.') {
        return Err(CallerError::invalid_method_format(method));
    }

    if method.starts_with('.') || method.ends_with('.') {
        return Err(CallerError::invalid_method_format(format!(
            "method cannot start or end with dot: '{}'",
            method
        )));
    }

    let parts: Vec<&str> = method.split('.').collect();
    let result: Vec<String> = parts.iter().map(|s| s.to_string()).collect();

    if result.len() != 2 {
        return Err(CallerError::invalid_method_format(format!(
            "method must be in format 'service.api', got: '{}'",
            method
        )));
    }

    if result[0].is_empty() {
        return Err(CallerError::invalid_method_format(
            "Service name cannot be empty".to_string(),
        ));
    }

    if result[1].is_empty() {
        return Err(CallerError::invalid_method_format(
            "API method name cannot be empty".to_string(),
        ));
    }

    Ok(result)
}

pub(crate) fn validate_path_parameters(
    url: &str,
    provided_params: Vec<&str>,
) -> Result<(), CallerError> {
    let mut required_params = Vec::new();
    let mut current_param = String::new();
    let mut in_param = false;

    for c in url.chars() {
        match c {
            '{' => {
                in_param = true;
                current_param.clear();
            }
            '}' if in_param => {
                if !current_param.is_empty() {
                    required_params.push(current_param.clone());
                }
                in_param = false;
            }
            '}' => {}
            _ if in_param => current_param.push(c),
            _ => (),
        }
    }

    // Only check if all required parameters are provided
    // Don't validate that provided parameters are all required (to support combined param types like "path,json")
    for required in &required_params {
        if !provided_params.contains(&required.as_str()) {
            return Err(CallerError::url_parameter_not_found(required.to_string()));
        }
    }

    Ok(())
}

pub(crate) fn substitute_path_parameters(
    url: &str,
    params: &HashMap<String, String>,
) -> Result<String, CallerError> {
    let mut result = url.to_string();

    for (key, value) in params {
        let placeholder = format!("{{{}}}", key);
        if result.contains(&placeholder) {
            result = result.replace(&placeholder, value);
        }
    }

    if result.contains('{') && result.contains('}') {
        let mut unreplaced_placeholders = Vec::new();
        let mut current_param = String::new();
        let mut in_param = false;

        for c in result.chars() {
            match c {
                '{' => {
                    in_param = true;
                    current_param.clear();
                }
                '}' => {
                    if in_param {
                        unreplaced_placeholders.push(current_param.clone());
                    }
                    in_param = false;
                }
                _ if in_param => current_param.push(c),
                _ => (),
            }
        }

        return Err(CallerError::parameter_error(format!(
            "Missing values for URL parameters: {:?}",
            unreplaced_placeholders
        )));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::config_loader::{ConfigLoader, TEST_STATE_LOCK};
    use crate::domain::{
        api_config::ApiConfig, caller_config::CallerConfig, service_config::ServiceConfig,
    };

    fn test_config_with_auth(auth_type: &str) -> CallerConfig {
        CallerConfig {
            authorizations: vec![],
            service_items: vec![ServiceConfig {
                api_name: "AuthService".to_string(),
                authorization_type: Some(auth_type.to_string()),
                base_url: "http://127.0.0.1:9".to_string(),
                timeout: Some(50),
                api_items: vec![ApiConfig {
                    method: "list".to_string(),
                    url: "/items".to_string(),
                    http_method: "GET".to_string(),
                    param_type: "none".to_string(),
                    description: None,
                    need_cache: None,
                    cache_time: None,
                    content_type: None,
                    authorization_type: None,
                    timeout: None,
                    use_new_http_client: None,
                }],
                use_new_http_client: None,
            }],
        }
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn test_missing_auth_provider_returns_error() {
        let _guard = TEST_STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        ConfigLoader::reset_state_for_test();
        crate::clear_auth().unwrap();
        ConfigLoader::init_with_config(test_config_with_auth("missing_provider"));

        let err = crate::call("AuthService.list", None)
            .await
            .expect_err("missing auth provider should fail");

        match err {
            CallerError::UnknownAuthProvider { name } => {
                assert_eq!(name, "missing_provider");
            }
            other => panic!("expected UnknownAuthProvider, got {other:?}"),
        }

        ConfigLoader::reset_state_for_test();
        crate::clear_auth().unwrap();
    }
}
