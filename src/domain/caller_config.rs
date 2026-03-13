use super::{authorization::Authorization, service_item::ServiceItem};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CallerConfig {
    #[serde(rename = "Authorizations")]
    pub authorizations: Vec<Authorization>,

    #[serde(rename = "ServiceItems")]
    pub service_items: Vec<ServiceItem>,
}

impl Display for CallerConfig {
    fn fmt(&self, f: &mut Formatter) -> Result {
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
