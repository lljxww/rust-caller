use super::constants;
use crate::config::config_loader::ConfigLoader;
use crate::domain::retry_config::RetryConfig;
use crate::domain::service_item::ServiceItem;
use crate::domain::{api_item::ApiItem, api_result::ApiResult, download_result::DownloadResult};
use crate::shared::error::CallerError;
use reqwest::{Method, header};
use std::collections::HashMap;

pub(crate) struct CallerContext {
    #[allow(dead_code)]
    pub service_name: String,
    #[allow(dead_code)]
    pub api_name: String,
    #[allow(dead_code)]
    pub service_item: ServiceItem,
    pub api_item: ApiItem,
    pub http_method: Method,
    pub url: String,
    pub params: Option<HashMap<String, String>>,
}

impl CallerContext {
    pub fn build(
        method: &str,
        params: Option<HashMap<String, String>>,
    ) -> Result<CallerContext, CallerError> {
        let splited_method = split_method(method)?;

        let service_name = &splited_method[0];
        let api_name = &splited_method[1];

        let (service_item, api_item, base_url) =
            ConfigLoader::get_config_with_base_url(service_name, api_name)?;

        let mut url = format!("{}{}", base_url, api_item.url);

        // Parse parameter types (supports single type like "path" or combined like "path,json")
        let param_type_lower = api_item.param_type.to_lowercase();
        let param_types: Vec<&str> = param_type_lower.split(',').collect();

        // Handle path parameters if present in param types
        if param_types.contains(&"path") {
            let params_map = params.as_ref().ok_or_else(|| {
                CallerError::parameter_error(
                    "Path parameters are required for this API".to_string(),
                )
            })?;
            validate_path_parameters(&url, params_map.keys().map(|s| s.as_str()).collect())?;
            url = substitute_path_parameters(&url, params_map)?;
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
            // Parse parameter types (supports single type like "path" or combined like "path,json")
            let param_type_lower = context.api_item.param_type.to_lowercase();
            let param_types: Vec<&str> = param_type_lower.split(',').collect();

            // Handle each parameter type
            for param_type in param_types {
                match param_type.trim() {
                    "query" => rb = rb.query(params_map),
                    "json" => rb = rb.json(params_map),
                    "form" => rb = rb.form(params_map),
                    "path" => {
                        // Path parameters were already substituted in URL construction
                        // No additional processing needed for request body
                    }
                    "none" => {
                        // No parameters needed for the request body
                    }
                    unsupported => {
                        return Err(CallerError::parameter_error(format!(
                            "Unsupported parameter type: {}",
                            unsupported
                        )));
                    }
                }
            }
        }

        let response = rb.send().await?;

        let status_code = response.status();
        let result = response.text().await?;

        ApiResult::build(result, status_code)
    }

    /// Call API with retry support using exponential backoff
    pub async fn call_with_retry(
        method: &str,
        params: Option<HashMap<String, String>>,
        retry_config: RetryConfig,
    ) -> Result<ApiResult, CallerError> {
        let context = CallerContext::build(method, params)?;
        let client = reqwest::Client::new();

        let mut last_error: Option<CallerError> = None;
        let mut attempt = 0u32;

        while attempt <= retry_config.max_retries {
            if attempt > 0 {
                let delay = retry_config.calculate_delay(attempt - 1);
                tokio::time::sleep(delay).await;
            }

            let mut rb = client
                .request(context.http_method.clone(), &context.url)
                .header(header::USER_AGENT, constants::UA)
                .header(header::CONTENT_TYPE, constants::DEFAULT_CONTENT_TYPE);

            if let Some(params_map) = &context.params {
                // Parse parameter types (supports single type like "path" or combined like "path,json")
                let param_type_lower = context.api_item.param_type.to_lowercase();
                let param_types: Vec<&str> = param_type_lower.split(',').collect();

                // Handle each parameter type
                for param_type in param_types {
                    match param_type.trim() {
                        "query" => rb = rb.query(params_map),
                        "json" => rb = rb.json(params_map),
                        "form" => rb = rb.form(params_map),
                        "path" => {
                            // Path parameters were already substituted in URL construction
                            // No additional processing needed for request body
                        }
                        "none" => {
                            // No parameters needed for the request body
                        }
                        unsupported => {
                            return Err(CallerError::parameter_error(format!(
                                "Unsupported parameter type: {}",
                                unsupported
                            )));
                        }
                    }
                }
            }

            match rb.send().await {
                Ok(response) => {
                    let status_code = response.status();
                    let status = status_code.as_u16();

                    // Check if we should retry based on status code
                    if retry_config.should_retry_status(status) && attempt < retry_config.max_retries {
                        last_error = Some(CallerError::HttpError(format!(
                            "HTTP {} - Retrying (attempt {}/{})",
                            status, attempt + 1, retry_config.max_retries
                        )));
                        attempt += 1;
                        continue;
                    }

                    let result = response.text().await?;
                    return ApiResult::build(result, status_code);
                }
                Err(e) => {
                    // Check if we should retry on network error
                    if retry_config.retry_on_network_error && attempt < retry_config.max_retries {
                        last_error = Some(CallerError::from(e));
                        attempt += 1;
                        continue;
                    }
                    return Err(CallerError::from(e));
                }
            }
        }

        // All retries exhausted
        Err(last_error.unwrap_or_else(|| {
            CallerError::HttpError("All retry attempts exhausted".to_string())
        }))
    }

    pub async fn download(
        method: &str,
        params: Option<HashMap<String, String>>,
        extension: Option<String>,
    ) -> Result<DownloadResult, CallerError> {
        let context = CallerContext::build(method, params)?;
        let client = reqwest::Client::new();

        let mut rb = client
            .request(context.http_method, &context.url)
            .header(header::USER_AGENT, constants::UA);

        if let Some(params_map) = &context.params {
            // Parse parameter types (supports single type like "path" or combined like "path,json")
            let param_type_lower = context.api_item.param_type.to_lowercase();
            let param_types: Vec<&str> = param_type_lower.split(',').collect();

            // Handle each parameter type
            for param_type in param_types {
                match param_type.trim() {
                    "query" => rb = rb.query(params_map),
                    "json" => rb = rb.json(params_map),
                    "form" => rb = rb.form(params_map),
                    "path" => {
                        // Path parameters were already substituted in URL construction
                        // No additional processing needed for request body
                    }
                    "none" => {
                        // No parameters needed for the request body
                    }
                    unsupported => {
                        return Err(CallerError::parameter_error(format!(
                            "Unsupported parameter type: {}",
                            unsupported
                        )));
                    }
                }
            }
        }

        let response = rb.send().await?;
        let status_code = response.status();
        
        // Get content type from response headers before consuming the response
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Extract Content-Disposition header before consuming the response
        let content_disposition = response
            .headers()
            .get(header::CONTENT_DISPOSITION)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let content = response.bytes().await?.to_vec();

        let mut download_result = DownloadResult::from_response(status_code, content, content_type)?;

        // Override extension if manually specified
        if let Some(ext) = extension {
            download_result = download_result.with_extension(&ext);
        }

        // Try to extract filename from Content-Disposition header
        if let Some(disposition) = content_disposition
            && let Some(filename) = extract_filename_from_disposition(&disposition)
        {
            download_result = download_result.with_filename(filename);
        }

        Ok(download_result)
    }
}

pub(crate) fn split_method(method: &str) -> Result<Vec<String>, CallerError> {
    if !method.contains('.') {
        return Err(CallerError::invalid_method_format(method));
    }

    if method.starts_with('.') || method.ends_with('.') {
        return Err(CallerError::invalid_method_format(format!(
            "Method cannot start or end with dot: '{}'",
            method
        )));
    }

    let parts: Vec<&str> = method.split('.').collect();
    let result: Vec<String> = parts.iter().map(|s| s.to_string()).collect();

    if result.len() != 2 {
        return Err(CallerError::invalid_method_format(format!(
            "Method must be in format 'service.api', got: '{}'",
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

/// Extract filename from Content-Disposition header
/// Supports both "inline" and "attachment" disposition types
/// Handles both "filename=" and "filename*=" parameters
fn extract_filename_from_disposition(disposition: &str) -> Option<String> {
    // Try to extract filename from "filename=" parameter
    if let Some(start) = disposition.find("filename=") {
        let rest = &disposition[start + 9..];
        
        // Filename can be in quotes
        if let Some(stripped) = rest.strip_prefix('"') {
            if let Some(end) = stripped.find('"') {
                return Some(stripped[..end].to_string());
            }
        } else {
            // Filename without quotes - extract until semicolon or end
            let end = rest.find(';').unwrap_or(rest.len());
            let filename = rest[..end].trim();
            return Some(filename.to_string());
        }
    }

    // Try to extract from "filename*=" parameter (RFC 5987 encoding)
    if let Some(start) = disposition.find("filename*=") {
        let rest = &disposition[start + 10..];
        
        // Remove charset and encoding if present (e.g., "UTF-8''")
        if let Some(prefix_end) = rest.find("'")
            && let Some(encoding_end) = rest[prefix_end + 1..].find("'")
        {
            let encoded = &rest[prefix_end + encoding_end + 2..];
            
            // Decode percent-encoded characters
            if let Ok(decoded) = percent_decode(encoded) {
                return Some(decoded);
            }
        }
    }

    None
}

/// Decode percent-encoded string (URL encoding)
fn percent_decode(s: &str) -> Result<String, std::string::FromUtf8Error> {
    let mut bytes = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        if chars[i] == '%' && i + 2 < chars.len() {
            let hex = format!("{}{}", chars[i + 1], chars[i + 2]);
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                bytes.push(byte);
                i += 3;
                continue;
            }
        }
        bytes.push(chars[i] as u8);
        i += 1;
    }
    
    String::from_utf8(bytes)
}

pub fn get_http_method(http_method: &str) -> Result<Method, CallerError> {
    match http_method.to_lowercase().as_str() {
        "get" => Ok(Method::GET),
        "post" => Ok(Method::POST),
        "put" => Ok(Method::PUT),
        "delete" => Ok(Method::DELETE),
        "patch" => Ok(Method::PATCH),
        _ => Err(CallerError::http_method_not_supported(
            http_method.to_string(),
        )),
    }
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
