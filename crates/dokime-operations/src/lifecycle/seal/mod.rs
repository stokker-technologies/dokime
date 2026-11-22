/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Execution Seal Composition

/* === Modules === */

mod digest;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos_operations::storage::{
    DeleteSnapshot, DeleteSnapshotRequest, ListFileProductions, ListFileProductionsRequest, MutateRequest, MutationId, MutationItem,
    RetainSnapshot, RetainSnapshotRequest, SavedSnapshot,
};
use harmos_protocol::model::{PageRequest, ReadCut, Receipt};
use harmos_runtime::operations::OperationManager;
use harmos_runtime::storage::query::{ObjectVersions, Source};
use harmos_runtime::storage::{ObjectDefinition, ObjectKey, StorageCursor};
use prost::Message;

// Local Imports
use super::{
    admission::{Closed, DrainedIntake},
    cycle, execution,
    io::{self, Version},
    mutations, rows,
};
use crate::{Error, Result};
use dokime_storage::persisted::{
    bindings::Protobuf,
    records::{CycleEvent, ExecutionEvent},
    shapes::{Cycle, Execution, Preview, Seal, State, Stream},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Close<'a> {
    pub mutation: MutationId,
    pub closed: &'a Closed,
    pub drained: DrainedIntake,
    pub execution: Version<Execution>,
    pub outcome: Protobuf,
    pub termination: i32,
    pub receipt_ns: i64,
}

pub struct Prepare<'a> {
    pub mutation: MutationId,
    pub closed: &'a Closed,
    pub execution: Version<Execution>,
    pub sources: &'a [Source],
    pub receipt_ns: i64,
}

/// Preserve This Exact Plan With Its Admission for Mutation Receipt Replay.
#[derive(Clone)]
pub struct Plan {
    request: MutateRequest,
    pub evidence_cut: ReadCut,
    retained: SavedSnapshot,
    execution: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Plan {
    pub fn request(&self) -> &MutateRequest {
        &self.request
    }
}

/* === Public Functions === */

pub async fn close(operations: &OperationManager, request: Close<'_>, rows: NonZeroUsize, bytes: NonZeroUsize) -> Result<Receipt> {
    let Close {
        mutation,
        closed,
        drained,
        execution,
        outcome,
        termination,
        receipt_ns,
    } = request;
    drained.require(closed, &execution.value)?;
    let durable: StorageCursor = drained
        .receipt
        .durable_at
        .ok_or_else(|| Error::Precondition("Close Execution: Supply the Durable Intake Frontier".into()))?
        .try_into()
        .map_err(|source| Error::Invalid(format!("Close Execution: Restore the Intake Frontier: {source}")))?;
    if &durable.history != execution.revision.history() {
        return Err(Error::Invalid("Close Execution: Drain Intake Within This Execution's Storage History".into()));
    }
    require_finished_files(operations, &execution.value, rows).await?;
    let mut at = None;
    let mut token = String::new();
    let mut streams = Vec::new();
    loop {
        let page = rows::list::<Stream>(
            operations,
            None,
            ObjectVersions::Latest,
            at,
            PageRequest {
                page_size: rows.get() as u32,
                page_token: token,
                ..Default::default()
            },
        )
        .await?;
        at = page.resolved_at.clone();
        for stream in rows::decode::<Stream>(&page, bytes)? {
            if stream.scope != execution.value.scope {
                continue;
            }
            let stream = io::read::<Stream>(operations, &stream.key, at.clone(), bytes)
                .await?
                .ok_or_else(|| Error::Precondition("Close Execution: Preserve Every Admitted Stream Binding".into()))?;
            streams.push(stream);
        }
        token = page.next_page_token;
        if token.is_empty() {
            break;
        }
    }
    let request = execution::Close {
        mutation,
        execution,
        outcome,
        termination,
        streams,
        receipt_ns,
    };
    execution::close(operations, request, bytes).await
}

pub async fn prepare(operations: &OperationManager, request: Prepare<'_>, rows: NonZeroUsize, bytes: NonZeroUsize) -> Result<Plan> {
    let Prepare {
        mutation,
        closed,
        execution,
        sources,
        receipt_ns,
    } = request;
    closed.require(&execution.value)?;
    if execution.value.state != State::Closed || execution.value.seal.is_some() || execution.value.outcome.is_none() {
        return Err(Error::Precondition("Seal Execution: Durably Close Its Settled Evidence Before Issuing C".into()));
    }
    require_finished_files(operations, &execution.value, rows).await?;
    let response = io::call::<RetainSnapshot>(
        operations,
        RetainSnapshotRequest {
            at: None,
            scope: Some(execution.value.scope.clone()),
        },
    )
    .await?;
    let retained = response.snapshot.ok_or_else(|| Error::Precondition("Seal Execution: Retain Its Evidence Snapshot".into()))?;
    let at = response.resolved_at.ok_or_else(|| Error::Precondition("Seal Execution: Return the Issued Evidence Cut C".into()))?;
    let key = execution.value.key.clone();
    let prepared: Result<MutateRequest> = async {
        let digest = digest::evidence(operations, &execution.value.scope, &at, sources, rows, bytes).await?;
        let value = Execution {
            state: State::Sealed,
            seal: Some(Seal {
                evidence_cut: at.encode_to_vec(),
                retained_snapshot: retained.token.clone(),
                digest,
                receipt_ns,
            }),
            ..execution.value
        };
        let event = ExecutionEvent {
            state: State::Sealed,
            kind: "sealed".into(),
            step: String::new(),
            operation: String::new(),
            attempt: 0,
            receipt_ns,
        };
        let mut items = vec![
            MutationItem::Set {
                expected: execution.revision,
                value: io::encode(&value, bytes)?,
            },
            mutations::event(&mutation, "execution", value.scope.clone(), &event, bytes)?,
        ];
        if !value.cycle.is_empty() {
            let cycle = io::read::<Cycle>(operations, &value.cycle, None, bytes)
                .await?
                .ok_or_else(|| Error::NotFound("Seal Execution: Restore Its Owning Cycle".into()))?;
            if cycle.value.sealed {
                return Err(Error::Precondition("Seal Execution: Seal Its Execution Before Finalizing the Cycle".into()));
            }
            let outcomes = cycle::attribute(&cycle.value.outcomes, value.outcome.as_ref().unwrap())?;
            let cycle_value = Cycle { outcomes, ..cycle.value };
            let event = CycleEvent {
                execution: value.key.clone(),
                kind: "execution-sealed".into(),
                receipt_ns,
            };
            items.push(MutationItem::Set {
                expected: cycle.revision,
                value: io::encode(&cycle_value, bytes)?,
            });
            items.push(mutations::event(&mutation, "cycle", cycle_value.scope, &event, bytes)?);
        }
        let schema = ObjectDefinition::of::<Preview>().schema().map_err(|source| Error::Invalid(format!("Seal Execution: {source}")))?;
        for key in &value.previews {
            let Some(preview) = io::read::<Preview>(operations, key, Some(at.clone()), bytes).await? else {
                continue;
            };
            if preview.value.scope != value.scope {
                return Err(Error::Permission("Seal Execution: Clear Only Previews Owned by This Execution".into()));
            }
            let item = ObjectKey::new(key, bytes).map_err(|source| Error::Invalid(format!("Seal Execution: {source}")))?;
            items.push(MutationItem::Delete {
                schema: schema.id.clone(),
                item,
                expected: Some(preview.revision),
            });
        }
        Ok(MutateRequest { mutation: Some(mutation), items })
    }
    .await;
    let request = match prepared {
        Ok(request) => request,
        Err(source) => {
            io::call::<DeleteSnapshot>(operations, DeleteSnapshotRequest { snapshot: Some(retained) }).await?;
            return Err(source);
        }
    };
    Ok(Plan {
        request,
        evidence_cut: at,
        retained,
        execution: key,
    })
}

/// The Original Durable Receipt Supplies S; Neither Object Nor Event Predicts It.
/// A Conflicting Plan Still Owns C Until Explicitly Abandoned or Scope Deletion.
pub async fn publish(operations: &OperationManager, plan: &Plan, bytes: NonZeroUsize) -> Result<Receipt> {
    mutations::mutate(operations, plan.request.clone(), bytes).await
}

pub async fn abandon(operations: &OperationManager, plan: Plan, bytes: NonZeroUsize) -> Result<()> {
    if let Some(execution) = io::read::<Execution>(operations, &plan.execution, None, bytes).await?
        && execution.value.seal.as_ref().is_some_and(|seal| seal.retained_snapshot == plan.retained.token)
    {
        return Err(Error::Precondition(
            "Release Seal Preparation: Keep a Published Execution's Evidence Cut Until Its Scope Is Deleted".into(),
        ));
    }
    io::call::<DeleteSnapshot>(operations, DeleteSnapshotRequest { snapshot: Some(plan.retained) }).await?;
    Ok(())
}

pub(super) async fn require_finished_files(operations: &OperationManager, execution: &Execution, rows: NonZeroUsize) -> Result<()> {
    let mut at = None;
    let mut token = String::new();
    loop {
        let response = io::call::<ListFileProductions>(
            operations,
            ListFileProductionsRequest {
                scope: execution.scope.clone(),
                at,
                page: Some(PageRequest {
                    page_size: rows.get() as u32,
                    page_token: token,
                    ..Default::default()
                }),
            },
        )
        .await?;
        if !response.productions.is_empty() {
            return Err(Error::Precondition(
                "Settle Execution Files: Finish or Abandon Every Accepted File Production Before Closing or Deleting".into(),
            ));
        }
        at = response.resolved_at;
        token = response.next_page_token;
        if token.is_empty() {
            return Ok(());
        }
    }
}

/* === Private Functions === */

/* === Tests === */
