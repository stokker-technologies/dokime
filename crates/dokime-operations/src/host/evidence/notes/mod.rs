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

// Third Party Imports
use harmos_runtime::Identifier;

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::model::EvidenceScope;
use dokime_storage::persisted::Scope;

/* === Public Functions === */

pub(crate) fn storage_scope(scope: &EvidenceScope) -> Result<Identifier> {
    if let Some(execution) = scope.execution_id.as_deref() {
        return Ok(Scope::Execution(execution).identifier()?);
    }
    if let Some(cycle) = scope.cycle_id.as_deref() {
        return Ok(Scope::Cycle(cycle).identifier()?);
    }
    Err(Error::Invalid("Write Note: Select an Execution or Cycle Owner".into()))
}

pub(crate) fn require_open(context: &Context, scope: &EvidenceScope) -> Result<()> {
    Ok(context.host.store().require_evidence_owner(scope.cycle_id.as_deref(), scope.execution_id.as_deref(), "Write Note")?)
}
