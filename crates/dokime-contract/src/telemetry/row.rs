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

// Local Imports
use crate::core::Value;
use crate::telemetry::TelemetryColumn;
use crate::values::FieldError;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */
pub trait TelemetryRow: Sized {
    fn columns() -> Vec<TelemetryColumn>;

    fn into_row(self) -> BTreeMap<String, Value>;

    fn from_row(values: &BTreeMap<String, Value>) -> Result<Self, FieldError>;
}

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
