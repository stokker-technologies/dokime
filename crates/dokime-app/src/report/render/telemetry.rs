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
use serde::{Serialize, Serializer, ser::SerializeMap};
// Local Imports
use super::ReportValue;
use dokime::telemetry::TelemetrySample;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct SampleValues<'a>(pub(super) &'a TelemetrySample);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Serialize for SampleValues<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let has_timestamp = self.0.values.contains_key("timestamp");
        let length = self.0.values.len() + usize::from(!has_timestamp);
        let mut fields = serializer.serialize_map(Some(length))?;
        if !has_timestamp {
            fields.serialize_entry("timestamp", &self.0.timestamp)?;
        }
        for (key, value) in &self.0.values {
            fields.serialize_entry(key, &ReportValue(value))?;
        }
        fields.end()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
