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

pub const MAX_SECRETS: usize = 1024;
pub const MAX_SECRET_BYTES: usize = 64 * 1024;
pub const MAX_EXPORT_BYTES: usize = 8 * 1024 * 1024;

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
    pub fn validate(&self) -> Result<(), &'static str> {
        validate_name(&self.name)?;
        if self.label.trim().is_empty() || self.label.len() > 256 || self.label.chars().any(char::is_control) {
            return Err("Secret labels must contain 1–256 bytes without control characters");
        }
        if self.description.as_ref().is_some_and(|text| text.len() > 2048 || text.contains('\0')) {
            return Err("Secret descriptions must contain at most 2048 bytes without NUL");
        }
        Ok(())
    }
}

impl Catalog {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.entries.len() > MAX_SECRETS {
            return Err("The host secret catalog exceeds its entry limit");
        }
        for (name, entry) in &self.entries {
            entry.metadata.validate()?;
            if name != &entry.metadata.name || uuid::Uuid::parse_str(&entry.slot).is_err() {
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

pub fn validate_name(name: &str) -> Result<(), &'static str> {
    let characters = name.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
    if name.is_empty() || name.len() > 128 || !characters || matches!(name, "." | "..") {
        return Err("Secret names must contain 1–128 ASCII letters, digits, dots, underscores or hyphens");
    }
    Ok(())
}

pub fn validate_value(value: &str) -> Result<(), &'static str> {
    if value.is_empty() || value.len() > MAX_SECRET_BYTES {
        return Err("Secret values must contain 1–65536 bytes");
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
