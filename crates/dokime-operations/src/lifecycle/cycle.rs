/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Cycle Transitions

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos_operations::storage::{MutateRequest, MutationId, MutationItem};
use harmos_protocol::model::{PageRequest, ReadCut, Receipt};
use harmos_runtime::operations::OperationManager;
use harmos_runtime::storage::query::ObjectVersions;
use prost::Message;

// Local Imports
use super::io::{self, Version};
use super::{mutations, rows};
use crate::{Error, Result};
use dokime_protocol::host::{ExecutionOutcome, ResultStatus, execution_outcome::Attribution};
use dokime_storage::persisted::{
    Scope,
    bindings::Protobuf,
    records::CycleEvent,
    shapes::{Cycle, Execution, Outcomes, State},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Image {
    pub outcomes: Outcomes,
    pub settled: bool,
    pub at: ReadCut,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn create(
    operations: &OperationManager, mutation: MutationId, cycle: Cycle, receipt_ns: i64, bytes: NonZeroUsize,
) -> Result<Receipt> {
    if cycle.scope != Scope::Cycle(&cycle.key).identifier()? || cycle.sealed || cycle.outcomes != Default::default() {
        return Err(Error::Invalid("Create Cycle: Select Its Exact Scope With an Open, Empty Lifecycle".into()));
    }
    cycle.metadata.decode(bytes)?;
    let event = CycleEvent {
        execution: String::new(),
        kind: "created".into(),
        receipt_ns,
    };
    let items = vec![MutationItem::Create(io::encode(&cycle, bytes)?), mutations::event(&mutation, "cycle", cycle.scope, &event, bytes)?];
    mutations::mutate(operations, MutateRequest { mutation: Some(mutation), items }, bytes).await
}

pub async fn set_metadata(
    operations: &OperationManager, mutation: MutationId, cycle: Version<Cycle>, metadata: Protobuf, receipt_ns: i64, bytes: NonZeroUsize,
) -> Result<Receipt> {
    if cycle.value.sealed {
        return Err(Error::Precondition("Set Cycle: Select an Unsealed Cycle".into()));
    }
    metadata.decode(bytes)?;
    let value = Cycle { metadata, ..cycle.value };
    let event = CycleEvent {
        execution: String::new(),
        kind: "metadata".into(),
        receipt_ns,
    };
    let items = vec![
        MutationItem::Set {
            expected: cycle.revision,
            value: io::encode(&value, bytes)?,
        },
        mutations::event(&mutation, "cycle", value.scope, &event, bytes)?,
    ];
    mutations::mutate(operations, MutateRequest { mutation: Some(mutation), items }, bytes).await
}

/// Current Summaries Derive From Surviving Bindings, Including After Scope Deletion.
pub async fn image(
    operations: &OperationManager, cycle: &str, mut at: Option<ReadCut>, rows: NonZeroUsize, bytes: NonZeroUsize,
) -> Result<Image> {
    let mut outcomes = Outcomes::default();
    let mut settled = true;
    let mut token = String::new();
    loop {
        let page = rows::list::<Execution>(
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
        for execution in rows::decode::<Execution>(&page, bytes)? {
            if execution.cycle != cycle {
                continue;
            }
            if execution.state != State::Sealed {
                settled = false;
                continue;
            }
            let outcome = execution
                .outcome
                .ok_or_else(|| Error::Precondition("Read Cycle Outcomes: Restore Every Sealed Execution's Attribution".into()))?;
            outcomes = attribute(&outcomes, &outcome)?;
        }
        token = page.next_page_token;
        if token.is_empty() {
            break;
        }
    }
    let at = at.ok_or_else(|| Error::Precondition("Read Cycle Outcomes: Return Their Issued Storage Cut".into()))?;
    Ok(Image { outcomes, settled, at })
}

pub async fn finish(
    operations: &OperationManager, mutation: MutationId, cycle: Version<Cycle>, receipt_ns: i64, rows: NonZeroUsize, bytes: NonZeroUsize,
) -> Result<Receipt> {
    if cycle.value.sealed {
        return Err(Error::Precondition("Seal Cycle: Select an Unsealed Cycle".into()));
    }
    let image = image(operations, &cycle.value.key, Some(cycle.at), rows, bytes).await?;
    if !image.settled {
        return Err(Error::Precondition("Seal Cycle: Settle and Seal Every Surviving Execution First".into()));
    }
    let value = Cycle {
        sealed: true,
        outcomes: image.outcomes,
        ..cycle.value
    };
    let event = CycleEvent {
        execution: String::new(),
        kind: "sealed".into(),
        receipt_ns,
    };
    let items = vec![
        MutationItem::Set {
            expected: cycle.revision,
            value: io::encode(&value, bytes)?,
        },
        mutations::event(&mutation, "cycle", value.scope, &event, bytes)?,
    ];
    mutations::mutate(operations, MutateRequest { mutation: Some(mutation), items }, bytes).await
}

/* === Private Functions === */

pub(super) fn attribute(counts: &Outcomes, saved: &Protobuf) -> Result<Outcomes> {
    if saved.message != "dokime.host.v1.ExecutionOutcome" {
        return Err(Error::Invalid("Seal Execution: Supply Its Frozen Execution Outcome".into()));
    }
    let outcome = ExecutionOutcome::decode(saved.payload.as_slice())?;
    let mut counts = counts.clone();
    let count = match outcome.attribution {
        Some(Attribution::Declared(_)) => &mut counts.declared,
        Some(Attribution::NotEvaluated(_)) => &mut counts.not_evaluated,
        Some(Attribution::Evaluated(value)) => match ResultStatus::try_from(value.status) {
            Ok(ResultStatus::Passed) => &mut counts.passed,
            Ok(ResultStatus::Failed) => &mut counts.failed,
            Ok(ResultStatus::Blocked) => &mut counts.blocked,
            Ok(ResultStatus::Cancelled) => &mut counts.skipped,
            _ => return Err(Error::Invalid("Seal Execution: Supply a Valid Attributed Result Status".into())),
        },
        None => return Err(Error::Invalid("Seal Execution: Attribute Its Outcome Before Sealing".into())),
    };
    *count = count
        .checked_add(1)
        .ok_or_else(|| Error::Bound("Seal Execution: Start a New Cycle Before Its Outcome Counter Overflows".into()))?;
    Ok(counts)
}

/* === Tests === */
