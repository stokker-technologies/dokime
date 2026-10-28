/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;

use crate::model::telemetry::TelemetryColumn;
use crate::model::values::FieldError;
use crate::model::values::Value;

pub trait TelemetryRow: Sized {
    fn columns() -> Vec<TelemetryColumn>;

    fn into_row(self) -> BTreeMap<String, Value>;

    fn from_row(values: &BTreeMap<String, Value>) -> Result<Self, FieldError>;
}
