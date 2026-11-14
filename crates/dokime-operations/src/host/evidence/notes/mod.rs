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

pub mod create;
pub mod get;
pub mod list;
pub mod set;
pub mod watch;

/* === Imports === */

use crate::{Error, Result, host::Context};
use dokime_protocol::model::EvidenceScope;
use dokime_storage::ExecutionState;
use std::num::NonZeroUsize;

/* === Public Functions === */

pub(crate) fn require_open(context: &Context, scope: &EvidenceScope) -> Result<()> {
    let store = context.host.store();
    if let Some(execution) = &scope.execution_id {
        let cut = store.metadata(execution)?;
        if cut.state == ExecutionState::Open && scope.cycle_id.as_ref().is_none_or(|cycle| cycle == &cut.execution.cycle) {
            return Ok(());
        }
    } else if let Some(cycle) = &scope.cycle_id {
        let mut after = String::new();
        loop {
            let entries = store.execution_identifier_page(Some(cycle), &after, NonZeroUsize::new(context.settings.page_items).unwrap())?;
            if entries.is_empty() {
                break;
            }
            for execution in entries {
                after = execution.clone();
                if store.metadata(&execution)?.state == ExecutionState::Open {
                    return Ok(());
                }
            }
        }
    }
    Err(Error::Precondition("Write Note: Select an Open Owner; Closed Evidence and Published Reports Are Immutable".into()))
}
