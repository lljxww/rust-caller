use crate::shared::error::CallerError;
use reqwest::StatusCode;
use serde_json::Value;

const SPLIT_OPERATOR: &str = ".";

#[derive(Debug, Clone)]
pub enum ResponseBody {
    Json(Value),
    Text(String),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct ApiResult {
    pub status_code: StatusCode,
    pub raw: String,
    pub body: ResponseBody,
    pub json: Value,
}

impl ApiResult {
    pub fn build(raw: String, status_code: StatusCode) -> Result<ApiResult, CallerError> {
        if let Ok(json) = serde_json::from_str::<Value>(&raw) {
            return Ok(ApiResult {
                status_code,
                raw,
                body: ResponseBody::Json(json.clone()),
                json,
            });
        }

        Ok(ApiResult {
            status_code,
            body: ResponseBody::Text(raw.clone()),
            raw,
            json: Value::Null,
        })
    }

    pub fn build_json(raw: String, status_code: StatusCode) -> Result<ApiResult, CallerError> {
        let json: Value = serde_json::from_str(&raw)
            .map_err(|e| CallerError::JsonError(format!("Failed to parse JSON response: {}", e)))?;

        Ok(ApiResult {
            status_code,
            raw,
            body: ResponseBody::Json(json.clone()),
            json,
        })
    }

    pub fn is_json(&self) -> bool {
        matches!(self.body, ResponseBody::Json(_))
    }

    pub fn is_text(&self) -> bool {
        matches!(self.body, ResponseBody::Text(_))
    }

    pub fn json(&self) -> Option<&Value> {
        match &self.body {
            ResponseBody::Json(json) => Some(json),
            _ => None,
        }
    }

    pub fn text(&self) -> Option<&str> {
        match &self.body {
            ResponseBody::Json(_) => Some(&self.raw),
            ResponseBody::Text(text) => Some(text.as_str()),
            ResponseBody::Bytes(_) => None,
        }
    }

    fn value_at_path(&self, keys: Vec<&str>) -> Option<&Value> {
        if keys.is_empty() {
            return None;
        }

        let mut value = Some(&self.json);
        for key in keys {
            match value {
                Some(v) => {
                    // Try to parse key as a number for array indexing
                    if v.is_array() {
                        if let Ok(index) = key.parse::<usize>() {
                            value = v.get(index);
                        } else {
                            // Key is not a valid number, try object key access
                            value = v.get(key);
                        }
                    } else {
                        // Not an array, use object key access
                        value = v.get(key);
                    }
                }
                None => return None,
            }
        }

        value
    }

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

    pub fn value_at(&self, key: &str) -> Option<&Value> {
        if !self.is_json() {
            return None;
        }
        let keys: Vec<&str> = key.split(SPLIT_OPERATOR).collect();
        self.value_at_path(keys)
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.value_at(key)
    }

    pub fn get_as_str(&self, key: &str) -> Option<&str> {
        self.str_at(key)
    }

    pub fn get_as_bool(&self, key: &str) -> Option<bool> {
        self.bool_at(key)
    }

    pub fn get_as_i64(&self, key: &str) -> Option<i64> {
        self.i64_at(key)
    }

    pub fn get_as_f64(&self, key: &str) -> Option<f64> {
        self.f64_at(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_accepts_text_response_without_json_error() {
        let result = ApiResult::build("plain text".to_string(), StatusCode::OK).unwrap();

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
}
