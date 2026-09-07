use super::{auth_config::AuthConfig, service_config::ServiceConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use crate::shared::error::CallerError;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
/// Root configuration containing all configured upstream services.
pub struct CallerConfig {
    /// Legacy metadata accepted while reading old configuration files.
    ///
    /// It is never applied to requests and is deliberately omitted when
    /// serializing to avoid copying credentials during format conversion.
    #[serde(
        default,
        skip_serializing,
        deserialize_with = "deserialize_and_discard_authorizations"
    )]
    pub authorizations: Vec<AuthConfig>,
    /// Services addressable through `service.method` lookup keys.
    #[serde(default)]
    pub service_items: Vec<ServiceConfig>,
}

fn deserialize_and_discard_authorizations<'de, D>(
    deserializer: D,
) -> Result<Vec<AuthConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let _legacy_values = Vec::<AuthConfig>::deserialize(deserializer)?;
    Ok(Vec::new())
}

impl CallerConfig {
    /// Validate names, duplicates, URLs, timeouts, and every nested endpoint.
    pub fn validate(&self) -> Result<(), CallerError> {
        let mut service_names = HashSet::new();
        let mut auth_names = HashSet::new();

        for authorization in &self.authorizations {
            if authorization.name.trim().is_empty()
                || authorization.name.trim() != authorization.name
            {
                return Err(CallerError::config_error(
                    "Authorization name must be non-empty and cannot have surrounding whitespace",
                ));
            }
            if !auth_names.insert(authorization.name.as_str()) {
                return Err(CallerError::config_error(format!(
                    "Duplicate authorization name '{}'",
                    authorization.name
                )));
            }
        }

        for service in &self.service_items {
            if !service_names.insert(service.api_name.as_str()) {
                return Err(CallerError::config_error(format!(
                    "Duplicate service name '{}'",
                    service.api_name
                )));
            }
            service.validate()?;
        }

        Ok(())
    }
}

impl Display for CallerConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r"CallerConfig {{
                authorizations: {:?},
                service_items: {:?}
            }}",
            self.authorizations, self.service_items
        )
    }
}
