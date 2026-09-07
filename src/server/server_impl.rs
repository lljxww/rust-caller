//! Server implementation using axum

use crate::client::Caller;
use crate::config::config_loader::ConfigLoader;
use crate::domain::{ApiConfig, ResponseBody};
use crate::openapi::OpenApiGenerator;
use crate::server::ServerConfig;
use crate::shared::error::{CallerError, ErrorCategory};
use axum::{
    Router,
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, Method, StatusCode, header},
    response::{Html, IntoResponse, Json},
    routing::{any, get},
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

/// Server state
#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) caller: Caller,
    pub(crate) server_config: ServerConfig,
}

/// Start the API documentation server
pub async fn start_server(config: ServerConfig) -> Result<(), CallerError> {
    let caller = Caller::from_config(ConfigLoader::get_full_config()?)?;
    start_server_with_caller(config, caller).await
}

/// Start the API documentation server with a specific [`Caller`] instance.
pub async fn start_server_with_caller(
    config: ServerConfig,
    caller: Caller,
) -> Result<(), CallerError> {
    config.validate()?;

    let state = Arc::new(AppState {
        caller,
        server_config: config.clone(),
    });

    let mut app = Router::new()
        .route("/", get(index))
        .route("/openapi.json", get(openapi_json));
    if config.enable_proxy {
        app = app
            .route("/proxy/{service}/{method}", any(proxy_handler_path))
            .route("/proxy", any(proxy_handler_query));
    }
    if config.allow_any_origin {
        app = app.layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );
    }
    let app = app.with_state(state);

    let addr = config.addr;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|error| CallerError::io(format!("binding server to {addr}"), error))?;

    axum::serve(listener, app)
        .await
        .map_err(|error| CallerError::io("serving HTTP requests", error))?;

    Ok(())
}

/// Index page with Swagger UI
async fn index(State(state): State<Arc<AppState>>) -> Html<String> {
    let proxy_url = format!("http://{}", state.server_config.addr);
    Html(SWAGGER_UI_HTML.replace("{{PROXY_URL}}", &proxy_url))
}

/// OpenAPI JSON specification (with proxy mode enabled)
async fn openapi_json(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let proxy_url = format!("http://{}", state.server_config.addr);

    match OpenApiGenerator::from_caller(&state.caller) {
        Ok(generator) => {
            let doc = generator
                .title(&state.server_config.title)
                .version(&state.server_config.version)
                .proxy_mode(state.server_config.enable_proxy)
                .proxy_url(&proxy_url)
                .generate();
            Json(doc).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

/// Proxy query parameters
#[derive(Debug, Deserialize)]
struct ProxyParams {
    /// Optional path parameter value (for single path param APIs)
    id: Option<String>,
    /// Additional query parameters
    #[serde(flatten)]
    extra: HashMap<String, String>,
}

/// Proxy handler - path style: /proxy/{service}/{method}
async fn proxy_handler_path(
    incoming_method: Method,
    State(state): State<Arc<AppState>>,
    Path((service, method)): Path<(String, String)>,
    Query(params): Query<ProxyParams>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    execute_proxy(
        &state,
        ProxyRequest {
            service_name: &service,
            method_name: &method,
            id: params.id.as_deref(),
            extra_params: &params.extra,
            incoming_method: &incoming_method,
            headers: &headers,
            body: &body,
        },
    )
    .await
}

/// Proxy handler - query style: /proxy?service=X&method=Y
#[derive(Debug, Deserialize)]
struct ProxyQueryParams {
    service: String,
    method: String,
    id: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, String>,
}

async fn proxy_handler_query(
    incoming_method: Method,
    State(state): State<Arc<AppState>>,
    Query(params): Query<ProxyQueryParams>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    execute_proxy(
        &state,
        ProxyRequest {
            service_name: &params.service,
            method_name: &params.method,
            id: params.id.as_deref(),
            extra_params: &params.extra,
            incoming_method: &incoming_method,
            headers: &headers,
            body: &body,
        },
    )
    .await
}

struct ProxyRequest<'a> {
    service_name: &'a str,
    method_name: &'a str,
    id: Option<&'a str>,
    extra_params: &'a HashMap<String, String>,
    incoming_method: &'a Method,
    headers: &'a HeaderMap,
    body: &'a [u8],
}

/// Execute proxy request
async fn execute_proxy(
    state: &Arc<AppState>,
    request: ProxyRequest<'_>,
) -> (StatusCode, Json<serde_json::Value>) {
    let ProxyRequest {
        service_name,
        method_name,
        id,
        extra_params,
        incoming_method,
        headers,
        body,
    } = request;
    let config = match state.caller.config() {
        Ok(config) => config,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": err.to_string() })),
            );
        }
    };

    let service = match config
        .service_items
        .iter()
        .find(|s| s.api_name == service_name)
    {
        Some(s) => s,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": format!("Service '{}' not found", service_name),
                    "available_services": config.service_items.iter().map(|s| &s.api_name).collect::<Vec<_>>()
                })),
            );
        }
    };

    let api_config = match service.api_items.iter().find(|a| a.method == method_name) {
        Some(a) => a,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": format!("method '{}' not found in service '{}'", method_name, service_name),
                    "available_methods": service.api_items.iter().map(|a| &a.method).collect::<Vec<_>>()
                })),
            );
        }
    };

    if api_config.http_method.as_str() != incoming_method.as_str() {
        return (
            StatusCode::METHOD_NOT_ALLOWED,
            Json(serde_json::json!({
                "error": format!(
                    "method '{}' requires HTTP {}",
                    method_name,
                    api_config.http_method.as_str()
                )
            })),
        );
    }

    let mut params = extra_params.clone();
    if let Some(id) = id
        && let Some(path_param) = first_path_param(api_config)
    {
        params.entry(path_param).or_insert_with(|| id.to_string());
    }

    let method = format!("{}.{}", service_name, method_name);
    let param_types = match api_config.param_types() {
        Ok(param_types) => param_types,
        Err(error) => return proxy_error_response(error),
    };
    let mut args = crate::RequestArgs::new();
    if param_types.contains(&crate::ParamType::Path) {
        let mut path_params = HashMap::new();
        for name in path_param_names(api_config) {
            if let Some(value) = params.remove(&name) {
                path_params.insert(name, value);
            }
        }
        args = args.with_path(crate::CallParams::from_hashmap(path_params));
    }

    if param_types.contains(&crate::ParamType::Query) {
        args = args.with_query(crate::CallParams::from_hashmap(std::mem::take(&mut params)));
    }

    if param_types.contains(&crate::ParamType::Form) {
        let mut form_params = std::mem::take(&mut params);
        if !body.is_empty() {
            let content_type = headers
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default();
            if !content_type.starts_with("application/x-www-form-urlencoded") {
                return (
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    Json(serde_json::json!({
                        "error": "form endpoints require application/x-www-form-urlencoded"
                    })),
                );
            }
            form_params.extend(form_urlencoded::parse(body).into_owned());
        }
        args = args.with_form(crate::CallParams::from_hashmap(form_params));
    }

    if param_types.contains(&crate::ParamType::Json) {
        let json = if body.is_empty() {
            serde_json::Value::Null
        } else {
            match serde_json::from_slice(body) {
                Ok(value) => value,
                Err(error) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({ "error": error.to_string() })),
                    );
                }
            }
        };
        args = args.with_json(json);
    }

    if !params.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("unexpected parameters: {:?}", params.keys().collect::<Vec<_>>())
            })),
        );
    }

    let result = state.caller.call_args(&method, args).await;

    match result {
        Ok(result) => match result.body {
            ResponseBody::Json(json) => (result.status_code, Json(json)),
            ResponseBody::Text(text) => (
                result.status_code,
                Json(serde_json::json!({ "response": text })),
            ),
        },
        Err(err) => proxy_error_response(err),
    }
}

fn first_path_param(api_config: &ApiConfig) -> Option<String> {
    path_param_names(api_config).into_iter().next()
}

fn path_param_names(api_config: &ApiConfig) -> Vec<String> {
    let mut chars = api_config.url.chars();
    let mut names = Vec::new();

    while let Some(c) = chars.next() {
        if c != '{' {
            continue;
        }

        let mut name = String::new();
        for c in chars.by_ref() {
            if c == '}' {
                if !name.is_empty() {
                    names.push(name);
                }
                break;
            }
            name.push(c);
        }
    }

    names
}

fn proxy_error_response(err: CallerError) -> (StatusCode, Json<serde_json::Value>) {
    let status = match &err {
        CallerError::ServiceNotFound { .. } | CallerError::ApiNotFound { .. } => {
            StatusCode::NOT_FOUND
        }
        _ if err.category() == ErrorCategory::Protocol => StatusCode::BAD_REQUEST,
        _ if err.category() == ErrorCategory::Security => StatusCode::UNAUTHORIZED,
        _ => StatusCode::BAD_GATEWAY,
    };

    (
        status,
        Json(serde_json::json!({ "error": err.to_string() })),
    )
}

/// Swagger UI HTML (embedded)
const SWAGGER_UI_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Caller API Documentation</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
    <style>
        html { box-sizing: border-box; overflow: -moz-scrollbars-vertical; overflow-y: scroll; }
        *, *:before, *:after { box-sizing: inherit; }
        body { margin:0; padding:0; }
        .topbar { display: none; }
        .information-container { background: #f5f5f5; padding: 10px; margin-bottom: 10px; }
        .information-container .url { font-size: 12px; color: #666; }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-standalone-preset.js"></script>
    <script>
        window.onload = function() {
            const ui = SwaggerUIBundle({
                url: "/openapi.json",
                dom_id: '#swagger-ui',
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                layout: "StandaloneLayout",
                deepLinking: true,
                tryItOutEnabled: true,
                requestSnippetsEnabled: true,
                // All requests go through caller proxy
                requestInterceptor: function(request) {
                    console.log('[Caller Proxy] Request:', request.url);
                    return request;
                }
            });
        }
    </script>
</body>
</html>
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HttpMethod, ParamType};

    fn api(url: &str) -> ApiConfig {
        ApiConfig {
            method: "get".to_string(),
            url: url.to_string(),
            http_method: HttpMethod::Get,
            param_type: vec![ParamType::Path],
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
    fn extracts_first_path_param_for_proxy_id_mapping() {
        assert_eq!(
            first_path_param(&api("/users/{username}/repos")),
            Some("username".to_string())
        );
        assert_eq!(first_path_param(&api("/users")), None);
        assert_eq!(first_path_param(&api("/users/{}")), None);
    }

    #[test]
    fn maps_proxy_errors_to_http_statuses() {
        assert_eq!(
            proxy_error_response(CallerError::service_not_found("svc")).0,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            proxy_error_response(CallerError::unsupported_param_type("xml")).0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            proxy_error_response(CallerError::unknown_auth_provider("token")).0,
            StatusCode::UNAUTHORIZED
        );
    }
}
