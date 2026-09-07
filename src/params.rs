//! 参数构建器模块
//! 提供类型安全的参数传递方式，替代 HashMap<String, String>

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// 参数值的枚举类型，支持多种数据类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParamValue {
    /// UTF-8 string value.
    String(String),
    /// Signed integer value.
    Number(i64),
    /// Unsigned integer value, preserving the complete `u64` range.
    Unsigned(u64),
    /// Floating-point value; non-finite values are rejected when serialized.
    Float(f64),
    /// Boolean value.
    Boolean(bool),
    /// JSON null value.
    Null,
    /// Ordered array of parameter values.
    Array(Vec<ParamValue>),
    /// Object with string keys and nested parameter values.
    Object(HashMap<String, ParamValue>),
}

impl From<String> for ParamValue {
    fn from(v: String) -> Self {
        ParamValue::String(v)
    }
}

impl From<&str> for ParamValue {
    fn from(v: &str) -> Self {
        ParamValue::String(v.to_string())
    }
}

impl From<i32> for ParamValue {
    fn from(v: i32) -> Self {
        ParamValue::Number(v as i64)
    }
}

impl From<i64> for ParamValue {
    fn from(v: i64) -> Self {
        ParamValue::Number(v)
    }
}

impl From<u32> for ParamValue {
    fn from(v: u32) -> Self {
        ParamValue::Number(v as i64)
    }
}

impl From<u64> for ParamValue {
    fn from(v: u64) -> Self {
        ParamValue::Unsigned(v)
    }
}

impl From<f32> for ParamValue {
    fn from(v: f32) -> Self {
        ParamValue::Float(v as f64)
    }
}

impl From<f64> for ParamValue {
    fn from(v: f64) -> Self {
        ParamValue::Float(v)
    }
}

impl From<bool> for ParamValue {
    fn from(v: bool) -> Self {
        ParamValue::Boolean(v)
    }
}

impl<T> From<Vec<T>> for ParamValue
where
    T: Into<ParamValue>,
{
    fn from(v: Vec<T>) -> Self {
        ParamValue::Array(v.into_iter().map(Into::into).collect())
    }
}

impl From<Value> for ParamValue {
    fn from(v: Value) -> Self {
        match v {
            Value::String(s) => ParamValue::String(s),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    ParamValue::Number(i)
                } else if let Some(u) = n.as_u64() {
                    ParamValue::Unsigned(u)
                } else if let Some(f) = n.as_f64() {
                    ParamValue::Float(f)
                } else {
                    ParamValue::String(n.to_string())
                }
            }
            Value::Bool(b) => ParamValue::Boolean(b),
            Value::Array(arr) => ParamValue::Array(arr.into_iter().map(|v| v.into()).collect()),
            Value::Object(obj) => {
                ParamValue::Object(obj.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
            Value::Null => ParamValue::Null,
        }
    }
}

// 为 CallParams 实现 From 以转换为 HashMap<String, String>
impl From<CallParams> for HashMap<String, String> {
    fn from(val: CallParams) -> Self {
        val.to_hashmap()
    }
}

impl ParamValue {
    /// Convert to string representation (for backward compatibility with HashMap-based approach)
    ///
    /// Converts the param value to a string representation:
    /// - String values are returned as-is
    /// - Numeric values are converted to their string representation
    /// - Boolean values are converted to "true" or "false"
    /// - Null values are converted to "null"
    /// - Array and Object values are serialized as JSON strings
    pub fn to_string_lossy(&self) -> String {
        match self {
            ParamValue::String(s) => s.clone(),
            ParamValue::Number(n) => n.to_string(),
            ParamValue::Unsigned(n) => n.to_string(),
            ParamValue::Float(f) => f.to_string(),
            ParamValue::Boolean(b) => b.to_string(),
            ParamValue::Null => "null".to_string(),
            ParamValue::Array(arr) => serde_json::to_string(arr).unwrap_or_default(),
            ParamValue::Object(obj) => serde_json::to_string(obj).unwrap_or_default(),
        }
    }
}

/// 参数构建器，提供流式 API
#[derive(Debug, Clone, Default)]
pub struct CallParams {
    params: HashMap<String, ParamValue>,
}

/// Parameters separated by their HTTP transport location.
///
/// This is the preferred input for endpoints that combine parameter kinds,
/// such as `path,json`. It prevents a path identifier from being copied into
/// the JSON body and allows JSON bodies that are not objects.
#[derive(Debug, Clone, Default)]
pub struct RequestArgs {
    path: CallParams,
    query: CallParams,
    form: CallParams,
    query_pairs: Vec<(String, String)>,
    form_pairs: Vec<(String, String)>,
    json: Option<Value>,
}

impl RequestArgs {
    /// Create an empty request argument set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set URL path parameters.
    pub fn with_path(mut self, params: CallParams) -> Self {
        self.path = params;
        self
    }

    /// Set URL query parameters.
    pub fn with_query(mut self, params: CallParams) -> Self {
        self.query = params;
        self
    }

    /// Append query pairs. Duplicate keys are retained.
    pub fn with_query_pairs<K, V, I>(mut self, pairs: I) -> Self
    where
        K: Into<String>,
        V: Into<String>,
        I: IntoIterator<Item = (K, V)>,
    {
        self.query_pairs.extend(
            pairs
                .into_iter()
                .map(|(key, value)| (key.into(), value.into())),
        );
        self
    }

    /// Set form body parameters.
    pub fn with_form(mut self, params: CallParams) -> Self {
        self.form = params;
        self
    }

    /// Append form pairs. Duplicate keys are retained.
    pub fn with_form_pairs<K, V, I>(mut self, pairs: I) -> Self
    where
        K: Into<String>,
        V: Into<String>,
        I: IntoIterator<Item = (K, V)>,
    {
        self.form_pairs.extend(
            pairs
                .into_iter()
                .map(|(key, value)| (key.into(), value.into())),
        );
        self
    }

    /// Set an arbitrary JSON body.
    pub fn with_json(mut self, body: Value) -> Self {
        self.json = Some(body);
        self
    }

    /// Set an object-shaped JSON body from [`CallParams`].
    pub fn with_json_params(mut self, params: CallParams) -> Result<Self, crate::CallerError> {
        self.json = Some(params.to_json()?);
        Ok(self)
    }

    pub(crate) fn path_params(&self) -> &CallParams {
        &self.path
    }

    pub(crate) fn query_params(&self) -> &CallParams {
        &self.query
    }

    pub(crate) fn query_pairs(&self) -> &[(String, String)] {
        &self.query_pairs
    }

    pub(crate) fn form_params(&self) -> &CallParams {
        &self.form
    }

    pub(crate) fn form_pairs(&self) -> &[(String, String)] {
        &self.form_pairs
    }

    pub(crate) fn json_body(&self) -> Option<&Value> {
        self.json.as_ref()
    }
}

impl CallParams {
    /// Create a new empty CallParams builder
    ///
    /// # Returns
    /// A new CallParams instance with no parameters
    ///
    /// # Example
    /// ```
    /// use caller::CallParams;
    ///
    /// let params = CallParams::new();
    /// assert!(params.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a single parameter to the builder
    ///
    /// # Type Parameters
    /// * `K` - Key type that can be converted to String
    /// * `V` - Value type that can be converted to ParamValue
    ///
    /// # Arguments
    /// * `key` - Parameter key
    /// * `value` - Parameter value
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Example
    /// ```
    /// use caller::CallParams;
    ///
    /// let params = CallParams::new()
    ///     .add("name", "Alice")
    ///     .add("age", 30)
    ///     .add("active", true);
    /// ```
    pub fn add<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<ParamValue>,
    {
        self.params.insert(key.into(), value.into());
        self
    }

    /// Add multiple parameters from an iterable
    ///
    /// # Type Parameters
    /// * `K` - Key type that can be converted to String
    /// * `V` - Value type that can be converted to ParamValue
    ///
    /// # Arguments
    /// * `iter` - An iterator of (key, value) pairs
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Example
    /// ```
    /// use caller::CallParams;
    ///
    /// let params = CallParams::new()
    ///     .add_many(vec![
    ///         ("name", "Alice"),
    ///         ("age", "30"),
    ///         ("active", "true"),
    ///     ]);
    /// ```
    pub fn add_many<K, V>(mut self, iter: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<String>,
        V: Into<ParamValue>,
    {
        for (key, value) in iter {
            self.params.insert(key.into(), value.into());
        }
        self
    }

    /// Create CallParams from a HashMap<String, String>
    ///
    /// This is useful for converting legacy HashMap-based parameters to CallParams
    ///
    /// # Arguments
    /// * `hashmap` - HashMap with string keys and values
    ///
    /// # Returns
    /// A new CallParams instance
    ///
    /// # Example
    /// ```
    /// use caller::CallParams;
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("name".to_string(), "Alice".to_string());
    /// map.insert("age".to_string(), "30".to_string());
    ///
    /// let params = CallParams::from_hashmap(map);
    /// ```
    pub fn from_hashmap(hashmap: HashMap<String, String>) -> Self {
        let params = hashmap
            .into_iter()
            .map(|(k, v)| (k, ParamValue::String(v)))
            .collect();
        Self { params }
    }

    /// Create CallParams from a JSON Value
    ///
    /// # Arguments
    /// * `value` - A JSON object value (serde_json::Value)
    ///
    /// # Returns
    /// Ok(CallParams) if the value is an object, Err otherwise
    ///
    /// # Example
    /// ```
    /// use caller::CallParams;
    /// use serde_json::json;
    ///
    /// let json_value = json!({
    ///     "name": "Alice",
    ///     "age": 30,
    ///     "active": true
    /// });
    ///
    /// let params = CallParams::from_json(json_value).unwrap();
    /// ```
    pub fn from_json(value: Value) -> Result<Self, crate::CallerError> {
        match value {
            Value::Object(obj) => {
                let params = obj.into_iter().map(|(k, v)| (k, v.into())).collect();
                Ok(Self { params })
            }
            _ => Err(crate::CallerError::parameter_error(
                "JSON parameters must be an object",
            )),
        }
    }

    /// Convert to HashMap<String, String> for backward compatibility
    ///
    /// This method converts all ParamValue instances to their string representation
    ///
    /// # Returns
    /// A HashMap with string keys and values
    pub fn to_hashmap(&self) -> HashMap<String, String> {
        self.params
            .iter()
            .map(|(k, v)| (k.clone(), v.to_string_lossy()))
            .collect()
    }

    /// Convert scalar values to strings without silently flattening structured values.
    ///
    /// Use this for path, query, and form parameters. Arrays, objects, nulls,
    /// and non-finite floats return an error.
    pub fn to_scalar_hashmap(&self) -> Result<HashMap<String, String>, crate::CallerError> {
        self.params
            .iter()
            .map(|(key, value)| Ok((key.clone(), value.try_to_scalar_string(key)?)))
            .collect()
    }

    /// Convert to a JSON value.
    ///
    /// # Returns
    /// Returns an error for `NaN` or infinite floating-point values because
    /// JSON cannot represent them.
    pub fn to_json(&self) -> Result<Value, crate::CallerError> {
        let mut map = serde_json::Map::new();
        for (key, value) in &self.params {
            map.insert(key.clone(), value.try_to_json(key)?);
        }
        Ok(Value::Object(map))
    }

    /// Check if the params collection is empty
    ///
    /// # Returns
    /// true if no parameters are stored, false otherwise
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// Get the number of parameters stored
    ///
    /// # Returns
    /// The count of parameters
    pub fn len(&self) -> usize {
        self.params.len()
    }

    /// Get a parameter value by key
    ///
    /// # Arguments
    /// * `key` - The parameter key to look up
    ///
    /// # Returns
    /// Some(&ParamValue) if the key exists, None otherwise
    pub fn get(&self, key: &str) -> Option<&ParamValue> {
        self.params.get(key)
    }
}

impl ParamValue {
    fn try_to_scalar_string(&self, key: &str) -> Result<String, crate::CallerError> {
        match self {
            Self::String(value) => Ok(value.clone()),
            Self::Number(value) => Ok(value.to_string()),
            Self::Unsigned(value) => Ok(value.to_string()),
            Self::Float(value) if value.is_finite() => Ok(value.to_string()),
            Self::Boolean(value) => Ok(value.to_string()),
            Self::Float(_) | Self::Null | Self::Array(_) | Self::Object(_) => {
                Err(crate::CallerError::parameter_error(format!(
                    "parameter '{key}' must be a finite scalar value"
                )))
            }
        }
    }

    fn try_to_json(&self, key: &str) -> Result<Value, crate::CallerError> {
        match self {
            Self::String(value) => Ok(Value::String(value.clone())),
            Self::Number(value) => Ok(Value::Number((*value).into())),
            Self::Unsigned(value) => Ok(Value::Number((*value).into())),
            Self::Float(value) => serde_json::Number::from_f64(*value)
                .map(Value::Number)
                .ok_or_else(|| {
                    crate::CallerError::parameter_error(format!(
                        "parameter '{key}' contains a non-finite floating-point value"
                    ))
                }),
            Self::Boolean(value) => Ok(Value::Bool(*value)),
            Self::Null => Ok(Value::Null),
            Self::Array(values) => values
                .iter()
                .enumerate()
                .map(|(index, value)| value.try_to_json(&format!("{key}[{index}]")))
                .collect::<Result<Vec<_>, _>>()
                .map(Value::Array),
            Self::Object(values) => {
                let mut map = serde_json::Map::new();
                for (nested_key, value) in values {
                    let full_key = format!("{key}.{nested_key}");
                    map.insert(nested_key.clone(), value.try_to_json(&full_key)?);
                }
                Ok(Value::Object(map))
            }
        }
    }
}

/// 方便创建参数的宏
///
/// # Examples
/// ```
/// use caller::params;
///
/// // 基本用法 - 整数
/// let p = params! {
///     "post_id" => 1
/// };
///
/// // 支持数组
/// let p = params! {
///     "tags" => vec!["rust", "web"]
/// };
///
/// // 基本用法 - 字符串
/// let p = params! {
///     "title" => "Hello"
/// };
///
/// // 基本用法 - 布尔值
/// let p = params! {
///     "active" => true
/// };
/// ```
#[macro_export]
macro_rules! params {
    // 空参数
    () => {
        $crate::CallParams::new()
    };
    // 键值对逐项插入，因此不同参数可以使用不同的 Rust 类型。
    ($($key:expr => $value:expr),+ $(,)?) => {
        {
            let params = $crate::CallParams::new();
            $(let params = params.add($key, $value);)+
            params
        }
    };
}

/// 从 serde_json::Value 创建参数的快捷方式
///
/// # Examples
/// ```
/// use caller::json_params;
/// use serde_json::json;
///
/// let params = json_params!(json!({
///     "title": "Hello",
///     "count": 10,
///     "active": true,
///     "tags": ["rust", "web"],
///     "user": {
///         "name": "Alice",
///         "age": 30
///     }
/// })).unwrap();
/// ```
#[macro_export]
macro_rules! json_params {
    ($value:expr) => {
        $crate::CallParams::from_json($value)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_unsigned_integer_range() {
        let params = CallParams::new().add("value", u64::MAX);
        assert_eq!(
            params.to_json().unwrap()["value"],
            serde_json::json!(u64::MAX)
        );
    }

    #[test]
    fn rejects_non_finite_json_numbers() {
        let error = CallParams::new()
            .add("value", f64::NAN)
            .to_json()
            .unwrap_err();
        assert!(matches!(error, crate::CallerError::ParameterError(_)));
    }

    #[test]
    fn request_args_accept_arbitrary_json_body() {
        let args = RequestArgs::new().with_json(serde_json::json!([1, 2, 3]));
        assert_eq!(args.json_body(), Some(&serde_json::json!([1, 2, 3])));
    }
}
