/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod boolean;
pub mod multiple_choice;
pub mod number;
pub mod path;
pub mod secret;
pub mod text_area;

/* === Re-Exports === */
pub use boolean::Boolean;
pub use multiple_choice::{Choice, MultipleChoice};
pub use number::Number;
pub use path::Path;
pub use secret::{SecretReference, SecretSetting};
pub use text_area::TextArea;

/* === Imports === */
// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};
// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use crate::core::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub type SettingValues = BTreeMap<String, Value>;

/* === Structs === */
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub contributions: Vec<SettingsEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsError {
    pub id: String,
    pub message: String,
}

/* === Enums === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SettingsEntry {
    Boolean(Boolean),
    TextArea(TextArea),
    Path(Path),
    Number(Number),
    MultipleChoice(MultipleChoice),
    SecretReference(SecretSetting),
}

/* === Traits === */

/* === Implementations === */
impl SettingsError {
    pub(crate) fn invalid(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "setting {}: {}", self.id, self.message)
    }
}

impl std::error::Error for SettingsError {}

impl Settings {
    pub fn validate(&self) -> crate::Result<()> {
        let mut ids = BTreeSet::new();
        for definition in &self.contributions {
            let id = definition.id();
            if crate::core::Identifier::new(id).is_err() || id.trim() != id {
                return Err(SettingsError::invalid(
                    id,
                    "id must be nonempty without surrounding whitespace",
                )
                .into());
            }
            if !ids.insert(id) {
                return Err(SettingsError::invalid(id, "duplicate id").into());
            }
            if definition.title().trim().is_empty() {
                return Err(SettingsError::invalid(id, "title must not be empty").into());
            }
            definition.validate()?;
        }
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&SettingsEntry> {
        self.contributions.iter().find(|entry| entry.id() == id)
    }

    pub fn resolve(&self, saved: &SettingValues) -> crate::Result<SettingValues> {
        self.validate()?;
        let mut values: SettingValues = self
            .contributions
            .iter()
            .filter_map(|entry| {
                entry
                    .default_value()
                    .map(|value| (entry.id().into(), value))
            })
            .collect();
        for (id, value) in saved {
            let definition = self
                .get(id)
                .ok_or_else(|| SettingsError::invalid(id, "unknown id"))?;
            definition.validate_value(value)?;
            values.insert(id.clone(), value.clone());
        }
        Ok(values)
    }

    pub fn secrets(&self) -> impl Iterator<Item = &SecretSetting> {
        self.contributions.iter().filter_map(|entry| match entry {
            SettingsEntry::SecretReference(secret) => Some(secret),
            _ => None,
        })
    }
}

impl SettingsEntry {
    pub fn id(&self) -> &str {
        match self {
            Self::Boolean(d) => &d.id,
            Self::TextArea(d) => &d.id,
            Self::Path(d) => &d.id,
            Self::Number(d) => &d.id,
            Self::MultipleChoice(d) => &d.id,
            Self::SecretReference(d) => &d.id,
        }
    }
    pub fn title(&self) -> &str {
        match self {
            Self::Boolean(d) => &d.title,
            Self::TextArea(d) => &d.title,
            Self::Path(d) => &d.title,
            Self::Number(d) => &d.title,
            Self::MultipleChoice(d) => &d.title,
            Self::SecretReference(d) => &d.title,
        }
    }
    pub fn validate(&self) -> crate::Result<()> {
        match self {
            Self::Boolean(d) => d.validate(),
            Self::TextArea(d) => d.validate(),
            Self::Path(d) => d.validate(),
            Self::Number(d) => d.validate(),
            Self::MultipleChoice(d) => d.validate(),
            Self::SecretReference(d) => d.validate(),
        }
    }
    pub fn default_value(&self) -> Option<Value> {
        match self {
            Self::Boolean(d) => Some(d.default_value()),
            Self::TextArea(d) => Some(d.default_value()),
            Self::Path(d) => Some(d.default_value()),
            Self::Number(d) => Some(d.default_value()),
            Self::MultipleChoice(d) => Some(d.default_value()),
            Self::SecretReference(d) => d.default_value(),
        }
    }
    pub fn validate_value(&self, value: &Value) -> crate::Result<()> {
        match self {
            Self::Boolean(d) => d.validate_value(value),
            Self::TextArea(d) => d.validate_value(value),
            Self::Path(d) => d.validate_value(value),
            Self::Number(d) => d.validate_value(value),
            Self::MultipleChoice(d) => d.validate_value(value),
            Self::SecretReference(d) => d.validate_value(value),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
