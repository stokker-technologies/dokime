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
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use crate::model::execution::RecordScope;
use crate::model::identity::IdentifierPattern;
use crate::model::telemetry::TelemetrySample;
use crate::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryFilter {
    pub streams: Vec<IdentifierPattern>,
    pub schema_versions: Vec<u32>,
    pub sources: Vec<String>,
    pub scope: Option<RecordScope>,
    pub start: Option<Timestamp>,
    pub end: Option<Timestamp>,
    pub signals: Vec<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl TelemetryFilter {
    pub fn matches(&self, sample: &TelemetrySample) -> bool {
        if !self.streams.is_empty() && !self.streams.iter().any(|pattern| pattern.matches(sample.stream.as_str())) {
            return false;
        }
        if !self.schema_versions.is_empty() && !self.schema_versions.contains(&sample.schema_version) {
            return false;
        }
        if !self.sources.is_empty() && !self.sources.contains(&sample.source) {
            return false;
        }
        if let Some(scope) = &self.scope {
            let matches = if scope.test_execution_id.is_some() || scope.test_cycle_id.is_some() {
                sample.is_visible_in(scope)
            } else {
                sample.scope.as_ref() == Some(scope)
            };
            if !matches {
                return false;
            }
        }
        if let Some(start) = &self.start
            && &sample.timestamp < start
        {
            return false;
        }
        if let Some(end) = &self.end
            && &sample.timestamp > end
        {
            return false;
        }
        if !self.signals.is_empty() && !self.signals.iter().all(|signal| sample.values.contains_key(signal)) {
            return false;
        }

        true
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
