use super::{auth_config::AuthConfig, service_config::ServiceConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use crate::shared::error::CallerError;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct CallerConfig {
    pub authorizations: Vec<AuthConfig>,
    pub service_items: Vec<ServiceConfig>,
}

impl CallerConfig {
    pub fn validate(&self) -> Result<(), CallerError> {
        let mut service_names = HashSet::new();

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
