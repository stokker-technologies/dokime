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
use crate::model::identity::HealthItemId;
use crate::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthItem {
    pub health_item_id: HealthItemId,
    pub scope: RecordScope,
    pub severity: HealthSeverity,
    pub message: String,
    pub detail: Option<String>,
    pub observed_at: Timestamp,
}

/* === Enums === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthSeverity {
    Info,
    Warning,
    Error,
}

/* === Traits === */

/* === Implementations === */
impl harmos_domain::MemorySize for HealthItem {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.health_item_id), std::mem::size_of_val(&self.health_item_id)),
            (harmos_domain::MemorySize::memory_size(&self.scope), std::mem::size_of_val(&self.scope)),
            (harmos_domain::MemorySize::memory_size(&self.message), std::mem::size_of_val(&self.message)),
            (harmos_domain::MemorySize::memory_size(&self.detail), std::mem::size_of_val(&self.detail)),
            (harmos_domain::MemorySize::memory_size(&self.observed_at), std::mem::size_of_val(&self.observed_at)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
