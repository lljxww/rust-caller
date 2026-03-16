//! Server implementation using axum

use crate::config::config_loader::ConfigLoader;
use crate::domain::caller_config::CallerConfig;
use crate::openapi::OpenApiGenerator;
use crate::server::ServerConfig;
use crate::shared::error::CallerError;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

/// Server state
#[derive(Clone)]
pub struct AppState {
    pub config: CallerConfig,
    pub server_config: ServerConfig,
}

/// Start the API documentation server
pub async fn start_server(config: ServerConfig) -> Result<(), CallerError> {
    let caller_config = ConfigLoader::get_full_config()?;
    let state = Arc::new(AppState {
        config: caller_config,
        server_config: config.clone(),
    });

    let addr_str = format!("{}", config.addr);
    
    let app = Router::new()
        .route("/", get(index))
        .route("/openapi.json", get(openapi_json))
        .route("/proxy/{service}/{method}", get(proxy_handler_path))
        .route("/proxy", get(proxy_handler_query))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let addr = config.addr;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| CallerError::IoError(format!("Failed to bind to {}: {}", addr, e)))?;

    println!("🚀 Caller API Server running at http://{}", addr);
    println!("📖 Swagger UI: http://{}/", addr);
    println!("📄 OpenAPI JSON: http://{}/openapi.json", addr);
    println!("🔄 Proxy: http://{}/proxy/{{service}}/{{method}}?id=VALUE", addr);
    println!("");
    println!("💡 Swagger UI 'Try it out' requests will go through caller proxy!");

    axum::serve(listener, app)
        .await
        .map_err(|e| CallerError::IoError(format!("Server error: {}", e)))?;

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
    
    let generator = OpenApiGenerator::new(state.config.clone())
        .title(&state.server_config.title)
        .version(&state.server_config.version)
        .proxy_mode(true)
        .proxy_url(&proxy_url);

    let doc = generator.generate();
    Json(doc)
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
    State(state): State<Arc<AppState>>,
    Path((service, method)): Path<(String, String)>,
    Query(params): Query<ProxyParams>,
) -> impl IntoResponse {
    execute_proxy(&state, &service, &method, params.id.as_deref(), &params.extra).await
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
    State(state): State<Arc<AppState>>,
    Query(params): Query<ProxyQueryParams>,
) -> impl IntoResponse {
    execute_proxy(&state, &params.service, &params.method, params.id.as_deref(), &params.extra).await
}

/// Execute proxy request
async fn execute_proxy(
    state: &Arc<AppState>,
    service_name: &str,
    method_name: &str,
    id: Option<&str>,
    extra_params: &HashMap<String, String>,
) -> (StatusCode, Json<serde_json::Value>) {
    // Find service and API item
    let service = match state
        .config
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
                    "available_services": state.config.service_items.iter().map(|s| &s.api_name).collect::<Vec<_>>()
                })),
            );
        }
    };

    let api_item = match service.api_items.iter().find(|a| a.method == method_name) {
        Some(a) => a,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": format!("Method '{}' not found in service '{}'", method_name, service_name),
                    "available_methods": service.api_items.iter().map(|a| &a.method).collect::<Vec<_>>()
                })),
            );
        }
    };

    // Build target URL
    let mut url = format!("{}{}", service.base_url, api_item.url);

    // Handle path parameters
    if url.contains('{') {
        // Replace path parameters
        if let Some(id_val) = id {
            // Replace first path parameter with id value
            if let Some(start) = url.find('{') {
                if let Some(end) = url.find('}') {
                    url = format!("{}{}{}", &url[..start], id_val, &url[end + 1..]);
                }
            }
        }
        // Replace any remaining path params from extra_params
        for (key, value) in extra_params {
            let placeholder = format!("{{{}}}", key);
            if url.contains(&placeholder) {
                url = url.replace(&placeholder, value);
            }
        }
    }

    // Build request
    let timeout_ms = api_item.timeout.unwrap_or(30_000);
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(timeout_ms as u64))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to create HTTP client: {}", e)})),
            );
        }
    };

    let mut request = match api_item.http_method.to_lowercase().as_str() {
        "get" => client.get(&url),
        "post" => client.post(&url),
        "put" => client.put(&url),
        "delete" => client.delete(&url),
        "patch" => client.patch(&url),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Unsupported HTTP method: {}", api_item.http_method)})),
            );
        }
    };

    // Add query parameters for query type APIs
    let param_type_lower = api_item.param_type.to_lowercase();
    if param_type_lower.contains("query") {
        // Filter out path params, add rest as query
        let query_params: HashMap<&String, &String> = extra_params
            .iter()
            .filter(|(k, _)| !url.contains(&format!("{{{}}}", k)))
            .collect();
        if !query_params.is_empty() {
            request = request.query(&query_params);
        }
    }

    // Execute request
    match request.send().await {
        Ok(response) => {
            let status = response.status();
            match response.text().await {
                Ok(body) => {
                    // Try to parse as JSON for pretty output
                    match serde_json::from_str::<serde_json::Value>(&body) {
                        Ok(json) => (status, Json(json)),
                        Err(_) => (status, Json(serde_json::json!({"response": body}))),
                    }
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("Failed to read response: {}", e)})),
                ),
            }
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "error": format!("Request failed: {}", e),
                "url": url
            })),
        ),
    }
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
