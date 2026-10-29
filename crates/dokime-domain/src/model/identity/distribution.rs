/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Install-Time Distribution Identity

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DistributionIdentity {
    pub schema: u32,
    pub identifier: String,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("distribution identity requires schema 1 and a lowercase reverse-DNS identifier")]
pub struct DistributionIdentityError;

impl DistributionIdentity {
    pub fn new(identifier: impl Into<String>) -> Result<Self, DistributionIdentityError> {
        let identity = Self { schema: 1, identifier: identifier.into() };
        identity.validate()?;
        Ok(identity)
    }

    pub fn validate(&self) -> Result<(), DistributionIdentityError> {
        let identifier = &self.identifier;
        let valid = identifier.len() <= 253
            && identifier.contains('.')
            && identifier.split('.').all(|label| {
                let bytes = label.as_bytes();
                let start = bytes.first().is_some_and(u8::is_ascii_alphanumeric);
                let end = bytes.last().is_some_and(u8::is_ascii_alphanumeric);
                let characters = bytes.iter().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-');
                start && end && label.len() <= 63 && characters
            });
        if self.schema != 1 || !valid {
            return Err(DistributionIdentityError);
        }
        Ok(())
    }

    pub fn slug(&self) -> String {
        self.identifier.replace('.', "-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_identity() {
        let identity = DistributionIdentity::new("com.hilti.dokime").unwrap();
        assert_eq!(identity.slug(), "com-hilti-dokime");
        assert!(DistributionIdentity::new("com.3m.dokime").is_ok());
        for value in ["", "dokime", "../escape", "com..dokime", "com.-bad.app", "com.bad-.app", "Com.example.app", "com.example_app"] {
            assert!(DistributionIdentity::new(value).is_err(), "{value}");
        }
        let mut identity = identity;
        identity.schema = 2;
        assert!(identity.validate().is_err());
    }
}
