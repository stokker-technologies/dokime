/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Execution Admission and Lifecycle Transitions

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeSet, num::NonZeroUsize};

// Third Party Imports
use harmos_operations::storage::{MutateRequest, MutationId, MutationItem};
use harmos_protocol::model::Receipt;
use harmos_runtime::operations::OperationManager;
use harmos_runtime::storage::{ObjectDefinition, ObjectKey, ObjectVersionToken, StoredObject};

// Local Imports
use super::admission::Permit;
use super::io::{self, Version};
use super::{cycle, mutations};
use crate::{Error, Result};
use dokime_protocol::model::Termination;
use dokime_storage::persisted::{
    Case,
    bindings::Protobuf,
    records::{CycleEvent, ExecutionEvent},
    shapes::{Cycle, Execution, Parameters, Preview, Progress, State, Steps, Stream},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Admission {
    pub execution: Execution,
    pub parameters: Parameters,
    pub steps: Steps,
    pub progress: Progress,
    pub streams: Vec<Stream>,
    pub receipt_ns: i64,
}

pub(super) struct Close {
    pub mutation: MutationId,
    pub execution: Version<Execution>,
    pub outcome: Protobuf,
    pub termination: i32,
    pub streams: Vec<Version<Stream>>,
    pub receipt_ns: i64,
}

pub struct Revision<'a> {
    pub mutation: MutationId,
    pub permit: &'a Permit,
    pub execution: Version<Execution>,
    pub receipt_ns: i64,
}

#[derive(Clone)]
pub struct Plan {
    request: MutateRequest,
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

pub async fn prepare(operations: &OperationManager, mutation: MutationId, admission: Admission, bytes: NonZeroUsize) -> Result<Plan> {
    let value = &admission.execution;
    value.validate().map_err(|source| Error::Invalid(format!("Admit Execution: {source}")))?;
    if value.state != State::Open || value.seal.is_some() || value.outcome.is_some() || value.termination != 0 {
        return Err(Error::Invalid("Admit Execution: Supply an Open Execution Without a Seal or Outcome".into()));
    }
    let previews = value.previews.iter().collect::<BTreeSet<_>>();
    if previews.len() != value.previews.len() || previews.iter().any(|key| key.is_empty()) {
        return Err(Error::Invalid("Admit Execution: Enumerate Each Derived Object Key Once".into()));
    }
    let case = io::read::<Case>(operations, &value.case, None, bytes)
        .await?
        .ok_or_else(|| Error::NotFound("Admit Execution: Select a Registered Case".into()))?;
    if !case.value.cycle.is_empty() && case.value.cycle != value.cycle {
        return Err(Error::Invalid("Admit Execution: Select the Case's Registered Cycle".into()));
    }
    // Reserve Both Request and Reply Space Before Any Execution Can Be Admitted.
    let schema = ObjectDefinition::of::<Preview>().schema().map_err(|source| Error::Invalid(format!("Admit Execution: {source}")))?;
    let mut deletes = Vec::new();
    for key in &value.previews {
        let item = ObjectKey::new(key, bytes).map_err(|source| Error::Invalid(format!("Admit Execution: {source}")))?;
        deletes.push(MutationItem::Delete {
            schema: schema.id.clone(),
            item,
            // Preview Revisions Share This History and the Same Token Size.
            expected: Some(case.revision.clone()),
        });
    }
    mutations::bound_preview_deletions(&deletes, bytes)?;

    let scope = &value.scope;
    if admission.parameters.key != value.key
        || admission.steps.key != value.key
        || admission.progress.key != value.key
        || admission.parameters.scope != *scope
        || admission.steps.scope != *scope
        || admission.progress.scope != *scope
        || admission.streams.iter().any(|stream| stream.scope != *scope || stream.closed)
    {
        return Err(Error::Invalid("Admit Execution: Freeze Every Initial Object Under Its Execution Owner".into()));
    }
    value.origin.decode(bytes)?;
    admission.parameters.value.decode(bytes)?;
    admission.steps.value.decode(bytes)?;
    admission.progress.value.decode(bytes)?;
    let mut items = vec![
        // The Case Guard Also Refuses an Admission Racing Its Unexecuted-Only Deletion.
        MutationItem::Set {
            expected: case.revision,
            value: io::encode(&case.value, bytes)?,
        },
        MutationItem::Create(io::encode(value, bytes)?),
        MutationItem::Create(io::encode(&admission.parameters, bytes)?),
        MutationItem::Create(io::encode(&admission.steps, bytes)?),
        MutationItem::Create(io::encode(&admission.progress, bytes)?),
    ];
    for stream in &admission.streams {
        items.push(MutationItem::Create(io::encode(stream, bytes)?));
    }
    let event = ExecutionEvent {
        state: State::Open,
        kind: "admitted".into(),
        step: String::new(),
        operation: String::new(),
        attempt: 0,
        receipt_ns: admission.receipt_ns,
    };
    items.push(mutations::event(&mutation, "execution", value.scope.clone(), &event, bytes)?);
    if !value.cycle.is_empty() {
        let cycle = io::read::<Cycle>(operations, &value.cycle, None, bytes)
            .await?
            .ok_or_else(|| Error::NotFound("Admit Execution: Select an Existing Cycle".into()))?;
        if cycle.value.sealed {
            return Err(Error::Precondition("Admit Execution: Select an Unsealed Cycle".into()));
        }
        let event = CycleEvent {
            execution: value.key.clone(),
            kind: "execution-admitted".into(),
            receipt_ns: admission.receipt_ns,
        };
        items.push(MutationItem::Set {
            expected: cycle.revision,
            value: io::encode(&cycle.value, bytes)?,
        });
        items.push(mutations::event(&mutation, "cycle", cycle.value.scope, &event, bytes)?);
    }
    let request = MutateRequest { mutation: Some(mutation), items };
    Ok(Plan { request })
}

pub async fn admit(operations: &OperationManager, plan: &Plan, bytes: NonZeroUsize) -> Result<Receipt> {
    mutations::mutate(operations, plan.request.clone(), bytes).await
}

pub(super) async fn close(operations: &OperationManager, request: Close, bytes: NonZeroUsize) -> Result<Receipt> {
    let Close {
        mutation,
        execution,
        outcome,
        termination,
        streams,
        receipt_ns,
    } = request;
    if execution.value.state != State::Open {
        return Err(Error::Precondition("Close Execution: Select an Open Execution After Settling Its Intake".into()));
    }
    outcome.decode(bytes)?;
    cycle::attribute(&Default::default(), &outcome)?;
    if Termination::try_from(termination).ok().is_none_or(|termination| termination == Termination::Unspecified) {
        return Err(Error::Invalid("Close Execution: Select a Recorded Termination".into()));
    }
    let value = Execution {
        state: State::Closed,
        outcome: Some(outcome),
        termination,
        ..execution.value
    };
    let event = ExecutionEvent {
        state: State::Closed,
        kind: "closed".into(),
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
    for stream in streams {
        if stream.value.scope != value.scope {
            return Err(Error::Permission("Close Execution: Settle Only Its Own Stream Bindings".into()));
        }
        let stream_value = Stream { closed: true, ..stream.value };
        items.push(MutationItem::Set {
            expected: stream.revision,
            value: io::encode(&stream_value, bytes)?,
        });
    }
    mutations::mutate(operations, MutateRequest { mutation: Some(mutation), items }, bytes).await
}

pub async fn set_progress(
    operations: &OperationManager, revision: Revision<'_>, progress: Version<Progress>, value: Protobuf, bytes: NonZeroUsize,
) -> Result<Receipt> {
    value.decode(bytes)?;
    let value = Progress { value, ..progress.value };
    revise(operations, revision, progress.revision, value, "progress", bytes).await
}

pub async fn set_parameters(
    operations: &OperationManager, revision: Revision<'_>, parameters: Version<Parameters>, value: Protobuf, bytes: NonZeroUsize,
) -> Result<Receipt> {
    value.decode(bytes)?;
    let value = Parameters { value, ..parameters.value };
    revise(operations, revision, parameters.revision, value, "parameters", bytes).await
}

pub async fn set_steps(
    operations: &OperationManager, revision: Revision<'_>, steps: Version<Steps>, value: Protobuf, bytes: NonZeroUsize,
) -> Result<Receipt> {
    value.decode(bytes)?;
    let value = Steps { value, ..steps.value };
    revise(operations, revision, steps.revision, value, "steps", bytes).await
}

pub async fn append_transition(
    operations: &OperationManager, mutation: MutationId, permit: &Permit, execution: Version<Execution>, event: ExecutionEvent,
    bytes: NonZeroUsize,
) -> Result<Receipt> {
    permit.require(&execution.value)?;
    if execution.value.state != State::Open || event.state != State::Open {
        return Err(Error::Precondition("Append Execution Transition: Select an Open Execution".into()));
    }
    let items = vec![
        MutationItem::Set {
            expected: execution.revision,
            value: io::encode(&execution.value, bytes)?,
        },
        mutations::event(&mutation, "execution", execution.value.scope, &event, bytes)?,
    ];
    mutations::mutate(operations, MutateRequest { mutation: Some(mutation), items }, bytes).await
}

/* === Private Functions === */

async fn revise<T: StoredObject<Key = str>>(
    operations: &OperationManager, revision: Revision<'_>, expected: ObjectVersionToken, value: T, kind: &str, bytes: NonZeroUsize,
) -> Result<Receipt> {
    let Revision { mutation, permit, execution, receipt_ns } = revision;
    permit.require(&execution.value)?;
    if execution.value.state != State::Open || value.scope() != Some(&execution.value.scope) || value.key() != execution.value.key {
        return Err(Error::Precondition("Revise Execution: Select an Open Execution and Its Own Resource Revision".into()));
    }
    let event = ExecutionEvent {
        state: State::Open,
        kind: kind.into(),
        step: String::new(),
        operation: String::new(),
        attempt: 0,
        receipt_ns,
    };
    let items = vec![
        MutationItem::Set {
            expected: execution.revision,
            value: io::encode(&execution.value, bytes)?,
        },
        MutationItem::Set {
            expected,
            value: io::encode(&value, bytes)?,
        },
        mutations::event(&mutation, "execution", execution.value.scope, &event, bytes)?,
    ];
    mutations::mutate(operations, MutateRequest { mutation: Some(mutation), items }, bytes).await
}

/* === Tests === */
