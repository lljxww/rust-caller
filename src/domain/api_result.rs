use crate::shared::error::CallerError;
use reqwest::StatusCode;
use reqwest::header::HeaderMap;
use serde_json::Value;
use std::time::Duration;

const SPLIT_OPERATOR: &str = ".";

#[derive(Debug, Clone)]
/// Decoded representation of a buffered response body.
pub enum ResponseBody {
    /// A body that parsed successfully as JSON.
    Json(Value),
    /// A non-JSON body represented as text.
    Text(String),
}

#[derive(Debug, Clone)]
/// Buffered HTTP response returned by the configured call APIs.
pub struct ApiResult {
    /// HTTP response status.
    pub status_code: StatusCode,
    /// Complete decoded response text.
    pub raw: String,
    /// JSON or text classification of the response body.
    pub body: ResponseBody,
    /// Compatibility view of the parsed JSON, or [`Value::Null`] for text.
    pub json: Value,
    /// Response headers after response middleware has run.
    pub headers: HeaderMap,
    /// Time spent sending and reading the response.
    pub duration: Duration,
}

impl ApiResult {
    /// Build a response and classify its body as JSON when parsing succeeds.
    ///
    /// Manually constructed values start with empty headers and zero duration.
    pub fn build(raw: String, status_code: StatusCode) -> ApiResult {
        if let Ok(json) = serde_json::from_str::<Value>(&raw) {
            return ApiResult {
                status_code,
                raw,
                body: ResponseBody::Json(json.clone()),
                json,
                headers: HeaderMap::new(),
                duration: Duration::ZERO,
            };
        }

        ApiResult {
            status_code,
            body: ResponseBody::Text(raw.clone()),
            raw,
            json: Value::Null,
            headers: HeaderMap::new(),
            duration: Duration::ZERO,
        }
    }

    /// Build a response that requires the supplied body to contain valid JSON.
    pub fn build_json(raw: String, status_code: StatusCode) -> Result<ApiResult, CallerError> {
        let json: Value = serde_json::from_str(&raw)
            .map_err(|e| CallerError::JsonError(format!("Failed to parse JSON response: {}", e)))?;

        Ok(ApiResult {
            status_code,
            raw,
            body: ResponseBody::Json(json.clone()),
            json,
            headers: HeaderMap::new(),
            duration: Duration::ZERO,
        })
    }

    pub(crate) fn build_with_metadata(
        raw: String,
        status_code: StatusCode,
        headers: HeaderMap,
        duration: Duration,
    ) -> ApiResult {
        let mut result = Self::build(raw, status_code);
        result.headers = headers;
        result.duration = duration;
        result
    }

    /// Return whether the buffered body was parsed as JSON.
    pub fn is_json(&self) -> bool {
        matches!(self.body, ResponseBody::Json(_))
    }

    /// Return whether the status code is in the 2xx range.
    pub fn is_success(&self) -> bool {
        self.status_code.is_success()
    }

    /// Convert a non-2xx response into a structured error while retaining a
    /// bounded body preview. Redirects remain successful return values because
    /// redirect policy belongs to the underlying HTTP client.
    pub fn error_for_status(self) -> Result<Self, CallerError> {
        if !self.status_code.is_client_error() && !self.status_code.is_server_error() {
            return Ok(self);
        }

        Err(CallerError::HttpStatus {
            status: self.status_code.as_u16(),
            body_preview: body_preview(&self.raw, 1024),
        })
    }

    /// Return whether the buffered body is represented as text rather than JSON.
    pub fn is_text(&self) -> bool {
        matches!(self.body, ResponseBody::Text(_))
    }

    /// Return the parsed JSON body, if this response contains JSON.
    pub fn json(&self) -> Option<&Value> {
        match &self.body {
            ResponseBody::Json(json) => Some(json),
            _ => None,
        }
    }

    /// Return the decoded response text for either body classification.
    pub fn text(&self) -> Option<&str> {
        match &self.body {
            ResponseBody::Json(_) => Some(&self.raw),
            ResponseBody::Text(text) => Some(text.as_str()),
        }
    }

    fn value_at_path(&self, keys: Vec<&str>) -> Option<&Value> {
        if keys.is_empty() {
            return None;
        }

        let mut value = Some(&self.json);
        for key in keys {
            let current = value?;
            if current.is_array() {
                value = key
                    .parse::<usize>()
                    .ok()
                    .and_then(|index| current.get(index));
            } else {
                value = current.get(key);
            }
        }

        value
    }

    /// Read a string from a dot-separated JSON object/array path.
    pub fn str_at(&self, key: &str) -> Option<&str> {
        if !self.is_json() {
            return None;
        }
        if key.contains(SPLIT_OPERATOR) {
            let keys: Vec<&str> = key.split(SPLIT_OPERATOR).collect();
            self.value_at_path(keys).and_then(|v| v.as_str())
        } else {
            self.json.get(key).and_then(Value::as_str)
        }
    }

    /// Read a boolean from a dot-separated JSON object/array path.
    pub fn bool_at(&self, key: &str) -> Option<bool> {
        if !self.is_json() {
            return None;
        }
        if key.contains(SPLIT_OPERATOR) {
            let keys: Vec<&str> = key.split(SPLIT_OPERATOR).collect();
            self.value_at_path(keys).and_then(|v| v.as_bool())
        } else {
            self.json.get(key).and_then(Value::as_bool)
        }
    }

    /// Read a signed integer from a dot-separated JSON object/array path.
    pub fn i64_at(&self, key: &str) -> Option<i64> {
        if !self.is_json() {
            return None;
        }
        if key.contains(SPLIT_OPERATOR) {
            let keys: Vec<&str> = key.split(SPLIT_OPERATOR).collect();
            self.value_at_path(keys).and_then(|v| v.as_i64())
        } else {
            self.json.get(key).and_then(Value::as_i64)
        }
    }

    /// Read a floating-point number from a dot-separated JSON object/array path.
    pub fn f64_at(&self, key: &str) -> Option<f64> {
        if !self.is_json() {
            return None;
        }
        if key.contains(SPLIT_OPERATOR) {
            let keys: Vec<&str> = key.split(SPLIT_OPERATOR).collect();
            self.value_at_path(keys).and_then(|v| v.as_f64())
        } else {
            self.json.get(key).and_then(Value::as_f64)
        }
    }

    /// Read any value from a dot-separated JSON object/array path.
    pub fn value_at(&self, key: &str) -> Option<&Value> {
        if !self.is_json() {
            return None;
        }
        let keys: Vec<&str> = key.split(SPLIT_OPERATOR).collect();
        self.value_at_path(keys)
    }

    /// Compatibility alias for [`Self::value_at`].
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.value_at(key)
    }

    /// Compatibility alias for [`Self::str_at`].
    pub fn get_as_str(&self, key: &str) -> Option<&str> {
        self.str_at(key)
    }

    /// Compatibility alias for [`Self::bool_at`].
    pub fn get_as_bool(&self, key: &str) -> Option<bool> {
        self.bool_at(key)
    }

    /// Compatibility alias for [`Self::i64_at`].
    pub fn get_as_i64(&self, key: &str) -> Option<i64> {
        self.i64_at(key)
    }

    /// Compatibility alias for [`Self::f64_at`].
    pub fn get_as_f64(&self, key: &str) -> Option<f64> {
        self.f64_at(key)
    }
}

fn body_preview(body: &str, max_chars: usize) -> String {
    let mut chars = body.chars();
    let preview: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{preview}…")
    } else {
        preview
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_accepts_text_response_without_json_error() {
        let result = ApiResult::build("plain text".to_string(), StatusCode::OK);

        assert!(result.is_text());
        assert!(!result.is_json());
        assert_eq!(result.text(), Some("plain text"));
        assert_eq!(result.json(), None);
        assert_eq!(result.str_at("anything"), None);
    }

    #[test]
    fn build_json_preserves_strict_json_validation() {
        let err = ApiResult::build_json("{invalid".to_string(), StatusCode::OK).unwrap_err();
        assert!(matches!(err, CallerError::JsonError(_)));
    }

    #[test]
    fn error_for_status_bounds_body_preview() {
        let body = "x".repeat(2_000);
        let error = ApiResult::build(body, StatusCode::BAD_GATEWAY)
            .error_for_status()
            .unwrap_err();

        match error {
            CallerError::HttpStatus {
                status,
                body_preview,
            } => {
                assert_eq!(status, 502);
                assert!(body_preview.chars().count() <= 1_025);
            }
            other => panic!("expected HttpStatus, got {other:?}"),
        }
    }

    #[test]
    fn error_for_status_accepts_redirects() {
        let result = ApiResult::build(String::new(), StatusCode::NOT_MODIFIED);
        assert!(result.error_for_status().is_ok());
    }
}
