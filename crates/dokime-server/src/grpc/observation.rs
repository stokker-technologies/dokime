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
use std::pin::Pin;
// Third Party Imports
use futures_util::{Stream, stream};
use prost::Message;
use tokio::sync::watch;
use tonic::{Request, Response, Status};
// Local Imports
use super::{execution, value};
use dokime::operations::cursor::{Cursor, Error as CursorError};
use dokime_app::{RuntimeHandle, core::TestExecutionId, runtime::query::RuntimeStateSnapshot};
use dokime_protocol::v1::{self, observation_server::Observation};

/* === Constants === */
pub(super) const STREAM: &str = "runtime.state";
const MAX_SHELL_RESPONSE_BYTES: usize = 512 * 1024;
const MAX_TRACEABILITY_RESPONSE_BYTES: usize = 512 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Service {
    pub(super) runtime: RuntimeHandle,
    pub(super) stopping: watch::Receiver<bool>,
    pub(super) max_message_bytes: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[tonic::async_trait]
impl Observation for Service {
    type WatchRuntimeStream =
        Pin<Box<dyn Stream<Item = Result<v1::RuntimeStateFrame, Status>> + Send + 'static>>;
    type WatchExecutionProgressStream =
        Pin<Box<dyn Stream<Item = Result<v1::ExecutionProgressFrame, Status>> + Send + 'static>>;

    async fn watch_execution_progress(
        &self,
        request: Request<v1::WatchExecutionProgressRequest>,
    ) -> Result<Response<Self::WatchExecutionProgressStream>, Status> {
        if *self.stopping.borrow() {
            return Err(Status::unavailable("listener is stopping"));
        }
        let execution = TestExecutionId::from_uuid(request.into_inner().test_execution_id)
            .map_err(|_| Status::invalid_argument("invalid test execution UUID"))?;
        let watch = self.runtime.watch_execution_progress();
        let initial = watch.latest();
        let initial_value = initial.get(&execution).cloned();
        let frames = stream::unfold(
            (
                watch,
                self.stopping.clone(),
                execution,
                Some((initial, initial_value)),
                None,
            ),
            |(mut watch, mut stopping, execution, initial, mut previous)| async move {
                if *stopping.borrow() {
                    return None;
                }
                let (snapshot, value, resynchronized) = if let Some((snapshot, value)) = initial {
                    (snapshot, value, true)
                } else {
                    loop {
                        let snapshot = tokio::select! { biased;
                            _=stopping.changed()=>return None,
                            next=watch.next()=>next?,
                        };
                        let value = snapshot.get(&execution).cloned();
                        if value != previous {
                            break (snapshot, value, false);
                        }
                    }
                };
                let frame = progress_frame(snapshot.revision(), resynchronized, value.as_ref());
                previous = value;
                Some((Ok(frame), (watch, stopping, execution, None, previous)))
            },
        );
        Ok(Response::new(Box::pin(frames)))
    }

    async fn get_runtime_shell(
        &self,
        request: Request<v1::GetRuntimeShellRequest>,
    ) -> Result<Response<v1::RuntimeShellSnapshot>, Status> {
        let request = request.into_inner();
        let snapshot = self.runtime.watch_state().latest();
        let state = snapshot.state();
        let epoch = state.process_epoch().to_string();
        let limit = page_at(
            request.limit,
            &request.after_cycle_id,
            request.cursor.as_ref(),
            &epoch,
            snapshot.revision(),
        )?;
        let cursor = cursor(&snapshot);
        let (cycle_summaries, next_after_cycle_id) = {
            let start = after_cycle_position(state, &request.after_cycle_id)?;
            let mut entries = state.cycles.values().skip(start);
            let mut page = Vec::with_capacity(limit);
            for cycle in entries.by_ref().take(limit) {
                page.push(execution::summary(state, cycle, cursor.clone())?);
            }
            let next = entries.next().map(|_| {
                page.last()
                    .expect("a next cycle follows a nonempty page")
                    .cycle_id
                    .clone()
            });
            (page, next)
        };

        let response = v1::RuntimeShellSnapshot {
            session_id: state.session_id.to_string(),
            revision: snapshot.revision(),
            shutting_down: state.shutting_down,
            hub_metadata: Some(value::checked_metadata(&self.runtime.hub_metadata())?),
            test_suite_count: state.suites.suites().count() as u64,
            cycle_summaries,
            evidence_counts: Some(evidence_counts(
                &self
                    .runtime
                    .evidence_counts_at(&snapshot)
                    .await
                    .map_err(|error| Status::internal(error.to_string()))?,
            )),
            observed_cursor: Some(cursor),
            next_after_cycle_id,
        };
        let response_limit = self.max_message_bytes.min(MAX_SHELL_RESPONSE_BYTES);
        if response.encoded_len() > response_limit {
            return Err(Status::resource_exhausted(format!(
                "runtime shell page exceeds {response_limit} bytes; lower limit and retry"
            )));
        }
        Ok(Response::new(response))
    }

    async fn get_traceability(
        &self,
        request: Request<v1::GetTraceabilityRequest>,
    ) -> Result<Response<v1::TraceabilityPage>, Status> {
        let request = request.into_inner();
        let snapshot = self.runtime.watch_state().latest();
        let limit = page(
            request.limit,
            &request.after_traceability_id,
            request.cursor.as_ref(),
            &snapshot,
        )?;
        let requested_scope = request
            .scope
            .as_ref()
            .map(|scope| super::plots::record_scope(scope.clone()))
            .transpose()?;
        if requested_scope
            .as_ref()
            .and_then(|scope| scope.session_id.as_ref())
            .is_some_and(|session| session != &snapshot.state().session_id)
        {
            return Err(Status::failed_precondition(
                "traceability scope epoch mismatch; resynchronize",
            ));
        }

        let after = request.after_traceability_id.clone();
        let (snapshots, has_more, host_traceability_fingerprint) = self
            .runtime
            .traceability_snapshots_page_at(
                &snapshot,
                requested_scope.as_ref(),
                &after,
                limit,
                8_192,
                MAX_TRACEABILITY_RESPONSE_BYTES,
            )
            .await
            .map_err(|error| Status::resource_exhausted(error.to_string()))?;
        if !after.is_empty()
            && (request.host_traceability_fingerprint.is_empty()
                || request.host_traceability_fingerprint != host_traceability_fingerprint)
        {
            return Err(Status::aborted(
                "host traceability changed; restart pagination from the first page",
            ));
        }
        let next_after_traceability_id = if has_more {
            snapshots
                .last()
                .map(|item| item.traceability_id.to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };
        let snapshots = snapshots
            .iter()
            .map(super::history::traceability)
            .collect::<Result<Vec<_>, _>>()?;
        let response = v1::TraceabilityPage {
            scope: request.scope,
            snapshots,
            cursor: Some(cursor(&snapshot)),
            next_after_traceability_id,
            complete: !has_more,
            host_traceability_fingerprint,
        };
        if response.encoded_len() > self.max_message_bytes.min(MAX_TRACEABILITY_RESPONSE_BYTES) {
            return Err(Status::resource_exhausted(
                "traceability page exceeds response budget; lower limit and retry",
            ));
        }
        Ok(Response::new(response))
    }

    async fn watch_runtime(
        &self,
        request: Request<v1::WatchRuntimeRequest>,
    ) -> Result<Response<Self::WatchRuntimeStream>, Status> {
        if *self.stopping.borrow() {
            return Err(Status::unavailable("listener is stopping"));
        }
        let watch = self.runtime.watch_state();
        let initial = watch.latest();
        validate_cursor(request.get_ref().cursor.as_ref(), &initial, false)?;
        // No spawned producer or queue: Tokio watch retains only the latest
        // snapshot, while the stream produces one frame when transport polls it.
        let frames = stream::unfold(
            (watch, self.stopping.clone(), Some(initial)),
            |(mut watch, mut stopping, initial)| async move {
                if *stopping.borrow() {
                    return None;
                }
                let (snapshot, resynchronized) = if let Some(snapshot) = initial {
                    (snapshot, true)
                } else {
                    let snapshot = tokio::select! { biased;
                        _=stopping.changed()=>return None,
                        next=watch.next()=>next?,
                    };
                    (snapshot, false)
                };
                let frame = v1::RuntimeStateFrame {
                    cursor: Some(cursor(&snapshot)),
                    resynchronized,
                    shutting_down: snapshot.state().shutting_down,
                    cycle_count: snapshot.state().cycles.len() as u64,
                };
                Some((Ok(frame), (watch, stopping, None)))
            },
        );
        Ok(Response::new(Box::pin(frames)))
    }
}

/* === Public Functions === */
pub(super) fn cursor(snapshot: &RuntimeStateSnapshot) -> v1::Cursor {
    cursor_at(
        snapshot.state().process_epoch().to_string(),
        snapshot.revision(),
    )
}

pub(super) fn cursor_at(epoch: String, position: u64) -> v1::Cursor {
    v1::Cursor {
        epoch,
        stream: STREAM.into(),
        position,
    }
}

pub(super) fn validate_cursor(
    given: Option<&v1::Cursor>,
    snapshot: &RuntimeStateSnapshot,
    exact: bool,
) -> Result<(), Status> {
    validate_cursor_at(
        given,
        &snapshot.state().process_epoch().to_string(),
        snapshot.revision(),
        exact,
    )
}

pub(super) fn validate_cursor_at(
    given: Option<&v1::Cursor>,
    epoch: &str,
    revision: u64,
    exact: bool,
) -> Result<(), Status> {
    let current = Cursor {
        epoch: epoch.into(),
        stream: STREAM.into(),
        position: revision,
    };
    let given = given.map(position);
    current
        .validate(given.as_ref(), exact)
        .map_err(cursor_failure)
}

pub(super) fn page(
    limit: u32,
    after: &str,
    given: Option<&v1::Cursor>,
    snapshot: &RuntimeStateSnapshot,
) -> Result<usize, Status> {
    page_at(
        limit,
        after,
        given,
        &snapshot.state().process_epoch().to_string(),
        snapshot.revision(),
    )
}

pub(super) fn page_at(
    limit: u32,
    after: &str,
    given: Option<&v1::Cursor>,
    epoch: &str,
    revision: u64,
) -> Result<usize, Status> {
    let current = Cursor {
        epoch: epoch.into(),
        stream: STREAM.into(),
        position: revision,
    };
    let given = given.map(position);
    current
        .page(limit, after, given.as_ref())
        .map_err(cursor_failure)
}

/* === Private Functions === */
fn position(value: &v1::Cursor) -> Cursor {
    Cursor {
        epoch: value.epoch.clone(),
        stream: value.stream.clone(),
        position: value.position,
    }
}

fn cursor_failure(error: CursorError) -> Status {
    let code = match error {
        CursorError::Selection => tonic::Code::FailedPrecondition,
        CursorError::Ahead => tonic::Code::OutOfRange,
        CursorError::Changed => tonic::Code::Aborted,
        CursorError::AfterTooLarge => tonic::Code::ResourceExhausted,
        CursorError::Limit | CursorError::Missing => tonic::Code::InvalidArgument,
    };
    Status::new(code, error.to_string())
}

fn progress_frame(
    revision: u64,
    resynchronized: bool,
    value: Option<&dokime_app::capabilities::execution::progress::LiveExecutionProgress>,
) -> v1::ExecutionProgressFrame {
    v1::ExecutionProgressFrame {
        revision,
        resynchronized,
        progress: value.map(|value| v1::ExecutionProgress {
            test_execution_id: value.test_execution_id.to_string(),
            step_id: value.step.to_string(),
            phase: value.progress.phase.clone(),
            completed: value.progress.completed,
            total: value.progress.total,
            elapsed_ms: value.progress.elapsed_ms,
            remaining_ms: value.progress.remaining_ms,
        }),
        cleared: value.is_none(),
    }
}

fn after_cycle_position(
    state: &dokime_app::runtime::RuntimeState,
    after: &str,
) -> Result<usize, Status> {
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

fn evidence_counts(
    counts: &dokime_app::capabilities::evidence::EvidenceCounts,
) -> v1::EvidenceCounts {
    v1::EvidenceCounts {
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
