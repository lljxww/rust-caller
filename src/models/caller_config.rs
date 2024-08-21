use std::fmt::{Display, Formatter, Result};

use serde::Deserialize;

use super::{authorization::Authorization, service_item::ServiceItem};

#[derive(Debug, Deserialize, Clone)]
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
