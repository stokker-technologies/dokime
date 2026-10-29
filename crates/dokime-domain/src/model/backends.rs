/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub identifier: String,
    pub label: String,
    pub endpoint: String,
    pub expected_backend_id: Option<String>,
    pub ca_certificate_pem: Option<String>,
    pub secret: Secret,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub version: u8,
    pub default: Option<String>,
    pub profiles: Vec<Profile>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Secret {
    None,
    Keyring,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            version: 1,
            default: None,
            profiles: Vec::new(),
        }
    }
}

impl Profile {
    pub fn validate(&self) -> Result<(), &'static str> {
        crate::model::secrets::catalog::validate_identifier(&self.identifier)?;
        crate::model::secrets::catalog::validate_label(&self.label)?;
        Ok(())
    }
}
