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
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Duration;
// Third Party Imports
// Third-Party Imports
use harmos::plugin::sidecar::{Frame, Refusal};
use tokio::sync::watch;
use tokio::task::JoinHandle;
// Local Imports
use crate::core::PromptId;
use crate::prompts::PromptResponse;
use crate::transport::{
    ArtifactChunk, EvidenceBatch, RunExecutionRequest, SidecarExecutionOutcome,
};

/* === Constants === */
pub(crate) const MAX_BUFFERED_FRAMES: usize = 1024;

const MAX_LOCAL_SAMPLES: usize = 1024;

pub(crate) const MAX_BUFFERED_ARTIFACT_BYTES: u64 = 16 * 1024 * 1024;

pub(crate) const PROMPT_POLL_INTERVAL: Duration = Duration::from_millis(10);

const MAX_DELIVERED_RECORDS: usize = 256;

/* === Macros === */

/* === Type Aliases === */
pub(crate) type Executions = Arc<Mutex<BTreeMap<String, ExecutionState>>>;

pub(crate) type Tasks = Arc<Mutex<Vec<JoinHandle<()>>>>;

/* === Structs === */
pub(crate) struct ExecutionState {
    pub(crate) local_inputs: bool,
    local_scope: crate::core::RecordScope,
    local_admitted: u64,
    pub(crate) callbacks: crate::execution::Callbacks,
    pub(crate) listening: tokio::sync::broadcast::Sender<crate::execution::PublishedRow>,
    pub(crate) request: RunExecutionRequest,
    pub(crate) controls: Arc<super::control::Controls>,
    pub(crate) cancel: watch::Sender<bool>,
    pub(crate) changed: Arc<tokio::sync::Notify>,
    frames: VecDeque<BufferedFrame>,
    next_cursor: u64,
    oldest_cursor: u64,
    buffered_artifact_bytes: u64,
    acknowledged_cursor: u64,
    artifact_condvar: Arc<Condvar>,
    phase: String,
    step_admission: Option<(crate::core::StepAttemptId, Option<Result<(), String>>)>,
    terminal: bool,
    prompt: Option<PromptExchange>,
    stream_rows: Vec<crate::telemetry::TelemetrySample>,
    sample_positions: BTreeMap<crate::core::TelemetrySampleId, usize>,
    samples_closed: bool,
    local_samples: Vec<crate::telemetry::TelemetrySample>,
    produced: BTreeMap<String, u64>,
    delivered: Vec<crate::records::Record>,
    subscriptions: Vec<crate::core::Identifier>,
    captured_records: Vec<crate::records::Record>,
    subscriptions_settling: bool,
    subscriptions_settled: bool,
    dropped: usize,
    gaps: Vec<crate::telemetry::LeaseGap>,
    leases: BTreeMap<String, LeaseOutcome>,
}

#[derive(Default)]
pub(crate) struct BufferedFrame {
    cursor: u64,
    phase: String,
    local_scope: Option<crate::core::RecordScope>,
    step_attempt: Option<String>,
    step_begin: Option<String>,
    capture_phase: Option<String>,
    record: Option<String>,
    outcome: Option<SidecarExecutionOutcome>,
    diagnostic: Option<crate::execution::log::Entry>,
    prompt_request: Option<String>,
    artifact_chunk: Option<ArtifactChunk>,
    traceability: Option<String>,
    document: Option<String>,
    lease_request: Option<String>,
    lease_release: Option<String>,
    progress: Option<crate::execution::ExecutionProgress>,
}

struct PromptExchange {
    prompt_id: PromptId,
    outcome: Option<PromptOutcome>,
}

/* === Enums === */
pub(crate) enum PromptOutcome {
    Answered(Box<PromptResponse>),
    Cancelled,
}

pub(crate) enum LeaseOutcome {
    Pending,
    Granted,
    Refused(String),
    Cancelled,
}

/* === Traits === */

/* === Implementations === */
impl ExecutionState {
    pub(crate) fn local_sample(
        &mut self,
        mut sample: crate::telemetry::TelemetrySample,
    ) -> Result<(), String> {
        if self.local_samples.len() >= MAX_LOCAL_SAMPLES {
            return Err(
                "local sample buffer is full; consume local evidence while executing".into(),
            );
        }
        if self.local_inputs {
            sample.scope = Some(self.local_scope.clone());
            sample.received_at = sample.timestamp.clone();
            if !self.receive(sample.clone())? {
                return Err("local sample evidence is closed or duplicated".into());
            }
        }
        self.local_samples.push(sample);
        Ok(())
    }

    pub(crate) fn take_local_samples(&mut self) -> Vec<crate::telemetry::TelemetrySample> {
        std::mem::take(&mut self.local_samples)
    }

    pub(crate) fn new(request: RunExecutionRequest, cancel: watch::Sender<bool>) -> Self {
        Self {
            local_inputs: false,
            local_scope: crate::core::RecordScope {
                test_cycle_id: crate::core::TestCycleId::from_uuid(request.test_cycle_id.clone())
                    .ok(),
                test_execution_id: crate::core::TestExecutionId::from_uuid(
                    request.test_execution_id.clone(),
                )
                .ok(),
                origin: crate::core::EvidenceOrigin::Initialization,
                ..Default::default()
            },
            local_admitted: 0,
            callbacks: crate::execution::Callbacks::default(),
            listening: crate::execution::listening_rows(),
            controls: Arc::new(super::control::Controls::new(request.mode == "interactive")),
            request,
            cancel,
            changed: Arc::new(tokio::sync::Notify::new()),
            frames: VecDeque::new(),
            next_cursor: 1,
            oldest_cursor: 1,
            buffered_artifact_bytes: 0,
            acknowledged_cursor: 0,
            artifact_condvar: Arc::new(Condvar::new()),
            phase: String::new(),
            step_admission: None,
            terminal: false,
            prompt: None,
            stream_rows: Vec::new(),
            sample_positions: BTreeMap::new(),
            samples_closed: false,
            local_samples: Vec::new(),
            produced: BTreeMap::new(),
            delivered: Vec::new(),
            subscriptions: Vec::new(),
            captured_records: Vec::new(),
            subscriptions_settling: false,
            subscriptions_settled: false,
            dropped: 0,
            gaps: Vec::new(),
            leases: BTreeMap::new(),
        }
    }

    pub(crate) fn samples_open(&self) -> bool {
        !self.terminal && !self.samples_closed
    }

    pub(crate) fn receive(
        &mut self,
        sample: crate::telemetry::TelemetrySample,
    ) -> Result<bool, String> {
        self.stage(sample)
    }

    pub(crate) fn validate_sample(
        &self,
        sample: &crate::telemetry::TelemetrySample,
    ) -> Result<(), String> {
        if let Some(position) = self.sample_positions.get(&sample.sample_id)
            && self.stream_rows[*position] != *sample
        {
            return Err("sample identity was reused with different admitted contents".into());
        }
        Ok(())
    }

    pub(crate) fn interrupted(&mut self, gap: crate::telemetry::LeaseGap) {
        if self.terminal {
            return;
        }
        self.gaps.push(gap);
    }

    pub(crate) fn all_gaps(&self) -> Vec<crate::telemetry::LeaseGap> {
        self.gaps.clone()
    }

    pub(crate) fn request_lease(&mut self, request: &crate::telemetry::LeaseRequest) {
        let Ok(encoded) = serde_json::to_string(request) else {
            // A request that cannot be encoded can never be answered, so it is
            // refused here rather than left pending forever.
            self.leases.insert(
                request.lease_token.clone(),
                LeaseOutcome::Refused("the lease request could not be encoded".to_owned()),
            );
            return;
        };
        self.leases
            .insert(request.lease_token.clone(), LeaseOutcome::Pending);
        self.push(BufferedFrame::lease_request(encoded));
    }

    pub(crate) fn resolve_lease(&mut self, lease_token: &str, granted: bool, message: &str) {
        let Some(outcome) = self.leases.get_mut(lease_token) else {
            return;
        };
        if !matches!(outcome, LeaseOutcome::Pending) {
            return;
        }
        *outcome = match granted {
            true => LeaseOutcome::Granted,
            false => LeaseOutcome::Refused(message.to_owned()),
        };
    }

    pub(crate) fn lease_settled(&self, lease_token: &str) -> Option<&LeaseOutcome> {
        self.leases
            .get(lease_token)
            .filter(|outcome| !matches!(outcome, LeaseOutcome::Pending))
    }

    pub(crate) fn release_lease(&mut self, lease_token: &str) {
        if self.leases.remove(lease_token).is_none() {
            return;
        }
        self.push(BufferedFrame::lease_release(lease_token.to_owned()));
    }

    pub(crate) fn subscribe(&mut self, subscriptions: &[crate::definition::Subscription]) {
        self.subscriptions = subscriptions
            .iter()
            .filter_map(crate::definition::Subscription::record)
            .cloned()
            .collect();
    }

    pub(crate) fn begin_record_settle(&mut self) {
        self.subscriptions_settling = true;
    }

    pub(crate) fn record_subscriptions_settled(&self) -> bool {
        self.subscriptions_settled
    }

    pub(crate) fn settle_record_subscriptions(&mut self) {
        if self.subscriptions_settling {
            self.subscriptions_settled = true;
            self.subscriptions_settling = false;
        }
    }

    pub(crate) fn captured_records(&self) -> Vec<crate::records::Record> {
        self.captured_records.clone()
    }

    pub(crate) fn deliver_captured(
        &mut self,
        record: &crate::records::Record,
        owner: &str,
        capture: Option<&crate::core::RecordScope>,
    ) -> bool {
        let own = record.source == owner
            && record
                .scope
                .as_ref()
                .and_then(|scope| scope.test_execution_id.as_ref())
                .is_some_and(|execution| execution.as_str() == self.request.test_execution_id);
        if self.terminal
            || self.subscriptions_settled
            || (capture.is_none() && !self.subscriptions.contains(&record.record_type))
            || (own && capture.is_none())
            || self
                .captured_records
                .iter()
                .any(|captured| captured.record_id == record.record_id)
        {
            return false;
        }
        if self.delivered.len() == MAX_DELIVERED_RECORDS {
            self.delivered.remove(0);
            self.dropped += 1;
        }
        let mut record = record.clone();
        if let Some(scope) = capture {
            record.scope = Some(scope.clone());
        }
        self.captured_records.push(record.clone());
        if !own {
            self.delivered.push(record);
        }

        true
    }

    pub(crate) fn observer_record(
        &mut self,
        mut request: crate::records::RecordPublishRequest,
        source: &str,
        schema_version: u32,
    ) -> bool {
        if self.terminal || self.cancelled() {
            return false;
        }
        request
            .timestamp
            .get_or_insert_with(crate::core::Timestamp::now_utc);
        // The host attaches this drain's execution scope. An authored payload
        // cannot redirect evidence to another execution or broaden it globally.
        request.scope = None;
        let mut record = request.clone().into_record(
            crate::core::RecordId::new(),
            schema_version,
            crate::core::Timestamp::now_utc(),
            source,
        );
        record.scope = Some(crate::core::RecordScope {
            test_cycle_id: crate::core::TestCycleId::from_uuid(self.request.test_cycle_id.clone())
                .ok(),
            test_execution_id: crate::core::TestExecutionId::from_uuid(
                self.request.test_execution_id.clone(),
            )
            .ok(),
            ..Default::default()
        });
        if self.delivered.len() == MAX_DELIVERED_RECORDS {
            self.delivered.remove(0);
            self.dropped += 1;
        }
        self.delivered.push(record);
        self.publish_record(request);
        true
    }

    pub(crate) fn publish_record(&mut self, request: crate::records::RecordPublishRequest) {
        if self.terminal {
            return;
        }
        let encoded =
            serde_json::to_string(&request).expect("a record request contains serializable values");
        self.push(BufferedFrame::record(encoded));
    }

    pub(crate) fn delivered(&self, index: usize) -> Option<crate::records::Record> {
        self.delivered
            .get(index.checked_sub(self.dropped)?)
            .cloned()
    }

    pub(crate) fn next_record(&self, cursor: &mut usize) -> Option<crate::records::Record> {
        *cursor = (*cursor).max(self.dropped);
        let record = self.delivered(*cursor)?;
        *cursor += 1;
        Some(record)
    }

    pub(crate) fn addressed_by(&self, scope: Option<&crate::core::RecordScope>) -> bool {
        self.addressed(scope)
    }

    fn addressed(&self, scope: Option<&crate::core::RecordScope>) -> bool {
        let Some(scope) = scope else {
            return true;
        };
        let names = |filed: Option<&str>, mine: &str| filed.is_none_or(|filed| filed == mine);

        names(
            scope.test_cycle_id.as_ref().map(|id| id.as_str()),
            &self.request.test_cycle_id,
        ) && names(
            scope.test_execution_id.as_ref().map(|id| id.as_str()),
            &self.request.test_execution_id,
        )
    }

    pub(crate) fn terminal(&self) -> bool {
        self.terminal
    }

    pub(crate) fn cancel(&mut self) {
        self.controls.stop_accepting();
        self.cancel.send_replace(true);
        if let Some(exchange) = self.prompt.as_mut()
            && exchange.outcome.is_none()
        {
            exchange.outcome = Some(PromptOutcome::Cancelled);
        }
        // A step waiting on a lease the host has not granted must not outlive
        // the cancellation either. The hold itself needs no undoing: one that
        // was never taken has nothing to release, and one that was is released
        // when its guard drops as the run unwinds.
        for outcome in self.leases.values_mut() {
            if matches!(outcome, LeaseOutcome::Pending) {
                *outcome = LeaseOutcome::Cancelled;
            }
        }
        // A producer parked on the artifact backlog gate must not outlive the
        // cancellation either: nothing is going to drain it now.
        self.artifact_condvar.notify_all();
    }

    pub(crate) fn artifact_condvar(&self) -> Arc<Condvar> {
        Arc::clone(&self.artifact_condvar)
    }

    pub(crate) fn cancelled(&self) -> bool {
        *self.cancel.borrow()
    }

    pub(crate) fn request_step(
        &mut self,
        step: &str,
        start: &crate::execution::StepAttemptStart,
        encoded: String,
    ) {
        self.local_scope.origin = crate::core::EvidenceOrigin::Step {
            attempt_id: start.attempt_id.clone(),
        };
        self.step_admission = Some((start.attempt_id.clone(), None));
        self.phase = step.to_owned();
        self.push(BufferedFrame {
            step_begin: Some(encoded),
            ..Default::default()
        });
    }

    pub(crate) fn acknowledge_step(&mut self, request: &crate::transport::StepAdmission) {
        let Some((id, answer)) = &mut self.step_admission else {
            return;
        };
        if id.as_str() == request.attempt_id && answer.is_none() {
            *answer = Some(if request.accepted {
                Ok(())
            } else {
                Err(request.message.clone())
            });
        }
    }

    pub(crate) fn step_answer(&self) -> Option<Result<(), String>> {
        self.step_admission
            .as_ref()
            .and_then(|(_, answer)| answer.clone())
    }

    pub(crate) fn push(&mut self, mut frame: BufferedFrame) {
        if self.local_inputs {
            if let Some(encoded) = &frame.capture_phase {
                let phase: crate::core::CapturePhase =
                    serde_json::from_str(encoded).expect("runner capture phase serializes");
                self.local_scope.origin = phase.into();
            }
            if frame.step_attempt.is_some() {
                self.local_scope.origin = crate::core::EvidenceOrigin::Default;
            }
            if frame.record.is_some() {
                frame.local_scope = Some(self.local_scope.clone());
            }
        }
        if self.frames.len() == MAX_BUFFERED_FRAMES
            && let Some(evicted) = self.frames.pop_front()
        {
            self.oldest_cursor = evicted.cursor.saturating_add(1);
            // An evicted artifact chunk cannot still count against a parked
            // producer's backlog gate: it is gone from the buffer outright,
            // not merely acknowledged, so nothing will ever drain it.
            if let Some(chunk) = &evicted.artifact_chunk {
                self.buffered_artifact_bytes = self
                    .buffered_artifact_bytes
                    .saturating_sub(chunk.bytes.len() as u64);
            }
        }
        frame.cursor = self.next_cursor;
        frame.phase.clone_from(&self.phase);
        self.next_cursor = self.next_cursor.saturating_add(1);
        self.terminal |= frame.outcome.is_some();
        if self.terminal {
            // Criteria are settled. Retain replay frames, not the staging copy
            // of cleanup or early-failure records until this run is evicted.
            self.captured_records.clear();
        }
        if let Some(chunk) = &frame.artifact_chunk {
            self.buffered_artifact_bytes = self
                .buffered_artifact_bytes
                .saturating_add(chunk.bytes.len() as u64);
        }
        self.frames.push_back(frame);
        self.changed.notify_one();
    }

    pub(crate) fn artifact_backlog(&self) -> u64 {
        self.buffered_artifact_bytes
    }

    pub(crate) fn raise(&mut self, prompt_id: PromptId, request: String) {
        self.prompt = Some(PromptExchange {
            prompt_id,
            outcome: None,
        });
        self.push(BufferedFrame {
            prompt_request: Some(request),
            ..BufferedFrame::default()
        });
    }

    pub(crate) fn answer(&mut self, prompt_id: &str, response: PromptResponse) {
        if let Some(exchange) = self.prompt.as_mut()
            && exchange.prompt_id.as_str() == prompt_id
            && exchange.outcome.is_none()
        {
            exchange.outcome = Some(PromptOutcome::Answered(Box::new(response)));
        }
    }

    pub(crate) fn settled(&self, prompt_id: &PromptId) -> Option<&PromptOutcome> {
        self.prompt
            .as_ref()
            .filter(|exchange| &exchange.prompt_id == prompt_id)
            .and_then(|exchange| exchange.outcome.as_ref())
    }

    pub(crate) fn stage(
        &mut self,
        sample: crate::telemetry::TelemetrySample,
    ) -> Result<bool, String> {
        if !self.samples_open() {
            return Ok(false);
        }
        self.validate_sample(&sample)?;
        if self.sample_positions.contains_key(&sample.sample_id) {
            return Ok(false);
        }
        *self.produced.entry(sample.stream.to_string()).or_default() += 1;
        self.sample_positions
            .insert(sample.sample_id.clone(), self.stream_rows.len());
        self.stream_rows.push(sample);
        Ok(true)
    }

    pub(crate) fn produced(&self, stream: &str) -> u64 {
        self.produced.get(stream).copied().unwrap_or_default()
    }

    pub(crate) fn take_staged(&mut self) -> Vec<crate::telemetry::TelemetrySample> {
        self.samples_closed = true;
        self.sample_positions.clear();
        std::mem::take(&mut self.stream_rows)
    }

    pub(crate) fn admit_local_inputs(
        &mut self,
        through: u64,
        source: &str,
        contracts: &[crate::records::RecordContract],
        admitted_at: &crate::core::Timestamp,
    ) -> Result<(), Refusal> {
        if !self.local_inputs {
            return Ok(());
        }
        if self.local_admitted.saturating_add(1) < self.oldest_cursor {
            return Err(Refusal::of(
                "local input admission expired before retained publications",
            ));
        }
        let frames = self
            .frames
            .iter()
            .filter(|frame| frame.cursor > self.local_admitted && frame.cursor <= through)
            .filter_map(|frame| {
                frame
                    .record
                    .as_ref()
                    .map(|encoded| (frame.cursor, encoded.clone(), frame.local_scope.clone()))
            })
            .collect::<Vec<_>>();
        let mut admitted = Vec::with_capacity(frames.len());
        for (cursor, encoded, scope) in frames {
            let mut request: crate::records::RecordPublishRequest =
                serde_json::from_str(&encoded).map_err(|error| Refusal::of(error.to_string()))?;
            let contract = contracts
                .iter()
                .find(|contract| contract.record_type == request.record_type);
            let version = contract.map_or(1, |contract| contract.schema_version);
            request.scope = scope.clone();
            let mut record = request.into_record(
                crate::core::RecordId::from_seed(format!(
                    "{}:local:{cursor}",
                    self.request.execution_token
                )),
                version,
                admitted_at.clone(),
                source,
            );
            record.details = crate::records::validate_record(
                contract,
                &record,
                crate::core::ValidationMode::Strict,
            )
            .map_err(|error| Refusal::of(error.to_string()))?;
            admitted.push((record, scope));
        }
        for (record, scope) in admitted {
            self.deliver_captured(&record, source, scope.as_ref());
        }
        self.local_admitted = self.local_admitted.max(through);
        Ok(())
    }

    pub(crate) fn settle_local_inputs(
        &mut self,
        source: &str,
        contracts: &[crate::records::RecordContract],
        admitted_at: &crate::core::Timestamp,
    ) -> Result<(), Refusal> {
        self.admit_local_inputs(
            self.next_cursor.saturating_sub(1),
            source,
            contracts,
            admitted_at,
        )?;
        self.settle_record_subscriptions();
        Ok(())
    }

    pub(crate) fn drain_after(&mut self, cursor: u64) -> Result<EvidenceBatch, Refusal> {
        if cursor.saturating_add(1) < self.oldest_cursor {
            return Err(Refusal::of(format!(
                "execution cursor {cursor} expired before buffered cursor {}",
                self.oldest_cursor
            )));
        }

        let mut reply = EvidenceBatch {
            cleanup_settled: self.terminal,
            cursor,
            // An empty phase with an unchanged cursor is the protocol's
            // explicit "nothing new yet" poll response.
            phase: String::new(),
            step_attempts: Vec::new(),
            step_begins: Vec::new(),
            capture_phases: Vec::new(),
            records: Vec::new(),
            outcome: None,
            diagnostics: Vec::new(),
            prompt_requests: Vec::new(),
            artifact_chunks: Vec::new(),
            traceability: Vec::new(),
            documents: Vec::new(),
            lease_requests: Vec::new(),
            lease_releases: Vec::new(),
            progress: None,
            record_subscriptions_settling: false,
        };
        for frame in self.frames.iter().filter(|frame| frame.cursor > cursor) {
            // A reply has one phase field: never erase an earlier transition by
            // coalescing frames from the next step into the same reply.
            if reply.cursor > cursor
                && (frame.phase != reply.phase
                    || frame.step_attempt.is_some()
                    || frame.step_begin.is_some()
                    || frame.capture_phase.is_some())
            {
                break;
            }
            let previous_cursor = reply.cursor;
            let previous_phase = reply.phase.clone();
            let previous_outcome = reply.outcome.clone();
            let previous_begins = reply.step_begins.len();
            let previous_phases = reply.capture_phases.len();
            let previous_attempts = reply.step_attempts.len();
            let previous_records = reply.records.len();
            let previous_diagnostics = reply.diagnostics.len();
            let previous_prompt_requests = reply.prompt_requests.len();
            let previous_artifact_chunks = reply.artifact_chunks.len();
            let previous_traceability = reply.traceability.len();
            let previous_documents = reply.documents.len();
            let previous_lease_requests = reply.lease_requests.len();
            let previous_lease_releases = reply.lease_releases.len();
            let previous_progress = reply.progress.clone();

            reply.cursor = frame.cursor;
            reply.phase.clone_from(&frame.phase);
            if let Some(phase) = &frame.capture_phase {
                reply.capture_phases.push(phase.clone());
            }
            if let Some(start) = &frame.step_begin {
                reply.step_begins.push(start.clone());
            }
            if let Some(attempt) = &frame.step_attempt {
                reply.step_attempts.push(attempt.clone());
            }
            if let Some(record) = &frame.record {
                reply.records.push(record.clone());
            }
            if let Some(outcome) = &frame.outcome {
                reply.outcome = Some(outcome.clone());
            }
            if let Some(log_line) = &frame.diagnostic {
                reply.diagnostics.push(log_line.clone());
            }
            if let Some(prompt_request) = &frame.prompt_request {
                reply.prompt_requests.push(prompt_request.clone());
            }
            if let Some(artifact_chunk) = &frame.artifact_chunk {
                reply.artifact_chunks.push(artifact_chunk.clone());
            }
            if let Some(section) = &frame.traceability {
                reply.traceability.push(section.clone());
            }
            if let Some(change) = &frame.document {
                reply.documents.push(change.clone());
            }
            if let Some(request) = &frame.lease_request {
                reply.lease_requests.push(request.clone());
            }
            if let Some(release) = &frame.lease_release {
                reply.lease_releases.push(release.clone());
            }
            if let Some(progress) = &frame.progress {
                reply.progress = Some(progress.clone());
            }

            // A lone item may use the whole frame; pages of several stay
            // inside the admission budget.
            let alone = previous_cursor == cursor;
            if page_fits(&reply)
                || (alone && Frame::request_fits::<crate::transport::PushEvidence>(&reply))
            {
                if frame.step_attempt.is_some()
                    || frame.step_begin.is_some()
                    || frame.capture_phase.is_some()
                {
                    break;
                }
                continue;
            }

            reply.cursor = previous_cursor;
            reply.phase = previous_phase;
            reply.outcome = previous_outcome;
            reply.step_begins.truncate(previous_begins);
            reply.capture_phases.truncate(previous_phases);
            reply.step_attempts.truncate(previous_attempts);
            reply.records.truncate(previous_records);
            reply.diagnostics.truncate(previous_diagnostics);
            reply.prompt_requests.truncate(previous_prompt_requests);
            reply.artifact_chunks.truncate(previous_artifact_chunks);
            reply.traceability.truncate(previous_traceability);
            reply.documents.truncate(previous_documents);
            reply.lease_requests.truncate(previous_lease_requests);
            reply.lease_releases.truncate(previous_lease_releases);
            reply.progress = previous_progress;

            if reply.cursor == cursor {
                return Err(Refusal::of(
                    "one execution output item exceeds the Harmos transport frame",
                ));
            }
            break;
        }

        // Newly assembled pages retire their artifact chunks from the
        // back-pressure backlog. Frames already covered by an earlier
        // acknowledgment are skipped, so replaying a repeated cursor cannot
        // subtract the same bytes twice.
        let previously_acknowledged = self.acknowledged_cursor;
        if reply.cursor > previously_acknowledged {
            for frame in self.frames.iter().filter(|frame| {
                frame.cursor > previously_acknowledged && frame.cursor <= reply.cursor
            }) {
                if let Some(chunk) = &frame.artifact_chunk {
                    self.buffered_artifact_bytes = self
                        .buffered_artifact_bytes
                        .saturating_sub(chunk.bytes.len() as u64);
                }
            }
            self.acknowledged_cursor = reply.cursor;
            // The producer parked on this execution's gate may now find it
            // open; wake it up to re-check rather than leave it to the
            // bounded poll `FrameArtifactSink::push` falls back to.
            self.artifact_condvar.notify_all();
        }

        reply.record_subscriptions_settling =
            self.subscriptions_settling && reply.cursor.saturating_add(1) == self.next_cursor;
        Ok(reply)
    }
}

impl BufferedFrame {
    pub(crate) fn capture_phase(phase: String) -> Self {
        Self {
            capture_phase: Some(phase),
            ..Default::default()
        }
    }

    pub(crate) fn step_attempt(attempt: String) -> Self {
        Self {
            step_attempt: Some(attempt),
            ..Default::default()
        }
    }

    pub(crate) fn progress(progress: crate::execution::ExecutionProgress) -> Self {
        Self {
            progress: Some(progress),
            ..Self::default()
        }
    }

    pub(crate) fn record(record: String) -> Self {
        Self {
            record: Some(record),
            ..Self::default()
        }
    }

    pub(crate) fn log(level: crate::execution::log::Level, message: String) -> Self {
        Self {
            diagnostic: Some(crate::execution::log::Entry::new(level, message)),
            ..Self::default()
        }
    }

    pub(crate) fn artifact(artifact_chunk: ArtifactChunk) -> Self {
        Self {
            artifact_chunk: Some(artifact_chunk),
            ..Self::default()
        }
    }

    pub(crate) fn traceability(section: String) -> Self {
        Self {
            traceability: Some(section),
            ..Self::default()
        }
    }

    pub(crate) fn document(change: String) -> Self {
        Self {
            document: Some(change),
            ..Self::default()
        }
    }

    pub(crate) fn lease_request(request: String) -> Self {
        Self {
            lease_request: Some(request),
            ..Self::default()
        }
    }

    pub(crate) fn lease_release(lease_token: String) -> Self {
        Self {
            lease_release: Some(lease_token),
            ..Self::default()
        }
    }

    pub(crate) fn terminal(outcome: SidecarExecutionOutcome) -> Self {
        Self {
            diagnostic: outcome.message.clone().map(|message| {
                crate::execution::log::Entry::new(
                    if outcome.passed {
                        crate::execution::log::Level::Info
                    } else {
                        crate::execution::log::Level::Error
                    },
                    message,
                )
            }),
            outcome: Some(outcome),
            ..Self::default()
        }
    }
}

/* === Public Functions === */
pub(crate) fn lock(executions: &Executions) -> MutexGuard<'_, BTreeMap<String, ExecutionState>> {
    executions.lock().unwrap_or_else(PoisonError::into_inner)
}

/* === Private Functions === */

fn page_fits(page: &crate::transport::EvidenceBatch) -> bool {
    use harmos::Message;
    page.encoded_len() <= crate::transport::protocol::MAX_EVIDENCE_PAGE_BYTES
        && Frame::request_fits::<crate::transport::PushEvidence>(page)
}

/* === Tests === */
