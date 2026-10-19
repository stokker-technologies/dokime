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
use std::collections::BTreeSet;
// Third Party Imports
use serde::Deserialize;
use serde::Serialize;
// Local Imports
use super::SettingsError;
use crate::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Choice {
    pub value: String,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MultipleChoice {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub default: String,
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub provider: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl MultipleChoice {
    pub fn validate(&self) -> crate::Result<()> {
        let mut values = BTreeSet::new();
        // A list filled at run time may declare none of its choices here: the
        // route named beside it is what an open card asks. One declared
        // statically still has to be a list an operator could pick from.
        if (self.choices.is_empty() && self.provider.is_none())
            || self.choices.iter().any(|c| c.value.trim().is_empty() || c.title.trim().is_empty() || !values.insert(&c.value))
        {
            return Err(SettingsError::invalid(&self.id, "choices must be nonempty and unique").into());
        }
        self.validate_value(&self.default_value())
    }
    pub fn default_value(&self) -> Value {
        Value::Text(self.default.clone())
    }
    pub fn validate_value(&self, value: &Value) -> crate::Result<()> {
        // A route-filled list admits whatever that route offered, which the
        // host cannot check without asking it; a declared one admits exactly
        // what it declared.
        let admitted = match self.provider {
            Some(_) => matches!(value, Value::Text(_)),
            None => {
                matches!(value, Value::Text(text) if self.choices.iter().any(|c| c.value == *text))
            }
        };
        if admitted { Ok(()) } else { Err(SettingsError::invalid(&self.id, "value is not a declared choice").into()) }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
