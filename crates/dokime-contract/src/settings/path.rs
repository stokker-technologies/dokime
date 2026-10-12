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

// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use super::SettingsError;
use crate::core::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Path {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub default: String,
    #[serde(default)]
    pub required: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Path {
    pub fn validate(&self) -> crate::Result<()> {
        self.validate_value(&self.default_value())
    }
    pub fn default_value(&self) -> Value {
        Value::Text(self.default.clone())
    }
    pub fn validate_value(&self, value: &Value) -> crate::Result<()> {
        if matches!(value, Value::Text(text) if !text.contains("\0") && (!self.required || !text.trim().is_empty()))
        {
            Ok(())
        } else {
            Err(SettingsError::invalid(&self.id, "invalid path value").into())
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
