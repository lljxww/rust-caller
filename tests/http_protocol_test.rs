use async_trait::async_trait;
use caller::{
    ApiConfig, Caller, CallerConfig, CallerError, DynamicBearerAuth, HeaderMiddleware, HttpMethod,
    Middleware, OAuth2Auth, ParamType, RequestArgs, RequestContext, ResponseContext, RetryConfig,
    ServiceConfig, TransportErrorKind, params,
};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn spawn_server(
    response_headers: &[(&str, &str)],
    response_body: &str,
    response_delay: Duration,
) -> (String, Receiver<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let headers = response_headers
        .iter()
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect::<Vec<_>>();
    let response_body = response_body.as_bytes().to_vec();
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();

        let mut request = Vec::new();
        let mut buffer = [0_u8; 4096];
        let mut expected_length = None;

        loop {
            let count = stream.read(&mut buffer).unwrap();
            request.extend_from_slice(&buffer[..count]);

            if expected_length.is_none()
                && let Some(header_end) = find_bytes(&request, b"\r\n\r\n")
            {
                let headers = String::from_utf8_lossy(&request[..header_end]);
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().ok())
                            .flatten()
                    })
                    .unwrap_or(0);
                expected_length = Some(header_end + 4 + content_length);
            }

            if expected_length.is_some_and(|length| request.len() >= length) {
                break;
            }
        }

        sender.send(request).unwrap();
        thread::sleep(response_delay);

        let mut response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n",
            response_body.len()
        );
        for (name, value) in headers {
            response.push_str(&format!("{name}: {value}\r\n"));
        }
        response.push_str("\r\n");
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.write_all(&response_body);
    });

    (format!("http://{address}"), receiver)
}

fn spawn_retry_server() -> (String, Receiver<usize>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        for attempt in 0..2 {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut request = Vec::new();
            let mut buffer = [0_u8; 1024];
            while find_bytes(&request, b"\r\n\r\n").is_none() {
                let count = stream.read(&mut buffer).unwrap();
                request.extend_from_slice(&buffer[..count]);
            }

            let response = if attempt == 0 {
                "HTTP/1.1 503 Service Unavailable\r\nRetry-After: 0\r\nContent-Length: 5\r\nConnection: close\r\n\r\nretry"
            } else {
                "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok"
            };
            stream.write_all(response.as_bytes()).unwrap();
        }
        sender.send(2).unwrap();
    });

    (format!("http://{address}"), receiver)
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

struct TestEndpoint<'a> {
    http_method: HttpMethod,
    path: &'a str,
    param_types: Vec<ParamType>,
    service_timeout: Option<u32>,
    endpoint_timeout: Option<u32>,
    content_type: Option<&'a str>,
    auth_type: Option<&'a str>,
}

fn caller_for(base_url: String, endpoint: TestEndpoint<'_>) -> Caller {
    Caller::from_config(CallerConfig {
        authorizations: vec![],
        service_items: vec![ServiceConfig {
            api_name: "test".to_string(),
            authorization_type: endpoint.auth_type.map(str::to_string),
            base_url,
            timeout: endpoint.service_timeout,
            api_items: vec![ApiConfig {
                method: "request".to_string(),
                url: endpoint.path.to_string(),
                http_method: endpoint.http_method,
                param_type: endpoint.param_types,
                description: None,
                need_cache: None,
                cache_time: None,
                content_type: endpoint.content_type.map(str::to_string),
                authorization_type: None,
                timeout: endpoint.endpoint_timeout,
                use_new_http_client: None,
            }],
            use_new_http_client: None,
        }],
    })
    .unwrap()
}

fn request_text(receiver: Receiver<Vec<u8>>) -> String {
    String::from_utf8(receiver.recv_timeout(Duration::from_secs(2)).unwrap()).unwrap()
}

#[tokio::test]
async fn typed_json_parameters_preserve_wire_types() {
    let (base_url, request) = spawn_server(
        &[("Content-Type", "application/json")],
        "{}",
        Duration::ZERO,
    );
    let caller = caller_for(
        base_url,
        TestEndpoint {
            http_method: HttpMethod::Post,
            path: "/items",
            param_types: vec![ParamType::Json],
            service_timeout: None,
            endpoint_timeout: None,
            content_type: None,
            auth_type: None,
        },
    );

    caller
        .call_params(
            "test.request",
            Some(params! {
                "name" => "Alice",
                "age" => 30,
                "active" => true,
                "ids" => vec![1, 2, 3],
                "large" => u64::MAX,
            }),
        )
        .await
        .unwrap();

    let request = request_text(request);
    let (_, body) = request.split_once("\r\n\r\n").unwrap();
    let body: serde_json::Value = serde_json::from_str(body).unwrap();
    assert_eq!(body["name"], "Alice");
    assert_eq!(body["age"], 30);
    assert_eq!(body["active"], true);
    assert_eq!(body["ids"], serde_json::json!([1, 2, 3]));
    assert_eq!(body["large"], serde_json::json!(u64::MAX));
}

#[tokio::test]
async fn path_values_are_encoded_and_get_has_no_implicit_content_type() {
    let (base_url, request) = spawn_server(&[], "ok", Duration::ZERO);
    let caller = caller_for(
        base_url,
        TestEndpoint {
            http_method: HttpMethod::Get,
            path: "/items/{id}",
            param_types: vec![ParamType::Path],
            service_timeout: None,
            endpoint_timeout: None,
            content_type: None,
            auth_type: None,
        },
    );

    caller
        .call(
            "test.request",
            Some([("id".to_string(), "a/b ?".to_string())].into()),
        )
        .await
        .unwrap();

    let request = request_text(request);
    assert!(request.starts_with("GET /items/a%2Fb%20%3F HTTP/1.1\r\n"));
    assert!(!request.to_ascii_lowercase().contains("content-type:"));
}

#[tokio::test]
async fn endpoint_content_type_is_sent_and_service_timeout_is_inherited() {
    let (base_url, request) = spawn_server(&[], "ok", Duration::from_millis(150));
    let caller = caller_for(
        base_url,
        TestEndpoint {
            http_method: HttpMethod::Post,
            path: "/items",
            param_types: vec![ParamType::Json],
            service_timeout: Some(30),
            endpoint_timeout: None,
            content_type: Some("application/vnd.example+json"),
            auth_type: None,
        },
    );

    let error = caller
        .call_params("test.request", Some(params! { "value" => 1 }))
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        CallerError::HttpTransport {
            kind: TransportErrorKind::Timeout,
            ..
        }
    ));

    let request = request_text(request).to_ascii_lowercase();
    assert!(request.contains("content-type: application/vnd.example+json\r\n"));
}

#[tokio::test]
async fn oauth2_uses_authorization_header_and_dynamic_env_is_fallible() {
    let (base_url, request) = spawn_server(&[], "ok", Duration::ZERO);
    let caller = caller_for(
        base_url,
        TestEndpoint {
            http_method: HttpMethod::Get,
            path: "/items",
            param_types: vec![ParamType::None],
            service_timeout: None,
            endpoint_timeout: None,
            content_type: None,
            auth_type: Some("oauth"),
        },
    );
    caller
        .register_auth("oauth", OAuth2Auth::new("secret-token".to_string()))
        .unwrap();

    caller.call("test.request", None).await.unwrap();
    let request = request_text(request);
    assert!(request.contains("authorization: Bearer secret-token\r\n"));

    let missing_name = format!(
        "CALLER_MISSING_AUTH_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let caller = caller_for(
        "http://127.0.0.1:9".to_string(),
        TestEndpoint {
            http_method: HttpMethod::Get,
            path: "/items",
            param_types: vec![ParamType::None],
            service_timeout: None,
            endpoint_timeout: None,
            content_type: None,
            auth_type: Some("dynamic"),
        },
    );
    caller
        .register_auth("dynamic", DynamicBearerAuth::from_env(&missing_name))
        .unwrap();
    let error = caller.call("test.request", None).await.unwrap_err();
    assert!(matches!(
        error,
        CallerError::MissingAuthEnvironmentVariable { name } if name == missing_name
    ));
}

#[test]
fn sensitive_auth_debug_output_is_redacted() {
    let bearer = caller::BearerAuth::new("never-log-me".to_string());
    let basic = caller::BasicAuth::new("user".to_string(), "secret-password".to_string());
    let api_key = caller::ApiKeyAuth::new("x-api-key".to_string(), "secret-key".to_string());

    let output = format!("{bearer:?} {basic:?} {api_key:?}");
    assert!(!output.contains("never-log-me"));
    assert!(!output.contains("secret-password"));
    assert!(!output.contains("secret-key"));
    assert!(output.contains("[REDACTED]"));
}

struct RewriteResponse;

#[async_trait]
impl Middleware for RewriteResponse {
    async fn after_response(&self, context: &mut ResponseContext) -> Result<(), CallerError> {
        context.body = r#"{"rewritten":true}"#.to_string();
        Ok(())
    }
}

#[tokio::test]
async fn caller_executes_middleware_against_the_wire_request_and_response() {
    let (base_url, request) = spawn_server(
        &[("Content-Type", "application/json")],
        r#"{"original":true}"#,
        Duration::ZERO,
    );
    let base = caller_for(
        base_url,
        TestEndpoint {
            http_method: HttpMethod::Get,
            path: "/items",
            param_types: vec![ParamType::None],
            service_timeout: None,
            endpoint_timeout: None,
            content_type: None,
            auth_type: None,
        },
    );
    let caller = Caller::builder()
        .config(base.config().unwrap())
        .middleware(
            HeaderMiddleware::new()
                .with_header("x-middleware", "applied")
                .unwrap(),
        )
        .middleware(RewriteResponse)
        .build()
        .unwrap();

    let response = caller.call("test.request", None).await.unwrap();

    assert_eq!(response.json().unwrap()["rewritten"], true);
    assert_eq!(response.headers["content-type"], "application/json");
    assert!(response.duration < Duration::from_secs(1));
    let request = request_text(request);
    assert!(request.contains("x-middleware: applied\r\n"));
}

#[tokio::test]
async fn request_args_separate_path_query_and_json_body() {
    let (base_url, request) = spawn_server(&[], "ok", Duration::ZERO);
    let caller = caller_for(
        base_url,
        TestEndpoint {
            http_method: HttpMethod::Patch,
            path: "/items/{id}",
            param_types: vec![ParamType::Path, ParamType::Query, ParamType::Json],
            service_timeout: None,
            endpoint_timeout: None,
            content_type: None,
            auth_type: None,
        },
    );
    let args = RequestArgs::new()
        .with_path(params! { "id" => "a/b" })
        .with_query(params! { "expand" => true })
        .with_json(serde_json::json!({ "name": "updated" }));

    caller.call_args("test.request", args).await.unwrap();

    let request = request_text(request);
    assert!(request.starts_with("PATCH /items/a%2Fb?expand=true HTTP/1.1\r\n"));
    let (_, body) = request.split_once("\r\n\r\n").unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(body).unwrap(),
        serde_json::json!({ "name": "updated" })
    );
}

#[tokio::test]
async fn request_args_preserve_duplicate_query_keys() {
    let (base_url, request) = spawn_server(&[], "ok", Duration::ZERO);
    let caller = caller_for(
        base_url,
        TestEndpoint {
            http_method: HttpMethod::Get,
            path: "/items",
            param_types: vec![ParamType::Query],
            service_timeout: None,
            endpoint_timeout: None,
            content_type: None,
            auth_type: None,
        },
    );
    let args = RequestArgs::new().with_query_pairs([("tag", "rust"), ("tag", "http")]);

    caller.call_args("test.request", args).await.unwrap();

    let request = request_text(request);
    assert!(request.starts_with("GET /items?tag=rust&tag=http HTTP/1.1\r\n"));
}

#[tokio::test]
async fn response_body_limit_rejects_oversized_payload() {
    let (base_url, request) = spawn_server(&[], "0123456789", Duration::ZERO);
    let base = caller_for(
        base_url,
        TestEndpoint {
            http_method: HttpMethod::Get,
            path: "/items",
            param_types: vec![ParamType::None],
            service_timeout: None,
            endpoint_timeout: None,
            content_type: None,
            auth_type: None,
        },
    );
    let caller = Caller::builder()
        .config(base.config().unwrap())
        .max_response_body_bytes(5)
        .build()
        .unwrap();

    let error = caller.call("test.request", None).await.unwrap_err();
    assert!(matches!(
        error,
        CallerError::ResponseBodyTooLarge {
            limit_bytes: 5,
            received_bytes: 10
        }
    ));
    let _ = request_text(request);
}

struct CountRequests(Arc<AtomicUsize>);

#[async_trait]
impl Middleware for CountRequests {
    async fn before_request(&self, _context: &mut RequestContext) -> Result<(), CallerError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn retry_does_not_repeat_authentication_errors() {
    let base = caller_for(
        "http://127.0.0.1:9".to_string(),
        TestEndpoint {
            http_method: HttpMethod::Get,
            path: "/items",
            param_types: vec![ParamType::None],
            service_timeout: None,
            endpoint_timeout: None,
            content_type: None,
            auth_type: Some("missing"),
        },
    );
    let attempts = Arc::new(AtomicUsize::new(0));
    let caller = Caller::builder()
        .config(base.config().unwrap())
        .middleware(CountRequests(attempts.clone()))
        .build()
        .unwrap();

    let error = caller
        .call_with_retry("test.request", None, RetryConfig::new().with_max_retries(3))
        .await
        .unwrap_err();

    assert!(matches!(error, CallerError::UnknownAuthProvider { .. }));
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn retry_repeats_configured_status_and_returns_final_response() {
    let (base_url, attempts) = spawn_retry_server();
    let caller = caller_for(
        base_url,
        TestEndpoint {
            http_method: HttpMethod::Get,
            path: "/items",
            param_types: vec![ParamType::None],
            service_timeout: None,
            endpoint_timeout: None,
            content_type: None,
            auth_type: None,
        },
    );

    let response = caller
        .call_with_retry(
            "test.request",
            None,
            RetryConfig::new()
                .with_max_retries(1)
                .with_base_delay(Duration::from_secs(1)),
        )
        .await
        .unwrap();

    assert_eq!(response.status_code, 200);
    assert_eq!(response.text(), Some("ok"));
    assert_eq!(attempts.recv_timeout(Duration::from_secs(2)).unwrap(), 2);
}
