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
use crate::records::RecordEntry;
use dokime::core::{RecordScope, SessionId, TestCycleId};

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
pub(super) fn matches(cycle: Option<&TestCycleId>, scope: Option<&RecordScope>) -> bool {
    scope.is_none_or(|scope| {
        // Ancestor Context Includes Unexecuted Case Notes In This Cycle
        (scope.test_cycle_id.is_some()
            || (scope.test_case_id.is_none()
                && scope.step_id.is_none()
                && scope.criterion_id.is_none()))
            && scope.test_execution_id.is_none()
            && scope
                .test_cycle_id
                .as_ref()
                .is_none_or(|id| Some(id) == cycle)
    })
}

/* === Private Functions === */

/* === Tests === */
