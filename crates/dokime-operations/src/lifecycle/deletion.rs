/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Guarded Native Execution Scope Deletion

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos_operations::storage::{MutateRequest, MutationId, MutationItem};
use harmos_protocol::model::Receipt;
use harmos_runtime::operations::OperationManager;

// Local Imports
use super::{admission::Closed, io::Version, mutations, seal};
use crate::{Error, Result};
use dokime_storage::persisted::{Scope, shapes::{Execution, Seal, State}};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Guard {
    pub case: String,
    pub cycle: String,
    pub seal: Seal,
}

#[derive(Clone)]
pub struct Plan {
    request: MutateRequest,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn prepare(
    operations: &OperationManager, mutation: MutationId, closed: &Closed, execution: Version<Execution>, guard: Guard, rows: NonZeroUsize,
) -> Result<Plan> {
    closed.require(&execution.value)?;
    let value = execution.value;
    let scope = Scope::Execution(&value.key).identifier()?;
    if value.scope != scope || value.case != guard.case || value.cycle != guard.cycle {
        return Err(Error::Permission("Delete Execution: Select Its Exact Execution, Case and Cycle Owners".into()));
    }
    if value.state != State::Sealed || value.seal.as_ref() != Some(&guard.seal) {
        return Err(Error::Precondition("Delete Execution: Refresh Its Current Immutable Seal Before Deleting".into()));
    }
    seal::require_finished_files(operations, &value, rows).await?;
    // Scope Deletion Releases Its Retained C Owner in the Same Backend Transaction.
    // Independent Reader Leases Remain Owned by Their Readers.
    let request = MutateRequest { mutation: Some(mutation), items: vec![MutationItem::DeleteScope(scope)] };
    Ok(Plan { request })
}

pub async fn delete(operations: &OperationManager, plan: &Plan, bytes: NonZeroUsize) -> Result<Receipt> {
    mutations::mutate(operations, plan.request.clone(), bytes).await
}

/* === Private Functions === */

/* === Tests === */
