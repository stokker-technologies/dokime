/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::{Deserialize, Serialize};

use crate::model::execution::RecordScope;
use crate::model::identity::HealthItemId;
use crate::model::time::Timestamp;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthItem {
    pub health_item_id: HealthItemId,
    pub scope: RecordScope,
    pub severity: HealthSeverity,
    pub message: String,
    pub detail: Option<String>,
    pub observed_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthSeverity {
    Info,
    Warning,
    Error,
}

impl harmos::MemorySize for HealthItem {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.health_item_id), std::mem::size_of_val(&self.health_item_id)),
            (harmos::MemorySize::memory_size(&self.scope), std::mem::size_of_val(&self.scope)),
            (harmos::MemorySize::memory_size(&self.message), std::mem::size_of_val(&self.message)),
            (harmos::MemorySize::memory_size(&self.detail), std::mem::size_of_val(&self.detail)),
            (harmos::MemorySize::memory_size(&self.observed_at), std::mem::size_of_val(&self.observed_at)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}
