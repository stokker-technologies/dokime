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

// Third Party Imports

// Local Imports
use super::buffer::{BufferedFrame, Executions, LeaseOutcome, PromptOutcome, lock, settled};
use crate::artifacts::ArtifactSink;
use crate::execution::lease::{LeaseHold, LeaseRelease};
use crate::execution::{ExecutionProgress, ProgressSink};
use crate::prompts::{PromptRequest, PromptResponse};
use crate::telemetry::{LeaseGap, LeaseRequest, SampleSink, TelemetrySample};
use crate::transport::ArtifactChunk;
use crate::transport::{ExecutionTransport, Journal, TransportFuture};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::PromptId;
use dokime_domain::model::records::publication::RecordPublishRequest;
use dokime_domain::model::values::Value;

/* === Constants === */

// Rows of one stream that publish together without waiting for the window.
const HOST_BATCH_ROWS: usize = 100;

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
    rows: HostRows,
}

// Rows bound for the host. Producers keep buffering while one host call is in
// flight, so the next call carries everything produced meanwhile, up to one
// batch; a producer that fills a batch waits for that call, bounding memory.
struct HostRows {
    batch_rows: usize,
    // Held only to add or take rows, never across a host call.
    pending: std::sync::Mutex<Pending>,
    // Held across every host call, so one stream's rows reach the host in
    // order and never through two calls at once.
    sending: tokio::sync::Mutex<()>,
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
    changed: Arc<tokio::sync::Notify>,
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
        Self { executions, token, prompts: 0 }
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
        let Some(condvar) = executions.get(&self.token).map(|state| state.artifact_condvar()) else {
            // The execution was retired before this piece arrived. Nothing is
            // left to buffer it into or to wait for.
            return;
        };
        // A step writes from a runtime worker, and the pump that drains the
        // backlog may be queued behind it on that same worker; parking there
        // hands the worker's queue to the rest of the runtime first.
        let at_gate = executions.get(&self.token).is_some_and(|state| state.artifact_backlog() >= 16 * 1024 * 1024);
        let multi_threaded = tokio::runtime::Handle::try_current()
            .is_ok_and(|runtime| runtime.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread);
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
        executions: Executions, token: String, source: String, host: Option<crate::execution::SampleOutput>,
        publisher: Option<crate::execution::SamplePublisher>,
    ) -> Self {
        Self {
            executions,
            token,
            source,
            host,
            template: std::marker::PhantomData,
            publisher,
            rows: HostRows::new(HOST_BATCH_ROWS),
        }
    }

    fn host_output(&self) -> Option<Result<(&harmos::sidecar::Supervisor, crate::execution::SamplePublisher), String>> {
        match self.host.as_ref()? {
            crate::execution::SampleOutput::Host(host) => Some(
                self.publisher.map(|publish| (host, publish)).ok_or_else(|| "native workflow has no owning plugin publisher".to_owned()),
            ),
            crate::execution::SampleOutput::Embedded(_) => None,
        }
    }
}

impl Pending {
    // Stamp the row with its production instant, never earlier than the
    // stream's previous row, and report how many rows the stream now holds.
    fn push(&mut self, stream: &Identifier, values: BTreeMap<String, Value>, storage_payload: Option<Vec<u8>>) -> usize {
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

    // The stream's oldest rows, at most `limit` of them, in production order.
    fn take(&mut self, stream: &Identifier, limit: usize) -> Vec<crate::execution::StampedRow> {
        let Some(buffered) = self.streams.get_mut(stream) else {
            return Vec::new();
        };
        let count = limit.min(buffered.rows.len());
        buffered.rows.drain(..count).collect()
    }

    fn len(&self, stream: &Identifier) -> usize {
        self.streams.get(stream).map_or(0, |buffered| buffered.rows.len())
    }

    // How many rows each stream holds now; a flush publishes exactly these.
    fn due(&self) -> Vec<(Identifier, usize)> {
        let counts = self.streams.iter().map(|(stream, buffered)| (stream.clone(), buffered.rows.len()));
        counts.filter(|(_, rows)| *rows > 0).collect()
    }

    fn is_empty(&self) -> bool {
        self.streams.values().all(|buffered| buffered.rows.is_empty())
    }
}

impl HostRows {
    fn new(batch_rows: usize) -> Self {
        Self {
            batch_rows,
            pending: Default::default(),
            sending: Default::default(),
        }
    }

    fn pending(&self) -> MutexGuard<'_, Pending> {
        self.pending.lock().unwrap_or_else(PoisonError::into_inner)
    }

    // Buffer one row. A producer that fills a batch waits for the host call
    // in flight, then publishes the batch itself unless a flush already did.
    async fn publish<'s>(
        &self, stream: &Identifier, values: BTreeMap<String, Value>, storage_payload: Option<Vec<u8>>,
        send: impl Fn(Identifier, Vec<crate::execution::StampedRow>) -> TransportFuture<'s, Result<(), String>>,
    ) -> Result<(), String> {
        if self.pending().push(stream, values, storage_payload) < self.batch_rows {
            return Ok(());
        }
        let _sending = self.sending.lock().await;
        if self.pending().len(stream) < self.batch_rows {
            return Ok(());
        }
        let rows = self.pending().take(stream, self.batch_rows);
        send(stream.clone(), rows).await
    }

    // Publish every row buffered when the flush starts, in batches, and
    // report the first refusal; later rows wait for the next flush.
    async fn flush<'s>(
        &self, send: impl Fn(Identifier, Vec<crate::execution::StampedRow>) -> TransportFuture<'s, Result<(), String>>,
    ) -> Result<(), (Identifier, String)> {
        let _sending = self.sending.lock().await;
        let due = self.pending().due();
        let batches_of = |(stream, rows): (Identifier, usize)| std::iter::repeat_n(stream, rows.div_ceil(self.batch_rows));
        let batches = due.into_iter().flat_map(batches_of);
        let mut refused = None;
        for stream in batches {
            let batch = self.pending().take(&stream, self.batch_rows);
            let Err(error) = send(stream.clone(), batch).await else {
                continue;
            };
            refused.get_or_insert((stream, error));
        }
        refused.map_or(Ok(()), Err)
    }

    fn buffering(&self) -> bool {
        !self.pending().is_empty()
    }
}

impl FrameRecordSource {
    pub(crate) fn new(executions: Executions, token: String) -> Self {
        let changed = lock(&executions).get(&token).map(|state| Arc::clone(&state.settled)).unwrap_or_default();
        Self { executions, token, changed }
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
    fn changed(&self) -> Arc<tokio::sync::Notify> {
        Arc::clone(&self.changed)
    }

    fn closed(&self) -> bool {
        lock(&self.executions)
            .get(&self.token)
            .is_none_or(|state| !Arc::ptr_eq(&state.settled, &self.changed) || state.record_inputs_closed())
    }

    fn next_record(&self, cursor: &mut usize) -> Option<dokime_domain::model::records::evidence::Record> {
        lock(&self.executions)
            .get(&self.token)
            .filter(|state| Arc::ptr_eq(&state.settled, &self.changed))
            .and_then(|state| state.next_record(cursor))
    }

    fn delivered(&self, index: usize) -> Option<dokime_domain::model::records::evidence::Record> {
        lock(&self.executions)
            .get(&self.token)
            .filter(|state| Arc::ptr_eq(&state.settled, &self.changed))
            .and_then(|state| state.delivered(index))
    }
}

impl<T: crate::execution::SidecarTemplate> SampleSink for ExecutionSampleSink<T> {
    fn publish<'a>(
        &'a self, stream: &'a Identifier, values: BTreeMap<String, Value>, storage_payload: Option<Vec<u8>>,
    ) -> crate::transport::TransportFuture<'a, Result<(), String>> {
        Box::pin(async move {
            if lock(&self.executions).get(&self.token).is_none_or(|state| !state.samples_open()) {
                return Err("execution sample inputs are closed".into());
            }
            if let Some(output) = self.host_output() {
                let (host, publish) = output?;
                return self.rows.publish(stream, values, storage_payload, host_send(host, &self.token, publish)).await;
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
            let state = executions.get_mut(&self.token).ok_or("execution evidence is closed")?;
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
        lock(&self.executions).get(&self.token).map(|state| state.produced(stream)).unwrap_or_default()
    }

    fn flush(&self) -> TransportFuture<'_, Result<(), (Identifier, String)>> {
        Box::pin(async move {
            let Some(Ok((host, publish))) = self.host_output() else {
                return Ok(());
            };
            self.rows.flush(host_send(host, &self.token, publish)).await
        })
    }

    fn buffering(&self) -> bool {
        self.rows.buffering()
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

    fn prompt<'a>(&'a mut self, request: PromptRequest) -> TransportFuture<'a, crate::Result<PromptResponse>> {
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
            let encoded = serde_json::to_string(&request)
                .map_err(|error| crate::Error::PromptRefused(format!("the operator question could not be encoded: {error}")))?;

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

            settled(
                &executions,
                &token,
                || Err(crate::Error::PromptCancelled),
                |state| match state.settled(&prompt_id) {
                    Some(PromptOutcome::Answered(response)) => Some(Ok((**response).clone())),
                    Some(PromptOutcome::Cancelled) => Some(Err(crate::Error::PromptCancelled)),
                    None => None,
                },
            )
            .await
        })
    }

    fn lease<'a>(&'a mut self, request: LeaseRequest) -> TransportFuture<'a, crate::Result<LeaseHold>> {
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
            settled(
                &executions,
                &token,
                || Err(crate::Error::LeaseCancelled),
                |state| match state.lease_settled(&lease_token) {
                    Some(LeaseOutcome::Granted) => Some(Ok(())),
                    Some(LeaseOutcome::Refused(reason)) => Some(Err(crate::Error::LeaseRefused(reason.clone()))),
                    Some(LeaseOutcome::Cancelled) => Some(Err(crate::Error::LeaseCancelled)),
                    Some(LeaseOutcome::Pending) | None => None,
                },
            )
            .await?;
            let release = Arc::new(FrameLeaseRelease {
                executions: Arc::clone(&executions),
                token: token.clone(),
            });
            Ok(LeaseHold {
                lease_token,
                release: release as Arc<dyn LeaseRelease>,
            })
        })
    }

    fn gaps(&self) -> Vec<LeaseGap> {
        lock(&self.executions).get(&self.token).map(|state| state.all_gaps()).unwrap_or_default()
    }

    fn gap_counts(&self) -> std::collections::BTreeMap<String, u64> {
        lock(&self.executions).get(&self.token).map(|state| state.gap_counts().clone()).unwrap_or_default()
    }

    fn emit(&mut self, mut record: RecordPublishRequest) {
        record.timestamp.get_or_insert_with(crate::model::time::now_utc);
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

    fn traceability(&self, sections: Vec<dokime_domain::model::traceability::TraceabilitySection>) {
        for section in sections {
            match serde_json::to_string(&section) {
                Ok(encoded) => self.push(BufferedFrame::traceability(encoded)),
                Err(error) => self.push(BufferedFrame::log(
                    crate::execution::log::Level::Error,
                    format!("traceability section `{}` could not be encoded: {error}", section.identifier),
                )),
            }
        }
    }

    fn finish_step(&mut self, attempt: dokime_domain::model::execution::result::TestStepAttempt) {
        match serde_json::to_string(&attempt) {
            Ok(encoded) => self.push(BufferedFrame::step_attempt(encoded)),
            Err(error) => {
                self.push(BufferedFrame::log(crate::execution::log::Level::Error, format!("step attempt could not be encoded: {error}")))
            }
        }
    }

    fn capture_phase(&mut self, phase: dokime_domain::model::execution::CapturePhase) {
        self.push(BufferedFrame::capture_phase(serde_json::to_string(&phase).expect("capture phase serializes")));
    }

    fn begin_step<'a>(
        &'a mut self, step: Identifier, start: dokime_domain::model::execution::result::StepAttemptStart,
    ) -> TransportFuture<'a, Result<(), crate::diagnostics::Diagnostic>> {
        let executions = Arc::clone(&self.executions);
        let token = self.token.clone();
        Box::pin(async move {
            let encoded = serde_json::to_string(&start).map_err(|error| crate::diagnostics::Diagnostic::of(error.to_string()))?;
            {
                let mut live = lock(&executions);
                let state =
                    live.get_mut(&token).ok_or_else(|| crate::diagnostics::Diagnostic::of("execution ended before step admission"))?;
                if state.cancelled() {
                    return Err(crate::diagnostics::Diagnostic::of("execution cancelled before step admission"));
                }
                state.request_step(step.as_str(), &start, encoded);
            }
            settled(
                &executions,
                &token,
                || Err(crate::diagnostics::Diagnostic::of("execution ended before step acknowledgement")),
                |state| state.step_answer().map(|answer| answer.map_err(crate::diagnostics::Diagnostic::of)),
            )
            .await
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
        record.timestamp.get_or_insert_with(crate::model::time::now_utc);
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

// One host call publishing a batch of one stream's rows.
fn host_send<'s>(
    host: &'s harmos::sidecar::Supervisor, token: &'s str, publish: crate::execution::SamplePublisher,
) -> impl Fn(Identifier, Vec<crate::execution::StampedRow>) -> TransportFuture<'s, Result<(), String>> {
    move |stream, rows| Box::pin(async move { publish(host, token, &stream, &rows).await })
}

// The publication clock Harmos stamps rows with: system time in Unix
// microseconds, negative before the epoch.
fn unix_micros() -> i64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => i64::try_from(elapsed.as_micros()).unwrap_or(i64::MAX),
        Err(error) => i64::try_from(error.duration().as_micros()).map_or(i64::MIN, |micros| -micros),
    }
}

fn park_at_gate<'a>(
    mut executions: MutexGuard<'a, BTreeMap<String, crate::transport::buffer::ExecutionState>>, token: &str, condvar: &std::sync::Condvar,
) -> Option<MutexGuard<'a, BTreeMap<String, crate::transport::buffer::ExecutionState>>> {
    loop {
        let state = executions.get(token)?;
        if !std::ptr::eq(state.artifact_condvar().as_ref(), condvar) {
            return None;
        }
        if state.cancelled() || state.artifact_backlog() < 16 * 1024 * 1024 {
            return Some(executions);
        }
        executions = condvar.wait(executions).unwrap_or_else(PoisonError::into_inner);
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::buffer::ExecutionState;

    #[tokio::test(start_paused = true)]
    async fn prompt_waits_wake_on_settlement() {
        let request = crate::transport::protocol::RunExecution {
            execution_token: "fixture".into(),
            test_cycle_id: crate::model::identity::new_id::<dokime_domain::model::identity::TestCycleId>().to_string(),
            test_execution_id: crate::model::identity::new_id::<dokime_domain::model::identity::TestExecutionId>().to_string(),
            template_identifier: "fixture".into(),
            workflow: "fixture".into(),
            parameters_json: "{}".into(),
            secrets_json: "{}".into(),
            mode: "interactive".into(),
        };
        let (cancel, _) = tokio::sync::watch::channel(false);
        let state = ExecutionState::new(request.clone(), cancel);
        let executions: Executions = Arc::new(std::sync::Mutex::new(BTreeMap::from([("fixture".to_owned(), state)])));
        let mut transport = FrameTransport {
            executions: Arc::clone(&executions),
            token: "fixture".into(),
            prompts: 0,
        };
        let question = || PromptRequest {
            prompt_id: PromptId::from_seed("unused"),
            prompt_type: Identifier::new("fixture.confirm").unwrap(),
            values: BTreeMap::new(),
            scope: RecordScope::default(),
        };

        // An Answer Wakes The Waiter Without Any Clock Advance
        let waiting = tokio::spawn(async move {
            let answer = transport.prompt(question()).await;
            (transport, answer)
        });
        let asked = PromptId::from_seed("fixture:prompt:0");
        raised(&executions, 1).await;
        let before = tokio::time::Instant::now();
        let response = PromptResponse {
            prompt_id: asked.clone(),
            action: "confirm".into(),
            responder: "operator".into(),
            submitted_at: crate::model::time::now_utc(),
            values: BTreeMap::new(),
            accepted: true,
        };
        lock(&executions).get_mut("fixture").unwrap().answer(asked.as_str(), response.clone());
        let (mut transport, answer) = waiting.await.unwrap();
        assert_eq!(answer.unwrap(), response);
        assert_eq!(tokio::time::Instant::now(), before);

        // Evicting The Run Cancels A Waiting Prompt At Once
        let waiting = tokio::spawn(async move { transport.prompt(question()).await });
        raised(&executions, 2).await;
        lock(&executions).remove("fixture");
        assert!(matches!(waiting.await.unwrap(), Err(crate::Error::PromptCancelled)));
        assert_eq!(tokio::time::Instant::now(), before);

        // Cancellation Settles Both A Prompt And Step Admission Without A Timer.
        let (cancel, _) = tokio::sync::watch::channel(false);
        lock(&executions).insert("fixture".into(), ExecutionState::new(request, cancel));
        let mut transport = FrameTransport {
            executions: Arc::clone(&executions),
            token: "fixture".into(),
            prompts: 0,
        };
        let waiting = tokio::spawn(async move { transport.prompt(question()).await });
        raised(&executions, 1).await;
        let step = dokime_domain::model::execution::result::StepAttemptStart {
            attempt_id: dokime_domain::model::identity::StepAttemptId::from_seed("fixture.step"),
            step_id: dokime_domain::model::identity::StepId::from_seed("fixture.step"),
            started_at: crate::model::time::now_utc(),
        };
        {
            let mut live = lock(&executions);
            let state = live.get_mut("fixture").unwrap();
            state.request_step("fixture.step", &step, String::new());
            state.cancel();
            assert!(state.step_answer().unwrap().is_err());
        }
        assert!(matches!(waiting.await.unwrap(), Err(crate::Error::PromptCancelled)));
        assert_eq!(tokio::time::Instant::now(), before);
    }

    // A Host That Records Every Call, Fails On Request And Can Hold Calls Open.
    struct Host {
        calls: std::sync::Mutex<Vec<Vec<String>>>,
        in_flight: std::sync::atomic::AtomicUsize,
        refusing: std::sync::atomic::AtomicBool,
        gated: bool,
        gate: tokio::sync::Semaphore,
    }

    impl Host {
        fn new(gated: bool) -> Arc<Self> {
            Arc::new(Self {
                calls: Default::default(),
                in_flight: Default::default(),
                refusing: Default::default(),
                gated,
                gate: tokio::sync::Semaphore::new(0),
            })
        }

        async fn receive(&self, rows: Vec<crate::execution::StampedRow>) -> Result<(), String> {
            use std::sync::atomic::Ordering;
            assert_eq!(self.in_flight.fetch_add(1, Ordering::AcqRel), 0, "host calls never overlap");
            let names = rows.iter().map(|row| String::from_utf8(row.storage_payload.clone().unwrap()).unwrap()).collect();
            self.calls.lock().unwrap().push(names);
            if self.gated {
                self.gate.acquire().await.unwrap().forget();
            } else {
                tokio::task::yield_now().await;
            }
            self.in_flight.fetch_sub(1, Ordering::AcqRel);
            if self.refusing.load(Ordering::Acquire) { Err("host refused the batch".into()) } else { Ok(()) }
        }

        fn calls(&self) -> Vec<Vec<String>> {
            self.calls.lock().unwrap().clone()
        }
    }

    fn sender(host: &Arc<Host>) -> impl Fn(Identifier, Vec<crate::execution::StampedRow>) -> TransportFuture<'static, Result<(), String>> {
        let host = Arc::clone(host);
        move |_, rows| {
            let host = Arc::clone(&host);
            Box::pin(async move { host.receive(rows).await })
        }
    }

    async fn produce(rows: &HostRows, host: &Arc<Host>, name: &str) -> Result<(), String> {
        let stream = Identifier::new("fixture.samples").unwrap();
        rows.publish(&stream, BTreeMap::new(), Some(name.as_bytes().to_vec()), sender(host)).await
    }

    // Yields Until The Host Has Received `count` Calls.
    async fn called(host: &Host, count: usize) {
        while host.calls.lock().unwrap().len() < count {
            tokio::task::yield_now().await;
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_producers_keep_stream_order_without_loss() {
        let rows = Arc::new(HostRows::new(16));
        let host = Host::new(false);
        let producers = (0..4)
            .map(|producer| {
                let (rows, host) = (Arc::clone(&rows), Arc::clone(&host));
                tokio::spawn(async move {
                    for sequence in 0..500 {
                        produce(&rows, &host, &format!("{producer}:{sequence}")).await.unwrap();
                    }
                })
            })
            .collect::<Vec<_>>();
        let flusher = {
            let (rows, host) = (Arc::clone(&rows), Arc::clone(&host));
            tokio::spawn(async move {
                for _ in 0..200 {
                    rows.flush(sender(&host)).await.unwrap();
                    tokio::task::yield_now().await;
                }
            })
        };
        for producer in producers {
            producer.await.unwrap();
        }
        flusher.await.unwrap();
        rows.flush(sender(&host)).await.unwrap();
        assert!(!rows.buffering());

        let calls = host.calls();
        assert!(calls.iter().all(|call| !call.is_empty() && call.len() <= 16), "every call carries one bounded batch");
        let delivered = calls.into_iter().flatten().collect::<Vec<_>>();
        assert_eq!(delivered.len(), 2_000);
        for producer in 0..4 {
            let sequences = delivered
                .iter()
                .filter_map(|name| name.strip_prefix(&format!("{producer}:")))
                .map(|sequence| sequence.parse::<u32>().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(sequences, (0..500).collect::<Vec<_>>(), "producer {producer} rows arrive once, in order");
        }
    }

    #[tokio::test]
    async fn rows_coalesce_behind_a_slow_host_up_to_one_bounded_batch() {
        let rows = Arc::new(HostRows::new(4));
        let host = Host::new(true);
        produce(&rows, &host, "0").await.unwrap();
        let flush = {
            let (rows, host) = (Arc::clone(&rows), Arc::clone(&host));
            tokio::spawn(async move { rows.flush(sender(&host)).await })
        };
        called(&host, 1).await;

        // Producers Keep Buffering While The Host Call Is In Flight.
        for name in ["1", "2", "3"] {
            produce(&rows, &host, name).await.unwrap();
        }
        // The Producer That Fills The Batch Waits, So The Buffer Stays Bounded.
        let filling = {
            let (rows, host) = (Arc::clone(&rows), Arc::clone(&host));
            tokio::spawn(async move { produce(&rows, &host, "4").await })
        };
        for _ in 0..16 {
            tokio::task::yield_now().await;
        }
        assert!(!filling.is_finished());
        assert_eq!(rows.pending().len(&Identifier::new("fixture.samples").unwrap()), 4);

        host.gate.add_permits(2);
        flush.await.unwrap().unwrap();
        filling.await.unwrap().unwrap();
        assert_eq!(host.calls(), [vec!["0"], vec!["1", "2", "3", "4"]]);
        assert!(!rows.buffering());
    }

    #[tokio::test]
    async fn host_refusals_surface_and_later_rows_still_publish() {
        use std::sync::atomic::Ordering;
        let rows = HostRows::new(2);
        let host = Host::new(false);
        host.refusing.store(true, Ordering::Release);
        produce(&rows, &host, "a").await.unwrap();
        assert_eq!(produce(&rows, &host, "b").await, Err("host refused the batch".to_owned()));

        host.refusing.store(false, Ordering::Release);
        produce(&rows, &host, "c").await.unwrap();
        rows.flush(sender(&host)).await.unwrap();

        host.refusing.store(true, Ordering::Release);
        produce(&rows, &host, "d").await.unwrap();
        let (stream, error) = rows.flush(sender(&host)).await.unwrap_err();
        assert_eq!((stream.as_str(), error.as_str()), ("fixture.samples", "host refused the batch"));
        assert_eq!(host.calls(), [vec!["a", "b"], vec!["c"], vec!["d"]]);
    }

    // Yields Until The Spawned Prompts Have Raised `count` Request Frames.
    async fn raised(executions: &Executions, count: usize) {
        while lock(executions).get_mut("fixture").unwrap().drain_after(0).unwrap().prompt_requests.len() < count {
            tokio::task::yield_now().await;
        }
    }
}
