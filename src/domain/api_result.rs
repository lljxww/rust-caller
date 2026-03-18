use crate::shared::error::CallerError;
use reqwest::StatusCode;
use serde_json::Value;

const SPLIT_OPERATOR: &str = ".";

#[derive(Debug, Clone)]
pub struct ApiResult {
    pub status_code: StatusCode,
    pub raw: String,
    pub j_obj: Value,
}

impl ApiResult {
    pub fn build(raw: String, status_code: StatusCode) -> Result<ApiResult, CallerError> {
        let j_obj: Value = serde_json::from_str(&raw)
            .map_err(|e| CallerError::JsonError(format!("Failed to parse JSON response: {}", e)))?;

        Ok(ApiResult {
            status_code,
            raw,
            j_obj,
        })
    }

    fn get_deep(&self, keys: Vec<&str>) -> Option<&Value> {
        if keys.is_empty() {
            return None;
        }

        let mut value = Some(&self.j_obj);
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

    pub fn get_as_str(&self, key: &str) -> Option<&str> {
        if key.contains(SPLIT_OPERATOR) {
            let keys: Vec<&str> = key.split(SPLIT_OPERATOR).collect();
            self.get_deep(keys).and_then(|v| v.as_str())
        } else {
            self.j_obj.get(key).and_then(Value::as_str)
        }
    }

    pub fn get_as_bool(&self, key: &str) -> Option<bool> {
        if key.contains(SPLIT_OPERATOR) {
            let keys: Vec<&str> = key.split(SPLIT_OPERATOR).collect();
            self.get_deep(keys).and_then(|v| v.as_bool())
        } else {
            self.j_obj.get(key).and_then(Value::as_bool)
        }
    }

    pub fn get_as_i64(&self, key: &str) -> Option<i64> {
        if key.contains(SPLIT_OPERATOR) {
            let keys: Vec<&str> = key.split(SPLIT_OPERATOR).collect();
            self.get_deep(keys).and_then(|v| v.as_i64())
        } else {
            self.j_obj.get(key).and_then(Value::as_i64)
        }
    }

    pub fn get_as_f64(&self, key: &str) -> Option<f64> {
        if key.contains(SPLIT_OPERATOR) {
            let keys: Vec<&str> = key.split(SPLIT_OPERATOR).collect();
            self.get_deep(keys).and_then(|v| v.as_f64())
        } else {
            self.j_obj.get(key).and_then(Value::as_f64)
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        let keys: Vec<&str> = key.split(SPLIT_OPERATOR).collect();
        self.get_deep(keys)
    }
}
