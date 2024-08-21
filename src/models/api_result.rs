use reqwest::StatusCode;
use serde_json::Value;

#[derive(Debug, Clone)]

pub struct ApiResult {
    pub status_code: StatusCode,
    pub raw: String,
    pub j_obj: Value,
}

impl ApiResult {
    pub fn build(raw: String, status_code: StatusCode) -> ApiResult {
        let j_obj: Value = serde_json::from_str(&raw).unwrap();

        ApiResult {
            status_code,
            raw,
            j_obj,
        }
    }

    pub fn get<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        match self
            .j_obj
            .get(key)
            .and_then(|v| v.as_str().map(|s| s.parse::<T>().ok()))
        {
            Some(value) => value,
            None => None,
        }
    }
}
