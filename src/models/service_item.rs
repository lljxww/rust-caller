use std::collections::HashMap;

use serde::Deserialize;

use super::api_item::ApiItem;

#[derive(Debug, Deserialize)]
pub struct ServiceItem {
    #[serde(rename = "ApiName")]
    pub api_name: String,

    #[serde(rename = "AuthorizationType")]
    pub authorization_type: Option<String>,

    #[serde(rename = "BaseUrl")]
    pub base_url: String,

    #[serde(rename = "Timeout")]
    pub timeout: Option<u32>,

    #[serde(rename = "ApiItems")]
    pub api_items: Vec<ApiItem>,

    #[serde(rename = "UseNewHttpClient")]
    pub use_new_http_client: Option<bool>,
}
