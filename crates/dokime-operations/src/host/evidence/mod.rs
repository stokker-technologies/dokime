/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

pub mod archive;
pub mod health_events;
pub mod logs;
pub mod notes;
pub mod records;
pub mod samples;
pub mod selection;
pub mod statistics;
pub mod traceability;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports

// Local Imports
use crate::host::{Context, execution_state};
use crate::{Error, Result};
use dokime_protocol::host::CyclePhase;
use dokime_storage::ExecutionState;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn require_open(context: &Context, cycle: Option<&str>, execution: Option<&str>, action: &str) -> Result<()> {
    let cycle = if let Some(execution) = execution {
        let snapshot = execution_state::read(context, execution)?;
        let owner = snapshot.execution.cycle;
        if cycle.is_some_and(|cycle| cycle != owner) {
            return Err(Error::Invalid(format!("{action}: Select Execution {execution} in Its Own Cycle {owner}")));
        }
        if snapshot.state != ExecutionState::Open {
            return Err(Error::Invalid(format!(
                "{action}: Execution {execution} Is Closed or Sealed; Attach New Evidence to Its Unsealed Cycle {owner}"
            )));
        }
        owner
    } else {
        cycle.ok_or_else(|| Error::Invalid(format!("{action}: Select an Execution or Cycle")))?.to_owned()
    };
    let store = context.host.store();
    if let Some(summary) = store.client_cycle(&cycle)? {
        if summary.sealed || summary.phase == CyclePhase::Sealing as i32 {
            return Err(Error::Invalid(format!("{action}: Cycle {cycle} Is Sealed or Sealing; Create a New Cycle for New Evidence")));
        }
        return Ok(());
    }
    if store.cycle_execution_page(&cycle, "", NonZeroUsize::MIN)?.is_empty() {
        return Err(Error::NotFound(format!("{action}: Select an Existing Cycle {cycle}")));
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
