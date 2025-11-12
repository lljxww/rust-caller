use super::constants;
use crate::config::config_loader::ConfigLoader;
use crate::domain::caller_config::CallerConfig;
use crate::domain::service_item::ServiceItem;
use crate::domain::{api_item::ApiItem, api_result::ApiResult};
use reqwest::{header, Method};
use std::collections::HashMap;
use crate::shared::error::CallerError;

pub struct CallerContext {
    pub service_name: String,
    pub api_name: String,
    pub service_item: ServiceItem,
    pub api_item: ApiItem,
    pub http_method: Method,
    pub url: String,
    pub params: Option<HashMap<String, String>>,
}

impl CallerContext {
    pub fn build(method: &str, params: Option<HashMap<String, String>>) -> Result<CallerContext, CallerError> {
        let splited_method = split_method(method)?;

        let service_name = &splited_method[0];
        let api_name = &splited_method[1];

        let (service_item, api_item) = ConfigLoader::get_config(service_name, api_name)?;

        let mut url = get_final_url(method, &*ConfigLoader::get_config_ref()?)?;

        if api_item.param_type.to_lowercase() == "path" {
            if let Some(params_map) = &params {
                validate_path_parameters(&url, params_map.keys().map(|s| s.as_str()).collect())?;
                url = substitute_path_parameters(&url, params_map)?;
            }
        }

        let http_method = get_http_method(&api_item.http_method)?;

        Ok(CallerContext {
            service_name: service_name.to_string(),
            api_name: api_name.to_string(),
            service_item,
            api_item,
            http_method,
            url,
            params,
        })
    }

    pub async fn call(
        method: &str,
        params: Option<HashMap<String, String>>,
    ) -> Result<ApiResult, CallerError> {
        let context = CallerContext::build(method, params)?;
        let client = reqwest::Client::new();

        let mut rb = client
            .request(context.http_method, &context.url)
            .header(header::USER_AGENT, constants::UA)
            .header(header::CONTENT_TYPE, constants::DEFAULT_CONTENT_TYPE);

        if let Some(params_map) = &context.params {
            match context.api_item.param_type.to_lowercase().as_str() {
                "query" => rb = rb.query(params_map),
                "json" => rb = rb.json(params_map),
                "form" => rb = rb.form(params_map),
                "none" => {
                    // No parameters needed for the request body
                }
                "path" => {
                    // Path parameters were already substituted in URL construction
                    // No additional processing needed for request body
                }
                param_type => {
                    return Err(CallerError::parameter_error(
                        format!("Unsupported parameter type: {}", param_type)
                    ));
                }
            }
        }

        let response = rb.send().await?;

        let status_code = response.status();
        let result = response.text().await?;

        Ok(ApiResult::build(result, status_code)?)
    }
}

pub fn split_method(method: &str) -> Result<Vec<String>, CallerError> {
    if !method.contains(".") {
        return Err(CallerError::invalid_method_format(method));
    }

    if method.starts_with('.') || method.ends_with('.') {
        return Err(CallerError::invalid_method_format(format!(
            "Method cannot start or end with dot: '{}'", method
        )));
    }

    let parts: Vec<&str> = method.split('.').collect();
    let result: Vec<String> = parts.iter().map(|s| s.to_string()).collect();

    if result.len() != 2 {
        return Err(CallerError::invalid_method_format(format!(
            "Method must be in format 'service.api', got: '{}'", method
        )));
    }

    if result[0].is_empty() {
        return Err(CallerError::invalid_method_format(
            "Service name cannot be empty".to_string()
        ));
    }

    if result[1].is_empty() {
        return Err(CallerError::invalid_method_format(
            "API method name cannot be empty".to_string()
        ));
    }

    Ok(result)
}

fn get_final_url(method: &str, config: &CallerConfig) -> Result<String, CallerError> {
    let splited_method = split_method(method)?;

    let service_item = config
        .service_items
        .iter()
        .find(|s| s.api_name == splited_method[0])
        .ok_or_else(|| CallerError::service_not_found(&splited_method[0]))?;

    let api_item = service_item
        .api_items
        .iter()
        .find(|a| a.method == splited_method[1])
        .ok_or_else(|| CallerError::api_not_found(&splited_method[0], &splited_method[1]))?;

    Ok(format!("{}{}", service_item.base_url, api_item.url))
}

pub fn get_http_method(http_method: &str) -> Result<Method, CallerError> {
    match http_method.to_lowercase().as_str() {
        "get" => Ok(Method::GET),
        "post" => Ok(Method::POST),
        "put" => Ok(Method::PUT),
        "delete" => Ok(Method::DELETE),
        "patch" => Ok(Method::PATCH),
        _ => Err(CallerError::http_method_not_supported(http_method.to_string())),
    }
}

fn validate_path_parameters(url: &str, provided_params: Vec<&str>) -> Result<(), CallerError> {
    // Find all required parameters in URL (between { and })
    let mut required_params = Vec::new();
    let mut chars = url.chars().peekable();
    let mut current_param = String::new();
    let mut in_param = false;

    while let Some(c) = chars.next() {
        match c {
            '{' => {
                in_param = true;
                current_param.clear();
            }
            '}' => {
                if in_param {
                    if !current_param.is_empty() {
                        required_params.push(current_param.clone());
                    }
                    in_param = false;
                }
            }
            _ if in_param => current_param.push(c),
            _ => (),
        }
    }

    // Check if all provided params are required
    for param in &provided_params {
        if !required_params.contains(&param.to_string()) {
            return Err(CallerError::parameter_error(format!(
                "Parameter '{}' is not required for this URL. Required parameters: {:?}", param, required_params
            )));
        }
    }

    // Check if all required params are provided
    for required in &required_params {
        if !provided_params.contains(&required.as_str()) {
            return Err(CallerError::url_parameter_not_found(required.to_string()));
        }
    }

    Ok(())
}

fn substitute_path_parameters(url: &str, params: &HashMap<String, String>) -> Result<String, CallerError> {
    let mut result = url.to_string();

    for (key, value) in params {
        let placeholder = format!("{{{}}}", key);
        if result.contains(&placeholder) {
            result = result.replace(&placeholder, value);
        }
    }

    // Check if all placeholders were replaced
    if result.contains('{') && result.contains('}') {
        let mut unreplaced_placeholders = Vec::new();
        let mut chars = result.chars().peekable();
        let mut in_param = false;
        let mut current_param = String::new();

        while let Some(c) = chars.next() {
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
            "Missing values for URL parameters: {:?}", unreplaced_placeholders
        )));
    }

    Ok(result)
}
