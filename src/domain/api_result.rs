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
    /// Build an ApiResult from raw response string and status code
    /// 
    /// This method parses the raw JSON response and creates an ApiResult instance
    /// that allows convenient access to response data.
    /// 
    /// # Arguments
    /// * `raw` - Raw response body as string
    /// * `status_code` - HTTP status code of the response
    /// 
    /// # Returns
    /// Ok(ApiResult) if JSON parsing succeeds, Err otherwise
    /// 
    /// # Errors
    /// Returns CallerError::JsonError if the response is not valid JSON
    pub fn build(raw: String, status_code: StatusCode) -> Result<ApiResult, CallerError> {
        let j_obj: Value = serde_json::from_str(&raw)
            .map_err(|e| CallerError::JsonError(format!("Failed to parse JSON response: {}", e)))?;

        Ok(ApiResult {
            status_code,
            raw,
            j_obj,
        })
    }

    /// Get a nested value from the JSON response using dot notation
    /// 
    /// Supports accessing nested objects and arrays:
    /// - `"user.name"` accesses nested object
    /// - `"items.0"` accesses array elements
    /// - `"users.0.address.city"` accesses nested arrays and objects
    /// 
    /// # Arguments
    /// * `keys` - Vector of key segments to navigate
    /// 
    /// # Returns
    /// Some(&Value) if the path exists, None otherwise
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

    /// Get a string value from the JSON response
    /// 
    /// Supports dot notation for nested access (e.g., "user.name", "items.0.title")
    /// 
    /// # Arguments
    /// * `key` - Key path to access (supports dot notation)
    /// 
    /// # Returns
    /// Some(&str) if the value exists and is a string, None otherwise
    /// 
    /// # Example
    /// ```
    /// # use caller::ApiResult;
    /// # use reqwest::StatusCode;
    /// # let result = ApiResult::build(r#"{"name": "Alice", "user": {"email": "alice@example.com"}}"#.to_string(), StatusCode::OK).unwrap();
    /// assert_eq!(result.get_as_str("name"), Some("Alice"));
    /// assert_eq!(result.get_as_str("user.email"), Some("alice@example.com"));
    /// ```
    pub fn get_as_str(&self, key: &str) -> Option<&str> {
        if key.contains(SPLIT_OPERATOR) {
            let keys: Vec<&str> = key.split(SPLIT_OPERATOR).collect();
            self.get_deep(keys).and_then(|v| v.as_str())
        } else {
            self.j_obj.get(key).and_then(Value::as_str)
        }
    }

    /// Get a boolean value from the JSON response
    /// 
    /// Supports dot notation for nested access
    /// 
    /// # Arguments
    /// * `key` - Key path to access (supports dot notation)
    /// 
    /// # Returns
    /// Some(bool) if the value exists and is a boolean, None otherwise
    /// 
    /// # Example
    /// ```
    /// # use caller::ApiResult;
    /// # use reqwest::StatusCode;
    /// # let result = ApiResult::build(r#"{"active": true, "settings": {"enabled": false}}"#.to_string(), StatusCode::OK).unwrap();
    /// assert_eq!(result.get_as_bool("active"), Some(true));
    /// assert_eq!(result.get_as_bool("settings.enabled"), Some(false));
    /// ```
    pub fn get_as_bool(&self, key: &str) -> Option<bool> {
        if key.contains(SPLIT_OPERATOR) {
            let keys: Vec<&str> = key.split(SPLIT_OPERATOR).collect();
            self.get_deep(keys).and_then(|v| v.as_bool())
        } else {
            self.j_obj.get(key).and_then(Value::as_bool)
        }
    }

    /// Get an i64 integer value from the JSON response
    /// 
    /// Supports dot notation for nested access
    /// 
    /// # Arguments
    /// * `key` - Key path to access (supports dot notation)
    /// 
    /// # Returns
    /// Some(i64) if the value exists and is an integer, None otherwise
    /// 
    /// # Example
    /// ```
    /// # use caller::ApiResult;
    /// # use reqwest::StatusCode;
    /// # let result = ApiResult::build(r#"{"count": 42, "page": {"size": 10}}"#.to_string(), StatusCode::OK).unwrap();
    /// assert_eq!(result.get_as_i64("count"), Some(42));
    /// assert_eq!(result.get_as_i64("page.size"), Some(10));
    /// ```
    pub fn get_as_i64(&self, key: &str) -> Option<i64> {
        if key.contains(SPLIT_OPERATOR) {
            let keys: Vec<&str> = key.split(SPLIT_OPERATOR).collect();
            self.get_deep(keys).and_then(|v| v.as_i64())
        } else {
            self.j_obj.get(key).and_then(Value::as_i64)
        }
    }

    /// Get an f64 floating-point value from the JSON response
    /// 
    /// Supports dot notation for nested access
    /// 
    /// # Arguments
    /// * `key` - Key path to access (supports dot notation)
    /// 
    /// # Returns
    /// Some(f64) if the value exists and is a number, None otherwise
    /// 
    /// # Example
    /// ```
    /// # use caller::ApiResult;
    /// # use reqwest::StatusCode;
    /// # let result = ApiResult::build(r#"{"price": 19.99, "tax": {"rate": 0.08}}"#.to_string(), StatusCode::OK).unwrap();
    /// assert_eq!(result.get_as_f64("price"), Some(19.99));
    /// assert_eq!(result.get_as_f64("tax.rate"), Some(0.08));
    /// ```
    pub fn get_as_f64(&self, key: &str) -> Option<f64> {
        if key.contains(SPLIT_OPERATOR) {
            let keys: Vec<&str> = key.split(SPLIT_OPERATOR).collect();
            self.get_deep(keys).and_then(|v| v.as_f64())
        } else {
            self.j_obj.get(key).and_then(Value::as_f64)
        }
    }

    /// Get a raw JSON value from the response
    /// 
    /// This is useful when you need to access complex nested structures
    /// or when the type is not known in advance.
    /// 
    /// # Arguments
    /// * `key` - Key path to access (supports dot notation)
    /// 
    /// # Returns
    /// Some(&Value) if the key exists, None otherwise
    /// 
    /// # Example
    /// ```
    /// # use caller::ApiResult;
    /// # use reqwest::StatusCode;
    /// # use serde_json::json;
    /// # let result = ApiResult::build(r#"{"user": {"name": "Alice", "age": 30}}"#.to_string(), StatusCode::OK).unwrap();
    /// let user = result.get("user");
    /// assert!(user.is_some());
    /// ```
    pub fn get(&self, key: &str) -> Option<&Value> {
        let keys: Vec<&str> = key.split(SPLIT_OPERATOR).collect();
        self.get_deep(keys)
    }
}
