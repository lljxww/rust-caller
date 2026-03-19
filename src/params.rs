//! 参数构建器模块
//! 提供类型安全的参数传递方式，替代 HashMap<String, String>

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// 参数值的枚举类型，支持多种数据类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParamValue {
    String(String),
    Number(i64),
    Float(f64),
    Boolean(bool),
    Null,
    Array(Vec<ParamValue>),
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
        ParamValue::Number(v as i64)
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
                } else if let Some(f) = n.as_f64() {
                    ParamValue::Float(f)
                } else {
                    ParamValue::String(n.to_string())
                }
            }
            Value::Bool(b) => ParamValue::Boolean(b),
            Value::Array(arr) => ParamValue::Array(
                arr.into_iter()
                    .map(|v| v.into())
                    .collect(),
            ),
            Value::Object(obj) => ParamValue::Object(
                obj.into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect(),
            ),
            Value::Null => ParamValue::Null,
        }
    }
}

// 为 CallParams 实现 Into<HashMap<String, String>>
impl Into<HashMap<String, String>> for CallParams {
    fn into(self) -> HashMap<String, String> {
        self.to_hashmap()
    }
}

// 为 ParamValue 实现 Into<Value> 以支持 to_json() 中的转换
impl From<ParamValue> for Value {
    fn from(v: ParamValue) -> Self {
        match v {
            ParamValue::String(s) => Value::String(s),
            ParamValue::Number(n) => Value::Number(n.into()),
            ParamValue::Float(f) => Value::Number(
                serde_json::Number::from_f64(f)
                    .unwrap_or_else(|| serde_json::Number::from(0))
            ),
            ParamValue::Boolean(b) => Value::Bool(b),
            ParamValue::Null => Value::Null,
            ParamValue::Array(arr) => Value::Array(arr.into_iter().map(Into::into).collect()),
            ParamValue::Object(obj) => Value::Object(
                obj.into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect()
            ),
        }
    }
}

impl ParamValue {
    /// 转换为字符串（用于兼容旧的 HashMap 方式）
    pub fn to_string_lossy(&self) -> String {
        match self {
            ParamValue::String(s) => s.clone(),
            ParamValue::Number(n) => n.to_string(),
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

impl CallParams {
    /// 创建新的参数构建器
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加一个参数
    pub fn add<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<ParamValue>,
    {
        self.params.insert(key.into(), value.into());
        self
    }

    /// 添加多个参数
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

    /// 从 HashMap<String, String> 转换
    pub fn from_hashmap(hashmap: HashMap<String, String>) -> Self {
        let params = hashmap
            .into_iter()
            .map(|(k, v)| (k, ParamValue::String(v)))
            .collect();
        Self { params }
    }

    /// 从 JSON 值转换
    pub fn from_json(value: Value) -> Result<Self, String> {
        match value {
            Value::Object(obj) => {
                let params = obj
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect();
                Ok(Self { params })
            }
            _ => Err("JSON value must be an object".to_string()),
        }
    }

    /// 转换为 HashMap<String, String>（用于兼容旧代码）
    pub fn to_hashmap(&self) -> HashMap<String, String> {
        self.params
            .iter()
            .map(|(k, v)| (k.clone(), v.to_string_lossy()))
            .collect()
    }

    /// 转换为 JSON
    pub fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        for (key, value) in &self.params {
            map.insert(
                key.clone(),
                value.clone().into(),
            );
        }
        Value::Object(map)
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// 获取参数数量
    pub fn len(&self) -> usize {
        self.params.len()
    }

    /// 获取参数值
    pub fn get(&self, key: &str) -> Option<&ParamValue> {
        self.params.get(key)
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
        $crate::params::CallParams::new()
    };
    // 单个键值对
    ($key:expr => $value:expr) => {
        $crate::params::CallParams::new().add($key, $value)
    };
    // 多个键值对
    ($($key:expr => $value:expr),+ $(,)?) => {
        $crate::params::CallParams::new().add_many([
            $(($key, $value)),+
        ])
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
/// }));
/// ```
#[macro_export]
macro_rules! json_params {
    ($value:expr) => {
        $crate::params::CallParams::from_json($value).expect("Invalid JSON object")
    };
}
