use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ApiItem {
    #[serde(rename = "Method")]
    pub method: String,

    #[serde(rename = "Url")]
    pub url: String,

    #[serde(rename = "HttpMethod")]
    pub http_method: String,

    #[serde(rename = "ParamType")]
    pub param_type: String,

    #[serde(rename = "Description")]
    pub description: Option<String>,

    #[serde(rename = "NeedCache")]
    pub need_cache: Option<bool>,

    #[serde(rename = "CacheTime")]
    pub cache_time: Option<u32>,

    #[serde(rename = "ContentType")]
    pub content_type: Option<String>,

    #[serde(rename = "AuthorizationType")]
    pub authorization_type: Option<String>,

    #[serde(rename = "Timeout")]
    pub timeout: Option<u32>,

    #[serde(rename = "UseNewHttpClient")]
    pub use_new_http_client: Option<bool>,
}
