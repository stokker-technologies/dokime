/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::Arc;

use dokime::harmos::Context;
use tonic::Status;

use super::execution;
use super::value::canonical;
use crate::grpc::transcode::Transcode;
use dokime::app::LiveRuntime;
use dokime::domain::model::identity::TestExecutionId;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::operations::observation as wire;
use dokime::runtime::progress::{OperationOutcome, OperationSnapshot};
use dokime::runtime::query::RuntimeChange;

const SERVER_SHELL_RESPONSE_BYTES: u64 = 524_288;
const SERVER_TRACEABILITY_RESPONSE_BYTES: u64 = 524_288;
const SERVER_TRACEABILITY_SCAN_ITEMS: u64 = 8_192;
// A Quiet Watch Repeats Its Current Frame Well Inside The Client's 30 s Idle Deadline.
pub const WATCH_HEARTBEAT: std::time::Duration = std::time::Duration::from_secs(10);

pub(super) const STREAM: &str = "runtime.state";

struct Resources {
    pub(super) runtime: LiveRuntime,
    pub(super) max_message_bytes: usize,
}

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum PreparedOperations {
    Operation0(wire::GetRuntimeShell),
    Operation1(wire::WatchRuntime),
    Operation2(wire::WatchExecutionProgress),
    Operation3(wire::GetTraceability),
}

impl Serve<wire::GetRuntimeShell> for Resources {
    fn serve<'a>(
        &'a self, request: wire::GetRuntimeShell, _: Context<'a, wire::GetRuntimeShell>,
    ) -> Served<'a, dokime::domain::model::observation::wire::RuntimeShellSnapshot> {
        Box::pin(async move {
            let snapshot = dokime::app::live_runtime(&self.runtime).await?.watch_state().latest();
            let state = snapshot.state();
            let observed = canonical::cursor(&snapshot);
            let limit =
                observed.page(request.limit, &request.after_cycle_id, request.cursor.as_ref()).map_err(canonical::cursor_failure)?;
            let start = after_cycle_position(state, &request.after_cycle_id)?;
            let response_limit =
                self.max_message_bytes.min(usize::try_from(SERVER_SHELL_RESPONSE_BYTES).expect("validated shell response size"));

            let hub = dokime::app::live_runtime(&self.runtime).await?.hub_metadata();

            let mut entries = state.cycles.values().skip(start);
            let count = state.cycles.len().saturating_sub(start).min(limit);
            let mut cycle_summaries = canonical::slots(count)?;
            for cycle in entries.by_ref().take(limit) {
                cycle_summaries.push(execution::summary(state, cycle, observed.clone())?);
            }
            let next_after_cycle_id = if entries.next().is_some() {
                let last = cycle_summaries.last().ok_or_else(|| Status::internal("empty continued cycle page"))?;

                Some(last.cycle_id.clone())
            } else {
                None
            };
            let counts = dokime::app::live_runtime(&self.runtime)
                .await?
                .evidence_counts_at(&snapshot)
                .await
                .map_err(|error| Status::internal(error.to_string()))?;
            let response = dokime::domain::model::observation::wire::RuntimeShellSnapshot {
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
            Ok::<_, Status>((response).into())
        })
    }
}

impl Serve<wire::WatchRuntime> for Resources {
    fn serve<'a>(&'a self, request: wire::WatchRuntime, ctx: Context<'a, wire::WatchRuntime>) -> Served<'a, ()> {
        Box::pin(async move {
            let call_context = ctx.call();
            let output = ctx.streams();
            let revoked = crate::grpc::access::revocation(call_context);
            crate::grpc::access::during(revoked, async {
                let mut watch = dokime::app::live_runtime(&self.runtime).await?.watch_state();
                let mut snapshot = watch.latest();
                canonical::cursor(&snapshot).validate(request.cursor.as_ref(), false).map_err(canonical::cursor_failure)?;
                let mut operations = watch.operations();
                let mut resynchronized = true;
                loop {
                    let observed = canonical::cursor(&snapshot);
                    let frames = operation_frames(&operations)?;
                    output
                        .send_owned(
                            (dokime::domain::model::observation::wire::RuntimeStateFrame {
                                cursor: Some(observed),
                                resynchronized,
                                shutting_down: snapshot.state().shutting_down,
                                cycle_count: snapshot.state().cycles.len() as u64,
                                operations: frames,
                            })
                            .into(),
                        )
                        .await
                        .map_err(crate::grpc::chunked::output_failure)?;
                    let Ok(change) = tokio::time::timeout(WATCH_HEARTBEAT, watch.next_change()).await else {
                        resynchronized = false;
                        continue;
                    };
                    match change {
                        None => return Ok(().into()),
                        Some(RuntimeChange::State(next)) => {
                            snapshot = next;
                            operations = watch.operations();
                        }
                        Some(RuntimeChange::Operations(next)) => operations = next,
                    }
                    resynchronized = false;
                }
            })
            .await
        })
    }
}

impl Serve<wire::WatchExecutionProgress> for Resources {
    fn serve<'a>(&'a self, request: wire::WatchExecutionProgress, ctx: Context<'a, wire::WatchExecutionProgress>) -> Served<'a, ()> {
        Box::pin(async move {
            let call_context = ctx.call();
            let output = ctx.streams();
            let revoked = crate::grpc::access::revocation(call_context);
            crate::grpc::access::during(revoked, async {
                let execution = TestExecutionId::from_uuid(request.test_execution_id.clone())
                    .map_err(|_| Status::invalid_argument("invalid test execution UUID"))?;
                let mut watch = dokime::app::live_runtime(&self.runtime).await?.watch_execution_progress();
                let mut snapshot = watch.latest();
                let mut previous = None;
                let mut resynchronized = true;
                let mut heartbeat = false;
                loop {
                    let current = snapshot.get(&execution);
                    if resynchronized || heartbeat || current != previous.as_ref() {
                        output
                            .send_owned((progress_frame(snapshot.revision(), resynchronized, current)).into())
                            .await
                            .map_err(crate::grpc::chunked::output_failure)?;
                        previous = current.cloned();
                    }
                    resynchronized = false;
                    let Ok(next) = tokio::time::timeout(WATCH_HEARTBEAT, watch.next()).await else {
                        heartbeat = true;
                        continue;
                    };
                    let Some(next) = next else {
                        return Ok(().into());
                    };
                    heartbeat = false;
                    snapshot = next;
                }
            })
            .await
        })
    }
}

impl Serve<wire::GetTraceability> for Resources {
    fn serve<'a>(
        &'a self, request: wire::GetTraceability, _: Context<'a, wire::GetTraceability>,
    ) -> Served<'a, dokime::domain::model::observation::wire::TraceabilityPage> {
        Box::pin(async move {
            let snapshot = dokime::app::live_runtime(&self.runtime).await?.watch_state().latest();
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
            let (source, has_more, host_traceability_fingerprint) = dokime::app::live_runtime(&self.runtime)
                .await?
                .traceability_snapshots_page_at(
                    &snapshot,
                    requested_scope.as_ref(),
                    after,
                    limit,
                    usize::try_from(SERVER_TRACEABILITY_SCAN_ITEMS).expect("validated traceability scan count"),
                    usize::try_from(SERVER_TRACEABILITY_RESPONSE_BYTES).expect("validated traceability response size"),
                )
                .await
                .map_err(|error| Status::resource_exhausted(error.to_string()))?;
            if !after.is_empty()
                && (request.host_traceability_fingerprint.is_empty()
                    || request.host_traceability_fingerprint != host_traceability_fingerprint)
            {
                return Err(Status::aborted("host traceability changed; restart pagination from the first page"));
            }
            let response_limit = self
                .max_message_bytes
                .min(usize::try_from(SERVER_TRACEABILITY_RESPONSE_BYTES).expect("validated traceability response size"));

            let next_after_traceability_id = if has_more {
                let last = source.last().ok_or_else(|| Status::internal("empty continued traceability page"))?;

                last.traceability_id.to_string()
            } else {
                String::new()
            };
            let mut snapshots = canonical::slots(source.len())?;
            for snapshot in &source {
                snapshots.push(super::history::traceability(snapshot)?);
            }
            let response = dokime::domain::model::observation::wire::TraceabilityPage {
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
            Ok::<_, Status>((response).into())
        })
    }
}

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, max_message_bytes: usize,
) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources {
        runtime: runtime.clone(),
        max_message_bytes,
    });

    setup.resource::<Host<wire::GetRuntimeShell>>(Host::<wire::GetRuntimeShell>::new(served.clone()));
    setup.resource::<Host<wire::WatchRuntime>>(Host::<wire::WatchRuntime>::new(served.clone()));
    setup.resource::<Host<wire::WatchExecutionProgress>>(Host::<wire::WatchExecutionProgress>::new(served.clone()));
    setup.resource::<Host<wire::GetTraceability>>(Host::<wire::GetTraceability>::new(served.clone()));

    setup.operations::<PreparedOperations>(dokime::app::Lanes::Open);
    Ok(())
}

fn operation_frames(operations: &OperationSnapshot) -> Result<Vec<dokime::domain::model::observation::wire::OperationProgress>, Status> {
    operations
        .operations()
        .iter()
        .map(|operation| {
            let (status, finished_at, failure) = match &operation.outcome {
                OperationOutcome::Running => (dokime::domain::model::observation::wire::OperationStatus::Running, None, None),
                OperationOutcome::Finished { at } => {
                    (dokime::domain::model::observation::wire::OperationStatus::Finished, Some(canonical::timestamp(at)?), None)
                }
                OperationOutcome::Failed { at, failure } => (
                    dokime::domain::model::observation::wire::OperationStatus::Failed,
                    Some(canonical::timestamp(at)?),
                    Some(failure.clone()),
                ),
            };
            Ok(dokime::domain::model::observation::wire::OperationProgress {
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

fn progress_frame(
    revision: u64, resynchronized: bool, value: Option<&dokime::domains::execution::progress::LiveExecutionProgress>,
) -> dokime::domain::model::observation::wire::ExecutionProgressFrame {
    dokime::domain::model::observation::wire::ExecutionProgressFrame {
        revision,
        resynchronized,
        // Exact Allocations: Display Formatting Overallocates Past The Reserved Bytes
        progress: value.map(|value| dokime::domain::model::observation::wire::ExecutionProgress {
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

fn after_cycle_position(state: &dokime::runtime::RuntimeState, after: &str) -> Result<usize, Status> {
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

fn evidence_counts(counts: &dokime::domains::evidence::EvidenceCounts) -> dokime::domain::model::observation::wire::EvidenceCounts {
    dokime::domain::model::observation::wire::EvidenceCounts {
        records: counts.records as u64,
        telemetry: counts.telemetry as u64,
        prompts: counts.prompts as u64,
        artifacts: counts.artifacts as u64,
        health: counts.health as u64,
        traceability: counts.traceability as u64,
        notes: counts.notes as u64,
    }
}
