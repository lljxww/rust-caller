use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Authorization {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "AuthorizationInfo")]
    pub authorization_info: Option<String>,
}
