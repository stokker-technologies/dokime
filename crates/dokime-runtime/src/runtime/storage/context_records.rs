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
use harmos::MemorySize;

// Local Imports
use dokime_domain::model::records::evidence::RecordEntry;
use dokime_domain::model::identity::SessionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextRecord {
    pub epoch: SessionId,
    pub record: RecordEntry,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl MemorySize for ContextRecord {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(
                self.epoch
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<SessionId>()),
            )
            .saturating_add(
                self.record
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<RecordEntry>()),
            )
    }
}

/* === Public Functions === */
/* === Private Functions === */

/* === Tests === */
