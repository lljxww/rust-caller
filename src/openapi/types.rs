//! OpenAPI 3.0 type definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenAPI 3.0 Document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiDoc {
    pub openapi: String,
    pub info: Info,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<Server>,
    pub paths: HashMap<String, PathItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
}

/// API Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub title: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,
}

/// Contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// License information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Server definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, ServerVariable>>,
}

/// Server variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerVariable {
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    pub default: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Path item (operations for a path)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Parameter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,
}

/// Operation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Parameter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,
    pub responses: HashMap<String, Response>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub security: Vec<HashMap<String, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,
}

/// Parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: String, // query, header, path, cookie
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Schema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
}

/// Request body definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub content: HashMap<String, MediaType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// Media type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Schema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<HashMap<String, Example>>,
}

/// Response definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, Header>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<HashMap<String, MediaType>>,
}

/// Header definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Schema>,
}

/// Schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub ref_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<serde_json::Value>>,
}

impl Schema {
    pub fn new(schema_type: &str) -> Self {
        Self {
            schema_type: Some(schema_type.to_string()),
            format: None,
            description: None,
            properties: None,
            items: None,
            required: None,
            ref_path: None,
            example: None,
            default: None,
            enum_values: None,
        }
    }

    pub fn object() -> Self {
        Self::new("object")
    }

    pub fn string() -> Self {
        Self::new("string")
    }

    pub fn integer() -> Self {
        Self::new("integer")
    }

    pub fn number() -> Self {
        Self::new("number")
    }

    pub fn boolean() -> Self {
        Self::new("boolean")
    }

    pub fn array() -> Self {
        Self::new("array")
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn format(mut self, format: &str) -> Self {
        self.format = Some(format.to_string());
        self
    }

    pub fn example(mut self, example: serde_json::Value) -> Self {
        self.example = Some(example);
        self
    }
}

/// Example definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_value: Option<String>,
}

/// Components definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<HashMap<String, Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_schemes: Option<HashMap<String, SecurityScheme>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, Parameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_bodies: Option<HashMap<String, RequestBody>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responses: Option<HashMap<String, Response>>,
}

/// Security scheme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScheme {
    #[serde(rename = "type")]
    pub scheme_type: String, // apiKey, http, oauth2, openIdConnect
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "in")]
    pub location: Option<String>, // query, header
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flows: Option<OAuthFlows>,
}

impl SecurityScheme {
    pub fn api_key(name: &str, location: &str) -> Self {
        Self {
            scheme_type: "apiKey".to_string(),
            description: None,
            name: Some(name.to_string()),
            location: Some(location.to_string()),
            scheme: None,
            bearer_format: None,
            flows: None,
        }
    }

    pub fn bearer() -> Self {
        Self {
            scheme_type: "http".to_string(),
            description: None,
            name: None,
            location: None,
            scheme: Some("bearer".to_string()),
            bearer_format: Some("JWT".to_string()),
            flows: None,
        }
    }

    pub fn basic() -> Self {
        Self {
            scheme_type: "http".to_string(),
            description: None,
            name: None,
            location: None,
            scheme: Some("basic".to_string()),
            bearer_format: None,
            flows: None,
        }
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
}

/// OAuth flows definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthFlows {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implicit: Option<OAuthFlow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<OAuthFlow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_credentials: Option<OAuthFlow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_code: Option<OAuthFlow>,
}

/// OAuth flow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthFlow {
    pub authorization_url: String,
    pub token_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,
    pub scopes: HashMap<String, String>,
}

/// Tag definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocs>,
}

/// External documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDocs {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
