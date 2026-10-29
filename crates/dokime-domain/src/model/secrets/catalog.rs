/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports

/* === Constants === */

pub const CATALOG_ITEMS: u64 = 1_024;
pub const SECRET_BYTES: u64 = 65_536;
pub const DOCUMENT_BYTES: u64 = 524_288;
pub const IDENTIFIER_BYTES: u64 = 128;
pub const LABEL_BYTES: u64 = 256;
pub const DESCRIPTION_BYTES: u64 = 2_048;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos_domain::message]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub identifier: String,
    pub label: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Slot {
    pub metadata: Metadata,
    pub slot: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub generation: u64,
    pub entries: BTreeMap<String, Slot>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    pub metadata: Metadata,
    pub known_uses: Vec<String>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub host_id: String,
    pub host_label: String,
    pub generation: u64,
    pub entries: Vec<Summary>,
    pub cleanup_pending: bool,
}

#[harmos_domain::message(skip_debug)]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Value {
    pub value: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Metadata {
    pub fn check_limits(&self) -> Result<(), &'static str> {
        check_identifier_limit(&self.identifier)?;
        if self.label.len() as u64 > LABEL_BYTES {
            return Err("Secret label exceeds the configured byte limit");
        }
        if self.description.as_ref().is_some_and(|text| text.len() as u64 > DESCRIPTION_BYTES) {
            return Err("Secret description exceeds the configured byte limit");
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        validate_identifier(&self.identifier)?;
        validate_label(&self.label)?;
        if self.description.as_ref().is_some_and(|text| text.contains('\0')) {
            return Err("Secret descriptions must not contain NUL");
        }
        Ok(())
    }
}

impl Catalog {
    pub fn check_limits(&self) -> Result<(), &'static str> {
        if self.entries.len() as u64 > CATALOG_ITEMS {
            return Err("The host secret catalog exceeds its configured entry limit");
        }
        for entry in self.entries.values() {
            entry.metadata.check_limits()?;
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        for (identifier, entry) in &self.entries {
            entry.metadata.validate()?;
            if identifier != &entry.metadata.identifier || crate::model::identity::id::validate_uuid("SecretSlot", &entry.slot).is_err() {
                return Err("The host secret catalog has an invalid slot or identifier");
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SecretValue(<redacted>)")
    }
}

/* === Public Functions === */

pub fn check_identifier_limit(identifier: &str) -> Result<(), &'static str> {
    if identifier.len() as u64 > IDENTIFIER_BYTES {
        return Err("Secret identifier exceeds the configured byte limit");
    }
    Ok(())
}

pub fn check_value_limit(value: &str) -> Result<(), &'static str> {
    if value.len() as u64 > SECRET_BYTES {
        return Err("Secret value exceeds the configured byte limit");
    }
    Ok(())
}

pub fn validate_identifier(identifier: &str) -> Result<(), &'static str> {
    check_identifier_limit(identifier)?;
    let characters = identifier.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
    if identifier.is_empty() || !characters || matches!(identifier, "." | "..") {
        return Err("Secret identifiers must contain ASCII letters, digits, dots, underscores or hyphens");
    }
    Ok(())
}

pub fn validate_value(value: &str) -> Result<(), &'static str> {
    if value.is_empty() {
        return Err("Secret values must not be empty");
    }
    Ok(())
}

pub fn validate_label(label: &str) -> Result<(), &'static str> {
    if label.trim().is_empty() || label.chars().any(char::is_control) || label.len() as u64 > LABEL_BYTES {
        return Err("Labels must be nonblank, control-free and at most 256 bytes");
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_and_labels_are_bounded() {
        for identifier in ["", ".", "..", "a/b", "two words", "é"] {
            assert!(validate_identifier(identifier).is_err());
        }
        assert!(validate_identifier(&"a".repeat(128)).is_ok());
        assert!(validate_identifier(&"a".repeat(129)).is_err());
        assert!(validate_identifier("Lab_01.meter-a").is_ok());
        assert!(validate_label(&"é".repeat(128)).is_ok());
        assert!(validate_label(&"é".repeat(129)).is_err());
        assert!(validate_label(" \t ").is_err());
        assert!(validate_label("Label\nNext").is_err());
    }

    #[test]
    fn metadata_rejects_removed_fields() {
        let current = r#"{"identifier":"meter","label":"Meter","description":null}"#;
        assert!(serde_json::from_str::<Metadata>(current).is_ok());
        let removed = r#"{"name":"meter","identifier":"meter","label":"Meter","description":null}"#;
        assert!(serde_json::from_str::<Metadata>(removed).is_err());
    }
}
