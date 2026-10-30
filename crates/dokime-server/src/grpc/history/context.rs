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

use crate::grpc::transcode::Transcode;

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use dokime::harmos::{Context, Response};
use tonic::Status;

// Local Imports
use super::document::canonical::wire as document_wire;
use super::wire;
use crate::grpc::{
    appearance, execution, notes, prompts,
    value::{self, canonical},
};
use dokime::app::LiveRuntime;
use dokime::domain::model::execution::result::CycleStatus;
use dokime::domain::model::execution::result::CycleView;
use dokime::operations::history;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::runtime::query::execution_history::ExecutionCheckpointContext;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: LiveRuntime,
    maximum: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Serve<history::GetCheckpointContext> for Resources {
    fn serve<'a>(
        &'a self, mut request: history::GetCheckpointContext, _: Context<'a, history::GetCheckpointContext>,
    ) -> Served<'a, dokime::domain::model::history::wire::CheckpointContext> {
        Box::pin(async move {
            let checkpoint = wire::checkpoint(request.checkpoint.take())?;
            let context = dokime::app::live_runtime(&self.runtime)
                .await?
                .execution_checkpoint_context(checkpoint)
                .await
                .map_err(wire::checkpoint_failure)?;
            let response = context
                .try_map(|context| -> dokime::runtime::Result<_> {
                    checkpoint_context(context, self.maximum).map_err(|error| original_error(error).into())
                })
                .map_err(wire::checkpoint_failure)?;
            if response.wire_len()? > self.maximum {
                return Err(Status::resource_exhausted("historical response exceeds transport limit"));
            }
            Ok::<_, Status>(Response::from_owner(response.owner()))
        })
    }
}

/* === Public Functions === */

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, maximum: usize,
) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });

    setup.resource::<Host<history::GetCheckpointContext>>(Host::<history::GetCheckpointContext>::new(served.clone()));

    Ok(())
}

pub(super) fn checkpoint_context(
    value: ExecutionCheckpointContext, maximum: usize,
) -> Result<dokime::domain::model::history::wire::CheckpointContext, Status> {
    let cycle = Some(cycle_context(value.cycle)?);
    let execution_documents = collect(value.execution_documents.values().map(|document| document_wire(document, &value.scope)))?;
    let prompts = collect(value.prompts.iter().map(prompts::canonical::instance))?;
    let health = collect(value.health.iter().map(notes::health))?;
    let notes = collect(value.notes.iter().map(notes::note))?;
    let traceability = collect(value.traceability.iter().map(wire::traceability))?;
    Ok(dokime::domain::model::history::wire::CheckpointContext {
        checkpoint: Some(wire::checkpoint_wire(value.checkpoint, maximum)?),
        session_id: value.session_id.as_str().to_owned(),
        execution_id: value.scope.test_execution_id.as_ref().map(|id| id.as_str().to_owned()),
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

/* === Private Functions === */

fn cycle_context(
    value: dokime::runtime::ExecutionCycleContext,
) -> Result<dokime::domain::model::history::wire::HistoricalCycleContext, Status> {
    let cycle = cycle(value.cycle, &value.cases)?;
    Ok(dokime::domain::model::history::wire::HistoricalCycleContext {
        cycle: Some(cycle),
        execution_order: collect(value.execution_order.into_iter().map(|id| Ok(id.as_str().to_owned())))?,
    })
}

fn cycle(
    value: CycleView, cases: &[dokime::definition::TestCase],
) -> Result<dokime::domain::model::history::wire::HistoricalCycle, Status> {
    let cases = collect(cases.iter().enumerate().map(|(ordinal, case)| execution::historical_case_summary(&value, case, ordinal)))?;
    Ok(dokime::domain::model::history::wire::HistoricalCycle {
        cycle_id: value.test_cycle_id.as_str().to_owned(),
        metadata: Some(canonical::metadata(&value.metadata)?),
        phase: value.phase,
        lifecycle: cycle_status(value.lifecycle),
        created_at: Some(canonical::timestamp(&value.created_at)?),
        started_at: value.started_at.as_ref().map(canonical::timestamp).transpose()?,
        updated_at: Some(canonical::timestamp(&value.updated_at)?),
        cases,
        result: value.result.as_ref().map(execution::result_summary),
    })
}

fn cycle_status(value: CycleStatus) -> dokime::domain::model::execution::wire::CycleStatus {
    match value {
        CycleStatus::Ready => dokime::domain::model::execution::wire::CycleStatus::Ready,
        CycleStatus::Running => dokime::domain::model::execution::wire::CycleStatus::Running,
        CycleStatus::Paused => dokime::domain::model::execution::wire::CycleStatus::Paused,
        CycleStatus::Stopping => dokime::domain::model::execution::wire::CycleStatus::Stopping,
        CycleStatus::Completed => dokime::domain::model::execution::wire::CycleStatus::Completed,
        CycleStatus::Failed => dokime::domain::model::execution::wire::CycleStatus::Failed,
        CycleStatus::Blocked => dokime::domain::model::execution::wire::CycleStatus::Blocked,
        CycleStatus::Stopped => dokime::domain::model::execution::wire::CycleStatus::Stopped,
    }
}

fn collect<T>(source: impl ExactSizeIterator<Item = Result<T, Status>>) -> Result<Vec<T>, Status> {
    let mut values = value::projection::slots(source.len()).map_err(|source| wire::checkpoint_failure(source.into()))?;
    for item in source {
        values.push(item?);
    }
    Ok(values)
}

fn original_error(source: Status) -> dokime::harmos::Error {
    if source.code() == tonic::Code::ResourceExhausted {
        return exhausted();
    }
    dokime::harmos::Error::backend("project original checkpoint context", dokime::harmos::BackendFailure::Corrupt, source)
}

fn exhausted() -> dokime::harmos::Error {
    dokime::harmos::Error::ResourceExhausted { resource: "canonical checkpoint context" }
}

/* === Tests === */
