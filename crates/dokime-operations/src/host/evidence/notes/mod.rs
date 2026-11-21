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

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos_runtime::Identifier;

// Local Imports
use crate::host::execution_state;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::evidence::{AnnotationOwner, annotation_owner_owner};
use dokime_protocol::model::EvidenceScope;
use dokime_storage::persisted::Scope;

/* === Public Functions === */

pub(crate) fn owner(context: &Context, owner: &AnnotationOwner) -> Result<EvidenceScope> {
    let kind = owner.owner.as_ref().and_then(|owner| owner.kind.as_ref());
    let (cycle, execution) = match kind {
        Some(annotation_owner_owner::Kind::CycleId(cycle)) => (cycle.as_str(), None),
        Some(annotation_owner_owner::Kind::Execution(owner)) => (owner.cycle_id.as_str(), Some(owner.execution_id.as_str())),
        Some(annotation_owner_owner::Kind::PlannedCase(_)) => {
            return Err(Error::Invalid("Notes: Select an Execution or Cycle; Planned Cases Were Replaced by Frozen Origins".into()));
        }
        None => return Err(Error::Invalid("Notes: Select an Execution or Cycle Owner".into())),
    };
    if let Some(execution) = execution {
        let snapshot = execution_state::read(context, execution)?;
        if snapshot.execution.cycle != cycle {
            return Err(Error::Invalid("Notes: Select an Existing Execution in Its Own Cycle".into()));
        }
    } else {
        let store = context.host.store();
        let exists = store.client_cycle(cycle)?.is_some() || !store.cycle_execution_page(cycle, "", NonZeroUsize::MIN)?.is_empty();
        if !exists {
            return Err(Error::Invalid("Notes: Select an Existing Execution or Cycle".into()));
        }
    }
    Ok(EvidenceScope {
        cycle_id: Some(cycle.into()),
        execution_id: execution.map(str::to_owned),
        ..Default::default()
    })
}

pub(crate) fn storage_scope(scope: &EvidenceScope) -> Result<Identifier> {
    if let Some(execution) = scope.execution_id.as_deref() {
        return Ok(Scope::Execution(execution).identifier()?);
    }
    if let Some(cycle) = scope.cycle_id.as_deref() {
        return Ok(Scope::Cycle(cycle).identifier()?);
    }
    Err(Error::Invalid("Write Note: Select an Execution or Cycle Owner".into()))
}

/* === Private Functions === */

/* === Tests === */
