use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct AuthConfig {
    pub name: String,
    pub authorization_info: Option<String>,
}
