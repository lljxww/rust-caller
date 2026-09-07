use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
/// Legacy serialized authentication metadata.
///
/// Root configuration deserialization accepts this shape for compatibility but
/// discards it immediately. Register runtime credentials on [`crate::Caller`]
/// instead.
pub struct AuthConfig {
    /// Legacy provider name.
    pub name: String,
    /// Legacy credential value; never emitted by `Debug`.
    pub authorization_info: Option<String>,
}

impl fmt::Debug for AuthConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthConfig")
            .field("name", &self.name)
            .field(
                "authorization_info",
                &self.authorization_info.as_ref().map(|_| "[REDACTED]"),
            )
            .finish()
    }
}
