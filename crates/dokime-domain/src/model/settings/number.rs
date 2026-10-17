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
use crate::model::values::numeric_value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Number {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub default: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Number {
    pub fn validate(&self) -> crate::Result<()> {
        if self.min.is_some_and(|v| !v.is_finite())
            || self.max.is_some_and(|v| !v.is_finite())
            || self.step.is_some_and(|v| !v.is_finite() || v <= 0.0)
            || matches!((self.min,self.max), (Some(a),Some(b)) if a>b)
        {
            return Err(SettingsError::invalid(&self.id, "invalid numeric bounds or step").into());
        }
        self.validate_value(&self.default_value())
    }
    pub fn default_value(&self) -> Value {
        Value::Number(self.default)
    }
    pub fn validate_value(&self, value: &Value) -> crate::Result<()> {
        let Some(number) = numeric_value(value) else {
            return Err(SettingsError::invalid(&self.id, "expected finite number").into());
        };
        if self.min.is_some_and(|min| number < min) || self.max.is_some_and(|max| number > max) {
            return Err(SettingsError::invalid(&self.id, "number is outside bounds").into());
        }
        if let Some(step) = self.step {
            let units = (number - self.min.unwrap_or(0.0)) / step;
            if !units.is_finite() || (units - units.round()).abs() > 1e-9 {
                return Err(
                    SettingsError::invalid(&self.id, "number does not align with step").into(),
                );
            }
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
