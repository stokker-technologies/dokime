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
use crate::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Boolean {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub default: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Boolean {
    pub fn validate(&self) -> crate::Result<()> {
        self.validate_value(&self.default_value())
    }
    pub fn default_value(&self) -> Value {
        Value::Bool(self.default)
    }
    pub fn validate_value(&self, value: &Value) -> crate::Result<()> {
        if matches!(value, Value::Bool(_)) {
            Ok(())
        } else {
            Err(SettingsError::invalid(&self.id, "invalid boolean value").into())
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
