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
use std::collections::BTreeMap;
use std::sync::{Arc, MutexGuard, PoisonError};
use std::time::Duration;
// Third Party Imports

// Local Imports
use super::buffer::{
    BufferedFrame, Executions, LeaseOutcome, MAX_BUFFERED_ARTIFACT_BYTES, PROMPT_POLL_INTERVAL,
    PromptOutcome, lock,
};
use crate::artifacts::ArtifactSink;
use crate::core::{Identifier, PromptId, RecordScope, Value};
use crate::execution::lease::{LeaseHold, LeaseRelease};
use crate::execution::{ExecutionProgress, ProgressSink};
use crate::prompts::{PromptRequest, PromptResponse};
use crate::records::RecordPublishRequest;
use crate::telemetry::{LeaseGap, LeaseRequest, SampleSink, TelemetrySample};
use crate::transport::ArtifactChunk;
use crate::transport::{ExecutionTransport, Journal, TransportFuture};

/* === Constants === */
const ARTIFACT_BACKPRESSURE_POLL_INTERVAL: Duration = Duration::from_millis(250);
// Rows of one stream that publish together without waiting for the window.
const BATCH_ROWS: usize = 100;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct FrameTransport {
    executions: Executions,
    token: String,
    prompts: u64,
}

pub(crate) struct FrameLeaseRelease {
    executions: Executions,
    token: String,
}

pub(crate) struct FrameArtifactSink {
    executions: Executions,
    token: String,
}

pub(crate) struct ExecutionSampleSink<T: crate::execution::SidecarTemplate> {
    executions: Executions,
    token: String,
    source: String,
    host: Option<crate::execution::SampleOutput>,
    template: std::marker::PhantomData<fn() -> T>,
    publisher: Option<crate::execution::SamplePublisher>,
    // One lock over every buffered row and every flush, so one stream's rows
    // reach the host in order and never through two calls at once.
    pending: tokio::sync::Mutex<Pending>,
}

// Rows produced for the host, per stream, waiting for one batch publication.
#[derive(Default)]
struct Pending {
    streams: BTreeMap<Identifier, Buffered>,
}

#[derive(Default)]
struct Buffered {
    rows: Vec<crate::execution::StampedRow>,
    latest: i64,
}

pub(crate) struct FrameRecordSource {
    executions: Executions,
    token: String,
}

pub(crate) struct FrameJournal {
    executions: Executions,
    token: String,
}

pub(crate) struct FrameProgressSink {
    executions: Executions,
    token: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl FrameTransport {
    pub(crate) fn new(executions: Executions, token: String) -> Self {
        Self {
            executions,
            token,
            prompts: 0,
        }
    }

    fn push(&self, frame: BufferedFrame) {
        if let Some(state) = lock(&self.executions).get_mut(&self.token) {
            state.push(frame);
        }
    }
}

impl FrameArtifactSink {
    pub(crate) fn new(executions: Executions, token: String) -> Self {
        Self { executions, token }
    }
}

impl ArtifactSink for FrameArtifactSink {
    fn push(&self, chunk: ArtifactChunk) {
        let executions = lock(&self.executions);
        // Cloned out rather than borrowed: the wait below needs to hold this
        // across a call that temporarily releases the map's lock, and a
        // borrow into the map could not survive that.
        let Some(condvar) = executions
            .get(&self.token)
            .map(|state| state.artifact_condvar())
        else {
            // The execution was retired before this piece arrived. Nothing is
            // left to buffer it into or to wait for.
            return;
        };
        // A step writes from a runtime worker, and the pump that drains the
        // backlog may be queued behind it on that same worker; parking there
        // hands the worker's queue to the rest of the runtime first.
        let at_gate = executions
            .get(&self.token)
            .is_some_and(|state| state.artifact_backlog() >= MAX_BUFFERED_ARTIFACT_BYTES);
        let multi_threaded = tokio::runtime::Handle::try_current().is_ok_and(|runtime| {
            runtime.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread
        });
        let Some(mut executions) = (if at_gate && multi_threaded {
            tokio::task::block_in_place(|| park_at_gate(executions, &self.token, &condvar))
        } else {
            park_at_gate(executions, &self.token, &condvar)
        }) else {
            return;
        };
        if let Some(state) = executions.get_mut(&self.token) {
            state.push(BufferedFrame::artifact(chunk));
        }
    }
}

impl<T: crate::execution::SidecarTemplate> ExecutionSampleSink<T> {
    pub(crate) fn new(
        executions: Executions,
        token: String,
        source: String,
        host: Option<crate::execution::SampleOutput>,
        publisher: Option<crate::execution::SamplePublisher>,
    ) -> Self {
        Self {
            executions,
            token,
            source,
            host,
            template: std::marker::PhantomData,
            publisher,
            pending: tokio::sync::Mutex::new(Pending::default()),
        }
    }

    fn host_output(
        &self,
    ) -> Option<
        Result<
            (
                &harmos::sidecar::Supervisor,
                crate::execution::SamplePublisher,
            ),
            String,
        >,
    > {
        match self.host.as_ref()? {
            crate::execution::SampleOutput::Host(host) => Some(
                self.publisher
                    .map(|publish| (host, publish))
                    .ok_or_else(|| "native workflow has no owning plugin publisher".to_owned()),
            ),
            crate::execution::SampleOutput::Embedded(_) => None,
        }
    }
}

impl Pending {
    // Stamp the row with its production instant, never earlier than the
    // stream's previous row, and report how many rows the stream now holds.
    fn push(
        &mut self,
        stream: &Identifier,
        values: BTreeMap<String, Value>,
        storage_payload: Option<Vec<u8>>,
    ) -> usize {
        let buffered = self.streams.entry(stream.clone()).or_default();
        let stamp = unix_micros().max(buffered.latest);
        buffered.latest = stamp;
        buffered.rows.push(crate::execution::StampedRow {
            values,
            storage_payload,
            published_unix_micros: stamp,
        });
        buffered.rows.len()
    }

    fn take(&mut self, stream: &Identifier) -> Vec<crate::execution::StampedRow> {
        self.streams
            .get_mut(stream)
            .map(|buffered| std::mem::take(&mut buffered.rows))
            .unwrap_or_default()
    }

    fn is_empty(&self) -> bool {
        self.streams
            .values()
            .all(|buffered| buffered.rows.is_empty())
    }
}

impl FrameRecordSource {
    pub(crate) fn new(executions: Executions, token: String) -> Self {
        Self { executions, token }
    }
}

impl ProgressSink for FrameProgressSink {
    fn update(&self, progress: ExecutionProgress) {
        if let Some(state) = lock(&self.executions).get_mut(&self.token) {
            state.push(BufferedFrame::progress(progress));
        }
    }
}

impl crate::records::RecordSource for FrameRecordSource {
    fn next_record(&self, cursor: &mut usize) -> Option<crate::records::Record> {
        lock(&self.executions)
            .get(&self.token)
            .and_then(|state| state.next_record(cursor))
    }

    fn delivered(&self, index: usize) -> Option<crate::records::Record> {
        lock(&self.executions)
            .get(&self.token)
            .and_then(|state| state.delivered(index))
    }
}

impl<T: crate::execution::SidecarTemplate> SampleSink for ExecutionSampleSink<T> {
    fn publish<'a>(
        &'a self,
        stream: &'a Identifier,
        values: BTreeMap<String, Value>,
        storage_payload: Option<Vec<u8>>,
    ) -> crate::transport::TransportFuture<'a, Result<(), String>> {
        Box::pin(async move {
            if lock(&self.executions)
                .get(&self.token)
                .is_none_or(|state| !state.samples_open())
            {
                return Err("execution sample inputs are closed".into());
            }
            if let Some(output) = self.host_output() {
                let (host, publish) = output?;
                let mut pending = self.pending.lock().await;
                if pending.push(stream, values, storage_payload) < BATCH_ROWS {
                    return Ok(());
                }
                let rows = pending.take(stream);
                return publish(host, &self.token, stream, &rows).await;
            }
            if let Some(crate::execution::SampleOutput::Embedded(publisher)) = &self.host {
                return publisher.publish(
                    &self.token,
                    stream,
                    values,
                    storage_payload.ok_or("typed publication requires original row bytes")?,
                );
            }

            let mut executions = lock(&self.executions);
            let state = executions
                .get_mut(&self.token)
                .ok_or("execution evidence is closed")?;
            let mut sample = TelemetrySample::new(
                crate::model::identity::new_id(),
                stream.clone(),
                state.produced(stream.as_str()),
                crate::model::time::now_utc(),
                self.source.clone(),
            );
            sample.values = values;
            sample.storage_payload = storage_payload;
            state.local_sample(sample)?;
            Ok(())
        })
    }

    fn produced(&self, stream: &str) -> u64 {
        lock(&self.executions)
            .get(&self.token)
            .map(|state| state.produced(stream))
            .unwrap_or_default()
    }

    fn flush(&self) -> TransportFuture<'_, Result<(), (Identifier, String)>> {
        Box::pin(async move {
            let Some(Ok((host, publish))) = self.host_output() else {
                return Ok(());
            };
            let mut pending = self.pending.lock().await;
            let mut refused = None;
            let streams = pending.streams.keys().cloned().collect::<Vec<_>>();
            for stream in streams {
                let rows = pending.take(&stream);
                if rows.is_empty() {
                    continue;
                }
                if let Err(error) = publish(host, &self.token, &stream, &rows).await {
                    refused.get_or_insert((stream, error));
                }
            }
            refused.map_or(Ok(()), Err)
        })
    }

    fn buffering(&self) -> bool {
        self.pending
            .try_lock()
            .map_or(true, |pending| !pending.is_empty())
    }
}

impl ExecutionTransport for FrameTransport {
    fn execution(&self) -> &str {
        &self.token
    }

    fn progress(&self) -> Arc<dyn ProgressSink> {
        Arc::new(FrameProgressSink {
            executions: Arc::clone(&self.executions),
            token: self.token.clone(),
        })
    }

    fn prompt<'a>(
        &'a mut self,
        request: PromptRequest,
    ) -> TransportFuture<'a, crate::Result<PromptResponse>> {
        let ordinal = self.prompts;
        self.prompts = self.prompts.saturating_add(1);
        let prompt_id = PromptId::from_seed(format!("{}:prompt:{ordinal}", self.token));
        let executions = Arc::clone(&self.executions);
        let token = self.token.clone();

        Box::pin(async move {
            let mut request = request;
            // Identity, scope, and status belong to the two ends that own them:
            // the transport derives the id, the host owns the scope, and a
            // question that has just been asked is pending by definition.
            request.prompt_id = prompt_id.clone();
            request.scope = RecordScope::default();
            let encoded = serde_json::to_string(&request).map_err(|error| {
                crate::Error::PromptRefused(format!(
                    "the operator question could not be encoded: {error}"
                ))
            })?;

            {
                let mut executions = lock(&executions);
                let Some(state) = executions.get_mut(&token) else {
                    return Err(crate::Error::PromptCancelled);
                };
                if state.cancelled() {
                    return Err(crate::Error::PromptCancelled);
                }
                state.raise(prompt_id.clone(), encoded);
            }

            loop {
                tokio::time::sleep(PROMPT_POLL_INTERVAL).await;
                let settled = {
                    let executions = lock(&executions);
                    match executions.get(&token) {
                        None => Some(Err(crate::Error::PromptCancelled)),
                        Some(state) => match state.settled(&prompt_id) {
                            Some(PromptOutcome::Answered(response)) => {
                                Some(Ok((**response).clone()))
                            }
                            Some(PromptOutcome::Cancelled) => {
                                Some(Err(crate::Error::PromptCancelled))
                            }
                            None => None,
                        },
                    }
                };
                if let Some(answer) = settled {
                    return answer;
                }
            }
        })
    }

    fn lease<'a>(
        &'a mut self,
        request: LeaseRequest,
    ) -> TransportFuture<'a, crate::Result<LeaseHold>> {
        let executions = Arc::clone(&self.executions);
        let token = self.token.clone();

        Box::pin(async move {
            let lease_token = request.lease_token.clone();
            {
                let mut live = lock(&executions);
                let Some(state) = live.get_mut(&token) else {
                    return Err(crate::Error::LeaseCancelled);
                };
                if state.cancelled() {
                    return Err(crate::Error::LeaseCancelled);
                }
                state.request_lease(&request);
            }

            // The prompt's wait, for the prompt's reason: only the host's
            // answer or the run's end may settle this, because a step that
            // guessed it had the hold would read somebody else's window as its
            // own evidence.
            loop {
                tokio::time::sleep(PROMPT_POLL_INTERVAL).await;
                let settled = {
                    let live = lock(&executions);
                    match live.get(&token) {
                        None => Some(Err(crate::Error::LeaseCancelled)),
                        Some(state) => match state.lease_settled(&lease_token) {
                            Some(LeaseOutcome::Granted) => Some(Ok(())),
                            Some(LeaseOutcome::Refused(reason)) => {
                                Some(Err(crate::Error::LeaseRefused(reason.clone())))
                            }
                            Some(LeaseOutcome::Cancelled) => {
                                Some(Err(crate::Error::LeaseCancelled))
                            }
                            Some(LeaseOutcome::Pending) | None => None,
                        },
                    }
                };
                match settled {
                    Some(Ok(())) => {
                        return Ok(LeaseHold {
                            lease_token,
                            release: Arc::new(FrameLeaseRelease {
                                executions: Arc::clone(&executions),
                                token: token.clone(),
                            }) as Arc<dyn LeaseRelease>,
                        });
                    }
                    Some(Err(error)) => return Err(error),
                    None => continue,
                }
            }
        })
    }

    fn gaps(&self) -> Vec<LeaseGap> {
        lock(&self.executions)
            .get(&self.token)
            .map(|state| state.all_gaps())
            .unwrap_or_default()
    }

    fn emit(&mut self, mut record: RecordPublishRequest) {
        record
            .timestamp
            .get_or_insert_with(crate::model::time::now_utc);
        if let Some(state) = lock(&self.executions).get_mut(&self.token) {
            state.publish_record(record);
        }
    }

    fn log(&self, level: crate::execution::log::Level, message: String) {
        self.push(BufferedFrame::log(level, message));
    }

    fn journal(&self) -> Arc<dyn Journal> {
        Arc::new(FrameJournal {
            executions: Arc::clone(&self.executions),
            token: self.token.clone(),
        })
    }

    fn traceability(&mut self, sections: Vec<crate::traceability::TraceabilitySection>) {
        for section in sections {
            match serde_json::to_string(&section) {
                Ok(encoded) => self.push(BufferedFrame::traceability(encoded)),
                Err(error) => self.push(BufferedFrame::log(
                    crate::execution::log::Level::Error,
                    format!(
                        "traceability section `{}` could not be encoded: {error}",
                        section.identifier
                    ),
                )),
            }
        }
    }

    fn finish_step(&mut self, attempt: crate::execution::TestStepAttempt) {
        match serde_json::to_string(&attempt) {
            Ok(encoded) => self.push(BufferedFrame::step_attempt(encoded)),
            Err(error) => self.push(BufferedFrame::log(
                crate::execution::log::Level::Error,
                format!("step attempt could not be encoded: {error}"),
            )),
        }
    }

    fn capture_phase(&mut self, phase: crate::core::CapturePhase) {
        self.push(BufferedFrame::capture_phase(
            serde_json::to_string(&phase).expect("capture phase serializes"),
        ));
    }

    fn begin_step<'a>(
        &'a mut self,
        step: Identifier,
        start: crate::execution::StepAttemptStart,
    ) -> TransportFuture<'a, Result<(), crate::diagnostics::Diagnostic>> {
        let executions = Arc::clone(&self.executions);
        let token = self.token.clone();
        Box::pin(async move {
            let encoded = serde_json::to_string(&start)
                .map_err(|error| crate::diagnostics::Diagnostic::of(error.to_string()))?;
            {
                let mut live = lock(&executions);
                let state = live.get_mut(&token).ok_or_else(|| {
                    crate::diagnostics::Diagnostic::of("execution ended before step admission")
                })?;
                state.request_step(step.as_str(), &start, encoded);
            }
            loop {
                let answer = {
                    let live = lock(&executions);
                    let state = live.get(&token).ok_or_else(|| {
                        crate::diagnostics::Diagnostic::of(
                            "execution ended before step acknowledgement",
                        )
                    })?;
                    state.step_answer()
                };
                if let Some(answer) = answer {
                    return answer.map_err(crate::diagnostics::Diagnostic::of);
                }
                tokio::time::sleep(PROMPT_POLL_INTERVAL).await;
            }
        })
    }

    fn document(&self, addition: crate::document::DocumentAddition) {
        match serde_json::to_string(&addition) {
            Ok(encoded) => self.push(BufferedFrame::document(encoded)),
            Err(error) => self.push(BufferedFrame::log(
                crate::execution::log::Level::Error,
                format!("a document addition could not be encoded: {error}"),
            )),
        }
    }
}

impl Journal for FrameJournal {
    fn log(&self, level: crate::execution::log::Level, message: String) {
        if let Some(state) = lock(&self.executions).get_mut(&self.token) {
            state.push(BufferedFrame::log(level, message));
        }
    }

    fn record(&self, mut record: RecordPublishRequest) {
        record
            .timestamp
            .get_or_insert_with(crate::model::time::now_utc);
        if let Some(state) = lock(&self.executions).get_mut(&self.token) {
            state.publish_record(record);
        }
    }
}

impl LeaseRelease for FrameLeaseRelease {
    fn release(&self, lease_token: &str) {
        if let Some(state) = lock(&self.executions).get_mut(&self.token) {
            state.release_lease(lease_token);
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
// The publication clock Harmos stamps rows with: system time in Unix
// microseconds, negative before the epoch.
fn unix_micros() -> i64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => i64::try_from(elapsed.as_micros()).unwrap_or(i64::MAX),
        Err(error) => {
            i64::try_from(error.duration().as_micros()).map_or(i64::MIN, |micros| -micros)
        }
    }
}

fn park_at_gate<'a>(
    mut executions: MutexGuard<'a, BTreeMap<String, crate::transport::buffer::ExecutionState>>,
    token: &str,
    condvar: &std::sync::Condvar,
) -> Option<MutexGuard<'a, BTreeMap<String, crate::transport::buffer::ExecutionState>>> {
    loop {
        let state = executions.get(token)?;
        if state.cancelled() || state.artifact_backlog() < MAX_BUFFERED_ARTIFACT_BYTES {
            return Some(executions);
        }
        executions = condvar
            .wait_timeout(executions, ARTIFACT_BACKPRESSURE_POLL_INTERVAL)
            .unwrap_or_else(PoisonError::into_inner)
            .0;
    }
}

/* === Tests === */
