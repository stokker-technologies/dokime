/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Pure Record Collections And Their State Transitions.

pub mod resources;
pub mod set;

pub use resources::ResourceRegistry;

use serde::{Deserialize, Serialize};

use crate::model::execution::RecordScope;
use crate::model::records::Record;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRecords {
    pub through: u64,
    pub session: crate::state::records::set::TestRecordSet,
}

#[harmos_domain::record(id = "dokime.captured_record")]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapturedRecord {
    pub record: Record,
    pub ordinal: u64,
    pub scopes: Vec<RecordScope>,
}

#[harmos_domain::records]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Records {
    Record(CapturedRecord),
}

impl harmos_domain::MemorySize for CapturedRecord {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(harmos_domain::MemorySize::memory_size(&self.record).saturating_sub(std::mem::size_of_val(&self.record)))
            .saturating_add(harmos_domain::MemorySize::memory_size(&self.scopes).saturating_sub(std::mem::size_of_val(&self.scopes)))
    }
}
