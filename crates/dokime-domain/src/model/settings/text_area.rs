/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::Deserialize;
use serde::Serialize;

use super::SettingsError;
use crate::model::values::Value;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextArea {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub default: String,

    #[serde(default)]
    pub required: bool,
}

impl TextArea {
    pub fn validate(&self) -> crate::Result<()> {
        self.validate_value(&self.default_value())
    }
    pub fn default_value(&self) -> Value {
        Value::Text(self.default.clone())
    }
    pub fn validate_value(&self, value: &Value) -> crate::Result<()> {
        if matches!(value, Value::Text(text) if !self.required || !text.trim().is_empty()) {
            Ok(())
        } else {
            Err(SettingsError::invalid(&self.id, "invalid text_area value").into())
        }
    }
}
