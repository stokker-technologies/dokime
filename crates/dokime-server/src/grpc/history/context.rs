/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod memory;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::sync::Arc;
// Third Party Imports
use harmos::{
    MemorySize, Protobuf,
    operations::{Context, Operation, Response},
};
use tonic::Status;
// Local Imports
use super::document::canonical::wire as document_wire;
use super::wire;
use crate::grpc::{
    appearance, execution, notes, prompts,
    value::{self, canonical},
};
use dokime_app::{
    RuntimeHandle,
    execution::{CycleStatus, CycleView},
    runtime::query::execution_history::ExecutionCheckpointContext,
};
use dokime_contract::operations::{common, execution as execution_wire, history};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}
struct Get {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Operation for Get {
    type Declaration = history::GetCheckpointContext;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    fn new(resources: Arc<Resources>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut history::GetCheckpointContextRequest,
        _: &mut (),
    ) -> Result<Response<history::CheckpointContext>, Status> {
        let checkpoint = wire::checkpoint(request.checkpoint.take())?;
        self.resources
            .runtime
            .execution_checkpoint_context(checkpoint, |context, allowance| {
                let required = context
                    .memory_size()
                    .saturating_add(memory::count(&context)?);
                if required > allowance.get() {
                    return Err(exhausted());
                }
                let response =
                    checkpoint_context(context, self.resources.maximum).map_err(original_error)?;
                if response.encoded_len() > self.resources.maximum {
                    return Err(exhausted());
                }
                Ok(response)
            })
            .await
            .map_err(wire::checkpoint_failure)?
            .try_into_response()
            .map_err(|source| wire::checkpoint_failure(source.into()))
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<Get>(
        "dokime",
        dokime_app::app::Lanes::Open,
        Arc::new(Resources {
            runtime: runtime.clone(),
            maximum: maximum.min(4 * 1024 * 1024),
        }),
    )?;
    Ok(())
}

/* === Private Functions === */
pub(super) fn checkpoint_context(
    value: ExecutionCheckpointContext,
    maximum: usize,
) -> Result<history::CheckpointContext, Status> {
    let cycle = Some(cycle_context(value.cycle, maximum)?);
    let execution_documents = collect(
        value
            .execution_documents
            .values()
            .map(|document| document_wire(document, &value.scope)),
    )?;
    let prompts = collect(value.prompts.iter().map(prompts::canonical::instance))?;
    let health = collect(value.health.iter().map(notes::health))?;
    let notes = collect(value.notes.iter().map(notes::note))?;
    let traceability = collect(value.traceability.iter().map(wire::traceability))?;
    Ok(history::CheckpointContext {
        checkpoint: Some(wire::checkpoint_wire(value.checkpoint)?),
        session_id: value.session_id.as_str().to_owned(),
        execution_id: value
            .scope
            .test_execution_id
            .as_ref()
            .map(|id| id.as_str().to_owned()),
        execution_scope: Some(canonical::scope(&value.scope)?),
        cycle,
        execution_documents,
        prompts,
        health,
        notes,
        traceability,
        // The captured historical frame includes only a data URL for a logo;
        // the asset identity is intentionally not invented without its digest.
        appearance: Some(appearance::historical(value.appearance)?),
    })
}

fn cycle_context(
    value: dokime_app::runtime::ExecutionCycleContext,
    maximum: usize,
) -> Result<history::HistoricalCycleContext, Status> {
    let cycle = cycle(value.cycle, &value.cases, maximum)?;
    Ok(history::HistoricalCycleContext {
        cycle: Some(cycle),
        execution_order: collect(
            value
                .execution_order
                .into_iter()
                .map(|id| Ok(id.as_str().to_owned())),
        )?,
    })
}

fn cycle(
    value: CycleView,
    cases: &[dokime_app::definition::TestCase],
    maximum: usize,
) -> Result<history::HistoricalCycle, Status> {
    let mut bounds = execution::Bounds::new(maximum);
    let cases = collect(cases.iter().enumerate().map(|(ordinal, case)| {
        execution::historical_case_summary(&value, case, ordinal, &mut bounds)
    }))?;
    Ok(history::HistoricalCycle {
        cycle_id: value.test_cycle_id.as_str().to_owned(),
        metadata: Some(canonical_metadata(&value.metadata)?),
        lifecycle: cycle_status(value.lifecycle),
        created_at: Some(canonical::timestamp(&value.created_at)?),
        started_at: value
            .started_at
            .as_ref()
            .map(canonical::timestamp)
            .transpose()?,
        updated_at: Some(canonical::timestamp(&value.updated_at)?),
        cases,
        result: value.result.as_ref().map(execution::result_summary),
    })
}

fn cycle_status(value: CycleStatus) -> execution_wire::CycleStatus {
    match value {
        CycleStatus::Previewed => execution_wire::CycleStatus::Previewed,
        CycleStatus::Ready => execution_wire::CycleStatus::Ready,
        CycleStatus::Running => execution_wire::CycleStatus::Running,
        CycleStatus::Paused => execution_wire::CycleStatus::Paused,
        CycleStatus::Stopping => execution_wire::CycleStatus::Stopping,
        CycleStatus::Completed => execution_wire::CycleStatus::Completed,
        CycleStatus::Failed => execution_wire::CycleStatus::Failed,
        CycleStatus::Blocked => execution_wire::CycleStatus::Blocked,
        CycleStatus::Stopped => execution_wire::CycleStatus::Stopped,
        CycleStatus::Cleared => execution_wire::CycleStatus::Cleared,
    }
}

fn canonical_metadata(value: &dokime_app::core::Metadata) -> Result<common::Metadata, Status> {
    canonical::metadata(value)
}
fn collect<T>(source: impl ExactSizeIterator<Item = Result<T, Status>>) -> Result<Vec<T>, Status> {
    let mut values = value::projection::slots(source.len())
        .map_err(|source| wire::checkpoint_failure(source.into()))?;
    for item in source {
        values.push(item?);
    }
    Ok(values)
}
fn original_error(source: Status) -> harmos::Error {
    if source.code() == tonic::Code::ResourceExhausted {
        return exhausted();
    }
    harmos::Error::backend(
        "project original checkpoint context",
        harmos::BackendFailure::Corrupt,
        source,
    )
}
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "canonical checkpoint context",
    }
}

/* === Tests === */
