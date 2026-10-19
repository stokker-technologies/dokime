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
use crate::records::RecordPattern;
use crate::records::RecordSeverity;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::ArtifactId;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::TelemetryStreamId;
use dokime_domain::model::records::evidence::RecordEntry;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordFilter {
    #[serde(default)]
    pub patterns: Vec<RecordPattern>,
    pub record_types: Vec<Identifier>,
    pub severities: Vec<RecordSeverity>,
    pub min_severity: Option<RecordSeverity>,
    pub sources: Vec<String>,
    pub scope: Option<RecordScope>,
    pub start: Option<Timestamp>,
    pub end: Option<Timestamp>,
    pub related_artifact_id: Option<ArtifactId>,
    pub related_stream_id: Option<TelemetryStreamId>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RecordFilter {
    pub fn matches(&self, record: &RecordEntry) -> bool {
        if !self.patterns.is_empty() && !self.patterns.iter().any(|pattern| pattern.matches(record.record_type.as_str())) {
            return false;
        }
        if !self.record_types.is_empty() && !self.record_types.contains(&record.record_type) {
            return false;
        }
        if !self.severities.is_empty() && !self.severities.contains(&record.severity) {
            return false;
        }
        if let Some(minimum) = self.min_severity
            && record.severity < minimum
        {
            return false;
        }
        if !self.sources.is_empty() && !self.sources.contains(&record.source) {
            return false;
        }
        if let Some(scope) = &self.scope
            && record.scope.as_ref() != Some(scope)
        {
            return false;
        }
        if let Some(start) = &self.start
            && &record.timestamp < start
        {
            return false;
        }
        if let Some(end) = &self.end
            && &record.timestamp > end
        {
            return false;
        }
        if let Some(artifact_id) = &self.related_artifact_id
            && record.related_artifact_id.as_ref() != Some(artifact_id)
        {
            return false;
        }
        if let Some(stream_id) = &self.related_stream_id
            && record.related_stream_id.as_ref() != Some(stream_id)
        {
            return false;
        }

        true
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
