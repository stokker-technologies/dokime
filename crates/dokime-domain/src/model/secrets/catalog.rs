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
pub const NAME_BYTES: u64 = 128;
pub const LABEL_BYTES: u64 = 256;
pub const DESCRIPTION_BYTES: u64 = 2_048;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos_domain::message]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub name: String,
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
        check_name_limit(&self.name)?;
        if self.label.len() as u64 > LABEL_BYTES {
            return Err("Secret label exceeds the configured byte limit");
        }
        if self.description.as_ref().is_some_and(|text| text.len() as u64 > DESCRIPTION_BYTES) {
            return Err("Secret description exceeds the configured byte limit");
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        validate_name(&self.name)?;
        if self.label.trim().is_empty() || self.label.chars().any(char::is_control) {
            return Err("Secret labels must be nonempty without control characters");
        }
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
        for (name, entry) in &self.entries {
            entry.metadata.validate()?;
            if name != &entry.metadata.name || crate::model::identity::id::validate_uuid("SecretSlot", &entry.slot).is_err() {
                return Err("The host secret catalog has an invalid slot or name");
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

pub fn check_name_limit(name: &str) -> Result<(), &'static str> {
    if name.len() as u64 > NAME_BYTES {
        return Err("Secret name exceeds the configured byte limit");
    }
    Ok(())
}

pub fn check_value_limit(value: &str) -> Result<(), &'static str> {
    if value.len() as u64 > SECRET_BYTES {
        return Err("Secret value exceeds the configured byte limit");
    }
    Ok(())
}

pub fn validate_name(name: &str) -> Result<(), &'static str> {
    let characters = name.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
    if name.is_empty() || !characters || matches!(name, "." | "..") {
        return Err("Secret names must contain ASCII letters, digits, dots, underscores or hyphens");
    }
    Ok(())
}

pub fn validate_value(value: &str) -> Result<(), &'static str> {
    if value.is_empty() {
        return Err("Secret values must not be empty");
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
