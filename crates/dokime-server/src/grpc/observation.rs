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
// Standard Library Imports
use std::sync::Arc;
// Third Party Imports
use crate::grpc::transcode::Transcode;
use harmos::domain::Portable;
use harmos::{
    MemorySize,
    operations::{Context, Operation, StreamOutput},
};
use tonic::Status;
// Local Imports
use super::value::canonical;
use super::{execution, value};
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::operations::observation as wire;
use dokime_runtime::RuntimeHandle;
use dokime_runtime::runtime::progress::{OperationOutcome, OperationSnapshot};
use dokime_runtime::runtime::query::RuntimeChange;

/* === Constants === */
pub(super) const STREAM: &str = "runtime.state";
const MAX_SHELL_RESPONSE_BYTES: usize = 512 * 1024;
const MAX_TRACEABILITY_RESPONSE_BYTES: usize = 512 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    pub(super) runtime: RuntimeHandle,
    pub(super) max_message_bytes: usize,
}

struct Traceability {
    resources: Arc<Resources>,
}

struct Shell {
    resources: Arc<Resources>,
}
struct Runtime {
    resources: Arc<Resources>,
}
struct Progress {
    resources: Arc<Resources>,
}

/* === Enums === */
#[harmos::catalog]
enum Operations {
    Traceability(Traceability),
    Shell(Shell),
    Runtime(Runtime),
    Progress(Progress),
}

/* === Traits === */

/* === Implementations === */
impl Operation for Shell {
    type Declaration = wire::GetRuntimeShell;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::GetRuntimeShell, _: &mut (),
    ) -> Result<harmos::operations::Response<dokime_domain::model::observation::wire::RuntimeShellSnapshot>, Status> {
        let snapshot = self.resources.runtime.watch_state().latest();
        let state = snapshot.state();
        let observed = canonical::cursor(&snapshot);
        let limit = observed.page(request.limit, &request.after_cycle_id, request.cursor.as_ref()).map_err(canonical::cursor_failure)?;
        let start = after_cycle_position(state, &request.after_cycle_id)?;
        let response_limit = self.resources.max_message_bytes.min(MAX_SHELL_RESPONSE_BYTES);
        let mut bounds = execution::Bounds::new(response_limit);
        bounds.bytes(std::mem::size_of::<dokime_domain::model::observation::wire::RuntimeShellSnapshot>())?;
        bounds.text(state.session_id.as_str())?;
        bounds.text(&observed.epoch)?;
        bounds.text(&observed.stream)?;
        let hub = self.resources.runtime.hub_metadata();
        bounds.bytes(canonical::metadata_memory(&hub)?)?;
        let mut entries = state.cycles.values().skip(start);
        let count = state.cycles.len().saturating_sub(start).min(limit);
        let mut cycle_summaries = canonical::slots(count)?;
        for cycle in entries.by_ref().take(limit) {
            cycle_summaries.push(execution::summary(state, cycle, observed.clone(), &mut bounds)?);
        }
        let next_after_cycle_id = if entries.next().is_some() {
            let last = cycle_summaries.last().ok_or_else(|| Status::internal("empty continued cycle page"))?;
            bounds.text(&last.cycle_id)?;
            Some(last.cycle_id.clone())
        } else {
            None
        };
        let counts = self.resources.runtime.evidence_counts_at(&snapshot).await.map_err(|error| Status::internal(error.to_string()))?;
        let response = dokime_domain::model::observation::wire::RuntimeShellSnapshot {
            session_id: state.session_id.to_string(),
            revision: snapshot.revision(),
            shutting_down: state.shutting_down,
            hub_metadata: Some(canonical::metadata(&hub)?),
            test_suite_count: state.suites.suites().count() as u64,
            cycle_summaries,
            evidence_counts: Some(evidence_counts(&counts)),
            observed_cursor: Some(observed),
            next_after_cycle_id,
        };
        if response.wire_len()? > response_limit {
            return Err(Status::resource_exhausted("runtime shell page exceeds response budget; lower limit and retry"));
        }
        Ok((response).into())
    }
}

impl Operation for Runtime {
    type Declaration = wire::WatchRuntime;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<dokime_domain::model::observation::wire::RuntimeStateFrame>;
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::WatchRuntime, output: &mut Self::Endpoints,
    ) -> Result<harmos::operations::Response<()>, Status> {
        let mut watch = self.resources.runtime.watch_state();
        let mut snapshot = watch.latest();
        canonical::cursor(&snapshot).validate(request.cursor.as_ref(), false).map_err(canonical::cursor_failure)?;
        let mut operations = watch.operations();
        let mut resynchronized = true;
        loop {
            let observed = canonical::cursor(&snapshot);
            let bytes = std::mem::size_of::<dokime_domain::model::observation::wire::RuntimeStateFrame>()
                .saturating_add(observed.epoch.len())
                .saturating_add(observed.stream.len());
            // The frames are built first: they are bounded, and measured exactly.
            let frames = operation_frames(&operations)?;
            let bytes = bytes.saturating_add(
                frames.memory_size().saturating_sub(std::mem::size_of::<Vec<dokime_domain::model::observation::wire::OperationProgress>>()),
            );
            output
                .send_with(bytes, || {
                    Ok(dokime_domain::model::observation::wire::RuntimeStateFrame {
                        cursor: Some(observed),
                        resynchronized,
                        shutting_down: snapshot.state().shutting_down,
                        cycle_count: snapshot.state().cycles.len() as u64,
                        operations: frames,
                    })
                })
                .await
                .map_err(stream_failure)?;
            match watch.next_change().await {
                None => return Ok(().into()),
                Some(RuntimeChange::State(next)) => {
                    snapshot = next;
                    operations = watch.operations();
                }
                Some(RuntimeChange::Operations(next)) => operations = next,
            }
            resynchronized = false;
        }
    }
}

impl Operation for Progress {
    type Declaration = wire::WatchExecutionProgress;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<dokime_domain::model::observation::wire::ExecutionProgressFrame>;
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::WatchExecutionProgress, output: &mut Self::Endpoints,
    ) -> Result<harmos::operations::Response<()>, Status> {
        let execution = TestExecutionId::from_uuid(request.test_execution_id.clone())
            .map_err(|_| Status::invalid_argument("invalid test execution UUID"))?;
        let mut watch = self.resources.runtime.watch_execution_progress();
        let mut snapshot = watch.latest();
        let mut previous = None;
        let mut resynchronized = true;
        loop {
            let current = snapshot.get(&execution);
            if resynchronized || current != previous.as_ref() {
                let bytes = progress_memory(current)?;
                output
                    .send_with(bytes, || Ok(progress_frame(snapshot.revision(), resynchronized, current)))
                    .await
                    .map_err(stream_failure)?;
                previous = current.cloned();
            }
            let Some(next) = watch.next().await else {
                return Ok(().into());
            };
            snapshot = next;
            resynchronized = false;
        }
    }
}

impl Operation for Traceability {
    type Declaration = wire::GetTraceability;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::GetTraceability, _: &mut (),
    ) -> Result<harmos::operations::Response<dokime_domain::model::observation::wire::TraceabilityPage>, Status> {
        let snapshot = self.resources.runtime.watch_state().latest();
        let observed = canonical::cursor(&snapshot);
        let limit =
            observed.page(request.limit, &request.after_traceability_id, request.cursor.as_ref()).map_err(canonical::cursor_failure)?;
        let requested_scope = request.scope.as_ref().map(canonical::decoded_scope).transpose()?;
        if requested_scope
            .as_ref()
            .and_then(|scope| scope.session_id.as_ref())
            .is_some_and(|session| session != &snapshot.state().session_id)
        {
            return Err(Status::failed_precondition("traceability scope epoch mismatch; resynchronize"));
        }
        let after = &request.after_traceability_id;
        let (source, has_more, host_traceability_fingerprint) = self
            .resources
            .runtime
            .traceability_snapshots_page_at(&snapshot, requested_scope.as_ref(), after, limit, 8_192, MAX_TRACEABILITY_RESPONSE_BYTES)
            .await
            .map_err(|error| Status::resource_exhausted(error.to_string()))?;
        if !after.is_empty()
            && (request.host_traceability_fingerprint.is_empty() || request.host_traceability_fingerprint != host_traceability_fingerprint)
        {
            return Err(Status::aborted("host traceability changed; restart pagination from the first page"));
        }
        let response_limit = self.resources.max_message_bytes.min(MAX_TRACEABILITY_RESPONSE_BYTES);
        let mut bounds = execution::Bounds::new(response_limit);
        bounds.bytes(std::mem::size_of::<dokime_domain::model::observation::wire::TraceabilityPage>())?;
        bounds.text(&observed.epoch)?;
        bounds.text(&observed.stream)?;
        bounds.text(&host_traceability_fingerprint)?;
        if let Some(scope) = &request.scope {
            bounds.bytes(harmos::MemorySize::memory_size(scope))?;
        }
        for snapshot in &source {
            bounds.bytes(super::history::traceability_memory(snapshot)?)?;
        }
        let next_after_traceability_id = if has_more {
            let last = source.last().ok_or_else(|| Status::internal("empty continued traceability page"))?;
            bounds.text(last.traceability_id.as_str())?;
            last.traceability_id.to_string()
        } else {
            String::new()
        };
        let mut snapshots = canonical::slots(source.len())?;
        for snapshot in &source {
            snapshots.push(super::history::traceability(snapshot)?);
        }
        let response = dokime_domain::model::observation::wire::TraceabilityPage {
            scope: request.scope.clone(),
            snapshots,
            cursor: Some(observed),
            next_after_traceability_id,
            complete: !has_more,
            host_traceability_fingerprint,
        };
        if response.wire_len()? > response_limit {
            return Err(Status::resource_exhausted("traceability page exceeds response budget; lower limit and retry"));
        }
        Ok((response).into())
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, max_message_bytes: usize) -> Result<(), super::Error> {
    runtime.operations().register_wire_operations::<Operations, Resources>(
        "dokime",
        dokime_runtime::app::Lanes::Open,
        Arc::new(Resources { runtime: runtime.clone(), max_message_bytes }),
    )?;
    Ok(())
}

/* === Private Functions === */
fn stream_failure(error: harmos::Error) -> Status {
    match error {
        harmos::Error::ResourceExhausted { .. } => Status::resource_exhausted(error.to_string()),
        harmos::Error::NotReady { .. } => Status::unavailable(error.to_string()),
        error => Status::internal(error.to_string()),
    }
}

fn operation_frames(operations: &OperationSnapshot) -> Result<Vec<dokime_domain::model::observation::wire::OperationProgress>, Status> {
    operations
        .operations()
        .iter()
        .map(|operation| {
            value::text(&operation.label)?;
            value::text(&operation.phase)?;
            for phase in &operation.phases {
                value::text(phase)?;
            }
            let (status, finished_at, failure) = match &operation.outcome {
                OperationOutcome::Running => (dokime_domain::model::observation::wire::OperationStatus::Running, None, None),
                OperationOutcome::Finished { at } => {
                    (dokime_domain::model::observation::wire::OperationStatus::Finished, Some(canonical::timestamp(at)?), None)
                }
                OperationOutcome::Failed { at, failure } => (
                    dokime_domain::model::observation::wire::OperationStatus::Failed,
                    Some(canonical::timestamp(at)?),
                    Some(failure.clone()),
                ),
            };
            Ok(dokime_domain::model::observation::wire::OperationProgress {
                operation_id: operation.operation_id.clone(),
                label: operation.label.clone(),
                phase: operation.phase.clone(),
                phases: operation.phases.clone(),
                fraction: operation.fraction,
                started_at: Some(canonical::timestamp(&operation.started_at)?),
                status,
                finished_at,
                failure,
            })
        })
        .collect()
}

fn progress_memory(value: Option<&dokime_runtime::domains::execution::progress::LiveExecutionProgress>) -> Result<usize, Status> {
    let mut bytes = std::mem::size_of::<dokime_domain::model::observation::wire::ExecutionProgressFrame>();
    if let Some(value) = value {
        value::text(value.test_execution_id.as_str())?;
        value::text(value.step.as_str())?;
        value::text(&value.progress.phase)?;
        bytes = bytes
            .saturating_add(value.test_execution_id.as_str().len())
            .saturating_add(value.step.as_str().len())
            .saturating_add(value.progress.phase.len());
    }
    Ok(bytes)
}

fn progress_frame(
    revision: u64, resynchronized: bool, value: Option<&dokime_runtime::domains::execution::progress::LiveExecutionProgress>,
) -> dokime_domain::model::observation::wire::ExecutionProgressFrame {
    dokime_domain::model::observation::wire::ExecutionProgressFrame {
        revision,
        resynchronized,
        // Exact Allocations: Display Formatting Overallocates Past The Reserved Bytes
        progress: value.map(|value| dokime_domain::model::observation::wire::ExecutionProgress {
            test_execution_id: value.test_execution_id.as_str().to_owned(),
            step_id: value.step.as_str().to_owned(),
            phase: value.progress.phase.clone(),
            completed: value.progress.completed,
            total: value.progress.total,
            elapsed_ms: value.progress.elapsed_ms,
            remaining_ms: value.progress.remaining_ms,
        }),
        cleared: value.is_none(),
    }
}

fn after_cycle_position(state: &dokime_runtime::runtime::RuntimeState, after: &str) -> Result<usize, Status> {
    if after.is_empty() {
        return Ok(0);
    }
    state
        .cycles
        .keys()
        .position(|cycle| cycle.as_str() == after)
        .map(|position| position + 1)
        .ok_or_else(|| Status::invalid_argument("after cycle ID not found"))
}

fn evidence_counts(counts: &dokime_runtime::domains::evidence::EvidenceCounts) -> dokime_domain::model::observation::wire::EvidenceCounts {
    dokime_domain::model::observation::wire::EvidenceCounts {
        records: counts.records as u64,
        telemetry: counts.telemetry as u64,
        prompts: counts.prompts as u64,
        artifacts: counts.artifacts as u64,
        health: counts.health as u64,
        traceability: counts.traceability as u64,
        notes: counts.notes as u64,
    }
}

/* === Tests === */
