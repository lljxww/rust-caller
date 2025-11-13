use reqwest::StatusCode;
use serde_json::Value;
use crate::shared::error::CallerError;

const SPLIT_OPERATOR: &'static str = ".";

#[derive(Debug, Clone)]
pub struct ApiResult {
    pub status_code: StatusCode,
    pub raw: String,
    pub j_obj: Value,
}

impl ApiResult {
    pub fn build(raw: String, status_code: StatusCode) -> Result<ApiResult, CallerError> {
        let j_obj: Value = serde_json::from_str(&raw)
            .map_err(|e| CallerError::JsonError(format!(
                "Failed to parse JSON response: {}", e
            )))?;

        Ok(ApiResult {
            status_code,
            raw,
            j_obj,
        })
    }

    fn get_deep(&self, keys: Vec<&str>) -> Option<&Value> {
        if keys.len() == 0 {
            return None;
        }

        let mut value = Some(&self.j_obj);
        for key in keys {
            match value {
                Some(v) => value = v.get(key),
                None => return None,
            }
        }

        return value;
    }

    pub fn get_as_str(&self, key: &str) -> Option<&str> {
        if key.contains(SPLIT_OPERATOR) {
            let keys: Vec<&str> = key.split(SPLIT_OPERATOR).collect();
            match self.get_deep(keys) {
                Some(value) => value.as_str(),
                None => None,
            }
        } else {
            self.j_obj.get(key).and_then(Value::as_str)
        }
    }

    pub fn get_as_bool(&self, key: &str) -> Option<bool> {
        if key.contains(SPLIT_OPERATOR) {
            let keys: Vec<&str> = key.split(SPLIT_OPERATOR).collect();
            match self.get_deep(keys) {
                Some(value) => value.as_bool(),
                None => None,
            }
        } else {
            self.j_obj.get(key).and_then(Value::as_bool)
        }
    }

    pub fn get_as_i64(&self, key: &str) -> Option<i64> {
        if key.contains(SPLIT_OPERATOR) {
            let keys: Vec<&str> = key.split(SPLIT_OPERATOR).collect();
            match self.get_deep(keys) {
                Some(value) => value.as_i64(),
                None => None,
            }
        } else {
            self.j_obj.get(key).and_then(Value::as_i64)
        }
    }

    pub fn get_as_f64(&self, key: &str) -> Option<f64> {
        self.j_obj.get(key).and_then(Value::as_f64)
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.j_obj.get(key)
    }
}
