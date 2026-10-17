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
use crate::model::execution::RecordScope;
use crate::model::identity::Identifier;
use crate::model::records::RecordSeverity;
use crate::model::records::evidence::RecordEntry;
use crate::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordMarker {
    pub sequence: u64,
    pub timestamp: Timestamp,
    pub record_type: Identifier,
    pub label: String,
    pub severity: RecordSeverity,
    pub scope: Option<RecordScope>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos_domain::MemorySize for RecordMarker {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.timestamp), std::mem::size_of_val(&self.timestamp)),
            (harmos_domain::MemorySize::memory_size(&self.record_type), std::mem::size_of_val(&self.record_type)),
            (harmos_domain::MemorySize::memory_size(&self.label), std::mem::size_of_val(&self.label)),
            (harmos_domain::MemorySize::memory_size(&self.scope), std::mem::size_of_val(&self.scope)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl RecordMarker {
    pub fn from_record(record: &RecordEntry) -> Self {
        Self {
            sequence: record.sequence,
            timestamp: record.timestamp.clone(),
            record_type: record.record_type.clone(),
            label: record.message.clone().unwrap_or_else(|| record.record_type.as_str().to_owned()),
            severity: record.severity,
            scope: record.scope.clone(),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
