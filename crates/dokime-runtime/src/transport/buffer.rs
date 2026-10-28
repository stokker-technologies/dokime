/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};

use harmos::sidecar::{Frame, Refusal};
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::prompts::PromptResponse;
use crate::transport::{ArtifactChunk, EvidenceBatch, RunExecution, SidecarExecutionOutcome};
use dokime_domain::model::identity::PromptId;

// Replay Frames Retained Per Execution; Only Host-Confirmed Frames Are Evicted By Progress.
const RETAINED_FRAMES: usize = 1024;

// Lease Gaps Retained Per Stream: The Earliest Ones Plus The Latest In The Final Slot.
const RETAINED_STREAM_GAPS: usize = 64;

const RECENT_STREAM_SAMPLES: usize = 64;

pub(crate) type Executions = Arc<Mutex<BTreeMap<String, ExecutionState>>>;
pub(crate) type Tasks = Arc<Mutex<Vec<JoinHandle<()>>>>;

pub(crate) struct ExecutionState {
    pub(crate) local_inputs: bool,
    local_scope: dokime_domain::model::execution::RecordScope,
    local_admitted: u64,
    pub(crate) callbacks: crate::execution::Callbacks,
    pub(crate) listening: tokio::sync::broadcast::Sender<crate::execution::PublishedRow>,
    pub(crate) request: RunExecution,
    pub(crate) controls: Arc<super::control::Controls>,
    pub(crate) cancel: watch::Sender<bool>,
    pub(crate) changed: Arc<tokio::sync::Notify>,

    // Wakes every host-exchange waiter when a prompt, lease or step settles, the run is cancelled, or it is evicted.
    pub(crate) settled: Arc<tokio::sync::Notify>,

    frames: VecDeque<BufferedFrame>,
    next_cursor: u64,
    oldest_cursor: u64,
    buffered_artifact_bytes: u64,
    acknowledged_cursor: u64,
    // Highest Cursor The Host Confirmed By Draining After It.
    confirmed_cursor: u64,
    // Cursor Of The One Retained Progress Frame; A Newer Update Replaces It.
    progress_cursor: Option<u64>,
    // Latest Progress That Found The Ring Full Of Unconfirmed Evidence.
    deferred_progress: Option<crate::execution::ExecutionProgress>,
    artifact_condvar: Arc<Condvar>,
    phase: String,
    step_admission: Option<(dokime_domain::model::identity::StepAttemptId, Option<Result<(), String>>)>,
    terminal: bool,
    prompt: Option<PromptExchange>,
    stream_rows: crate::execution::evidence_rows::EvidenceRows<crate::telemetry::TelemetrySample>,
    sample_streams: BTreeMap<String, SampleStream>,
    confirmed_samples: usize,
    samples_closed: bool,
    operation_settled: bool,
    local_samples: Vec<crate::telemetry::TelemetrySample>,
    // How many of the held local samples a wait has already reported.
    local_samples_announced: usize,
    produced: BTreeMap<String, u64>,
    delivered: VecDeque<dokime_domain::model::records::evidence::Record>,
    subscriptions: Vec<dokime_domain::model::identity::Identifier>,
    captured_records: crate::execution::evidence_rows::EvidenceRows<dokime_domain::model::records::evidence::Record>,
    pub(crate) record_ordinal: u64,
    recent_records: VecDeque<RecentRecord>,
    subscriptions_settling: bool,
    subscriptions_settled: bool,
    dropped: usize,
    gaps: Vec<crate::telemetry::LeaseGap>,
    // Every Gap Counted Per Stream, Including Those Beyond The Retained Examples.
    gap_counts: BTreeMap<String, u64>,
    leases: BTreeMap<String, LeaseOutcome>,
}

#[derive(Default)]
pub(crate) struct BufferedFrame {
    cursor: u64,
    phase: String,
    local_scope: Option<dokime_domain::model::execution::RecordScope>,
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

#[derive(Default)]
struct SampleStream {
    high_water: Option<u64>,
    recent: VecDeque<RecentSample>,
}

struct RecentRecord {
    ordinal: u64,
    identity: String,
    row: crate::execution::evidence_rows::EvidenceRow<dokime_domain::model::records::evidence::Record>,
}

struct RecentSample {
    sample_id: String,
    row: crate::execution::evidence_rows::EvidenceRow<crate::telemetry::TelemetrySample>,
    confirmed: bool,
}

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

impl ExecutionState {
    pub(crate) fn announce_local_samples(&mut self) -> bool {
        let fresh = self.local_samples.len() > self.local_samples_announced;
        self.local_samples_announced = self.local_samples.len();
        fresh
    }

    pub(crate) fn local_sample(&mut self, mut sample: crate::telemetry::TelemetrySample) -> Result<(), String> {
        if self.local_samples.len() as u64 >= 1024 {
            return Err("local sample buffer is full; consume local evidence while executing".into());
        }
        if self.local_inputs {
            sample.scope = Some(self.local_scope.clone());
            sample.received_at = sample.timestamp.clone();
            if !sample.values.contains_key("timestamp") {
                let timestamp = sample
                    .timestamp
                    .unix_millis()
                    .ok_or_else(|| "local sample timestamp is not an RFC 3339 instant at or after the Unix epoch".to_owned())?;
                sample.values.insert("timestamp".to_owned(), dokime_domain::model::values::Value::Number(timestamp as f64 / 1_000.0));
            }
            if !self.receive(sample.clone())? {
                return Err("local sample evidence is closed or duplicated".into());
            }
        }
        self.local_samples.push(sample);
        // A Local Consumer Drains Samples As They Arrive, So Each One Wakes It.
        self.changed.notify_waiters();
        Ok(())
    }

    pub(crate) fn take_local_samples(&mut self) -> Vec<crate::telemetry::TelemetrySample> {
        self.local_samples_announced = 0;
        std::mem::take(&mut self.local_samples)
    }

    pub(crate) fn new(request: RunExecution, cancel: watch::Sender<bool>) -> Self {
        Self {
            local_inputs: false,
            local_scope: dokime_domain::model::execution::RecordScope {
                test_cycle_id: dokime_domain::model::identity::TestCycleId::from_uuid(request.test_cycle_id.clone()).ok(),
                test_execution_id: dokime_domain::model::identity::TestExecutionId::from_uuid(request.test_execution_id.clone()).ok(),
                origin: dokime_domain::model::execution::EvidenceOrigin::Initialization,
                ..Default::default()
            },
            local_admitted: 0,
            callbacks: crate::execution::Callbacks::default(),
            listening: crate::execution::listening_rows(1024),
            controls: Arc::new(super::control::Controls::new(request.mode == "interactive", 4096)),
            request,
            cancel,
            changed: Arc::new(tokio::sync::Notify::new()),
            settled: Arc::new(tokio::sync::Notify::new()),
            frames: VecDeque::new(),
            next_cursor: 1,
            oldest_cursor: 1,
            buffered_artifact_bytes: 0,
            acknowledged_cursor: 0,
            confirmed_cursor: 0,
            progress_cursor: None,
            deferred_progress: None,
            artifact_condvar: Arc::new(Condvar::new()),
            phase: String::new(),
            step_admission: None,
            terminal: false,
            prompt: None,
            stream_rows: Default::default(),
            sample_streams: BTreeMap::new(),
            confirmed_samples: 0,
            samples_closed: false,
            operation_settled: false,
            local_samples: Vec::new(),
            local_samples_announced: 0,
            produced: BTreeMap::new(),
            delivered: VecDeque::new(),
            subscriptions: Vec::new(),
            captured_records: Default::default(),
            record_ordinal: 0,
            recent_records: VecDeque::new(),
            subscriptions_settling: false,
            subscriptions_settled: false,
            dropped: 0,
            gaps: Vec::new(),
            gap_counts: BTreeMap::new(),
            leases: BTreeMap::new(),
        }
    }

    pub(crate) fn samples_open(&self) -> bool {
        !self.terminal && !self.samples_closed
    }

    pub(crate) fn receive(&mut self, sample: crate::telemetry::TelemetrySample) -> Result<bool, String> {
        self.stage(sample)
    }

    pub(crate) fn sample_receipts(&self) -> Vec<super::protocol::SampleReceipt> {
        self.sample_streams
            .iter()
            .filter_map(|(stream, state)| {
                state.recent.back().map(|row| super::protocol::SampleReceipt {
                    stream: stream.clone(),
                    sample_id: row.sample_id.clone(),
                })
            })
            .collect()
    }

    pub(crate) fn validate_sample(&self, sample: &crate::telemetry::TelemetrySample) -> Result<(), String> {
        let Some(stream) = self.sample_streams.get(sample.stream.as_str()) else {
            return Ok(());
        };
        let previous = stream.recent.iter().find(|previous| previous.sample_id == sample.sample_id.as_str());
        let previous = match previous {
            Some(previous) => Some(previous.row.read()?),
            None => None,
        };
        if let Some(previous) = &previous
            && let Some(field) = sample_difference(previous, sample, self.local_inputs)
        {
            let key = (field == "values")
                .then(|| {
                    previous
                        .values
                        .keys()
                        .chain(sample.values.keys())
                        .find(|key| !value_equivalent(previous.values.get(*key), sample.values.get(*key)))
                })
                .flatten();
            let suffix = key.map_or_else(String::new, |key| format!(".{key}"));
            return Err(format!("sample identity was reused with different admitted contents: {field}{suffix}"));
        }
        if previous.is_none() && stream.high_water.is_some_and(|position| sample.sequence <= position) {
            return Err("sample stream sequence is out of order".to_owned());
        }
        Ok(())
    }

    pub(crate) fn interrupted(&mut self, gap: crate::telemetry::LeaseGap) {
        if self.terminal {
            return;
        }
        let count = self.gap_counts.entry(gap.stream.to_string()).or_default();
        *count = count.checked_add(1).expect("execution gap counter overflow");
        // A Multi-Day Run Keeps Bounded Gap Examples Per Stream: Once Full, The Last Slot Holds The Latest Gap.
        let mut retained = self.gaps.iter().enumerate().filter(|(_, held)| held.stream == gap.stream);
        if let Some((latest, _)) = retained.by_ref().nth(RETAINED_STREAM_GAPS - 1) {
            self.gaps[latest] = gap;
            return;
        }
        self.gaps.push(gap);
    }

    pub(crate) fn all_gaps(&self) -> Vec<crate::telemetry::LeaseGap> {
        self.gaps.clone()
    }

    pub(crate) fn gap_counts(&self) -> &BTreeMap<String, u64> {
        &self.gap_counts
    }

    pub(crate) fn request_lease(&mut self, request: &crate::telemetry::LeaseRequest) {
        let Ok(encoded) = serde_json::to_string(request) else {
            // A request that cannot be encoded can never be answered, so it is
            // refused here rather than left pending forever.
            self.leases.insert(request.lease_token.clone(), LeaseOutcome::Refused("the lease request could not be encoded".to_owned()));
            return;
        };
        self.leases.insert(request.lease_token.clone(), LeaseOutcome::Pending);
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
        self.settled.notify_waiters();
    }

    pub(crate) fn lease_settled(&self, lease_token: &str) -> Option<&LeaseOutcome> {
        self.leases.get(lease_token).filter(|outcome| !matches!(outcome, LeaseOutcome::Pending))
    }

    pub(crate) fn release_lease(&mut self, lease_token: &str) {
        if self.leases.remove(lease_token).is_none() {
            return;
        }
        self.push(BufferedFrame::lease_release(lease_token.to_owned()));
    }

    pub(crate) fn subscribe(&mut self, subscriptions: &[crate::definition::Subscription]) {
        self.subscriptions = subscriptions.iter().filter_map(crate::definition::Subscription::record).cloned().collect();
    }

    pub(crate) fn begin_record_settle(&mut self) {
        self.subscriptions_settling = true;
        self.changed.notify_waiters();
    }

    pub(crate) fn record_subscriptions_settled(&self) -> bool {
        self.subscriptions_settled
    }

    pub(crate) fn record_inputs_closed(&self) -> bool {
        self.terminal || self.operation_settled || self.subscriptions_settled
    }

    pub(crate) fn settle_operation(&mut self) {
        self.operation_settled = true;
        self.samples_closed = true;
        if !self.terminal {
            self.cancel();
        }
        self.settled.notify_waiters();
    }

    pub(crate) fn settlement_progress(&self) -> usize {
        self.stream_rows.len() + self.captured_records.len() + self.confirmed_samples
    }

    pub(crate) fn settle_record_subscriptions(&mut self) {
        if self.subscriptions_settling {
            self.subscriptions_settled = true;
            self.subscriptions_settling = false;
            self.samples_closed = true;
            self.settled.notify_waiters();
        }
    }

    pub(crate) fn captured_records(
        &self,
    ) -> crate::execution::evidence_rows::EvidenceRows<dokime_domain::model::records::evidence::Record> {
        self.captured_records.clone()
    }

    pub(crate) fn deliver_captured(
        &mut self, ordinal: u64, record: &dokime_domain::model::records::evidence::Record, owner: &str,
        capture: Option<&dokime_domain::model::execution::RecordScope>,
    ) -> bool {
        let own = record.source == owner
            && record
                .scope
                .as_ref()
                .and_then(|scope| scope.test_execution_id.as_ref())
                .is_some_and(|execution| execution.as_str() == self.request.test_execution_id);
        if self.record_inputs_closed()
            || (capture.is_none() && !self.subscriptions.contains(&record.record_type))
            || (own && capture.is_none())
        {
            return false;
        }
        let mut record = record.clone();
        if let Some(scope) = capture {
            record.scope = Some(scope.clone());
        }
        let admitted = (|| -> Result<bool, String> {
            if let Some(recent) = self.recent_records.iter().find(|recent| recent.ordinal == ordinal) {
                let previous = recent.row.read()?;
                return if previous == record {
                    Ok(false)
                } else {
                    Err("captured record ordinal was reused with different contents".into())
                };
            }
            if ordinal <= self.record_ordinal {
                return Err("captured record ordinal is out of order".into());
            }
            for recent in &self.recent_records {
                if recent.identity == record.record_id.as_str() {
                    return Err("captured record identity was reused".into());
                }
            }
            let row = self.captured_records.retain(&record)?;
            if self.recent_records.len() == RECENT_STREAM_SAMPLES {
                self.recent_records.pop_front();
            }
            self.recent_records.push_back(RecentRecord {
                ordinal,
                identity: record.record_id.to_string(),
                row,
            });
            self.record_ordinal = ordinal;
            Ok(true)
        })();
        match admitted {
            Ok(true) => {}
            Ok(false) => return false,
            Err(message) => {
                self.cancel();
                self.push(BufferedFrame::terminal(SidecarExecutionOutcome {
                    passed: false,
                    message: Some(message),
                    ..Default::default()
                }));
                return false;
            }
        }
        if self.delivered.len() as u64 >= 256 {
            self.delivered.pop_front();
            self.dropped += 1;
        }
        if !own {
            self.delivered.push_back(record);
        }
        self.settled.notify_waiters();

        true
    }

    pub(crate) fn observer_record(
        &mut self, mut request: dokime_domain::model::records::publication::RecordPublishRequest, source: &str, schema_version: u32,
    ) -> bool {
        if self.terminal || self.cancelled() {
            return false;
        }
        request.timestamp.get_or_insert_with(crate::model::time::now_utc);
        // The host attaches this drain's execution scope. An authored payload
        // cannot redirect evidence to another execution or broaden it globally.
        request.scope = None;
        let mut record = request.clone().into_record(
            crate::model::identity::new_id::<dokime_domain::model::identity::RecordId>(),
            schema_version,
            crate::model::time::now_utc(),
            source,
        );
        record.scope = Some(dokime_domain::model::execution::RecordScope {
            test_cycle_id: dokime_domain::model::identity::TestCycleId::from_uuid(self.request.test_cycle_id.clone()).ok(),
            test_execution_id: dokime_domain::model::identity::TestExecutionId::from_uuid(self.request.test_execution_id.clone()).ok(),
            ..Default::default()
        });
        if self.delivered.len() as u64 >= 256 {
            self.delivered.pop_front();
            self.dropped += 1;
        }
        self.delivered.push_back(record);
        self.settled.notify_waiters();
        self.publish_record(request);
        true
    }

    pub(crate) fn publish_record(&mut self, request: dokime_domain::model::records::publication::RecordPublishRequest) {
        if self.terminal {
            return;
        }
        let encoded = serde_json::to_string(&request).expect("a record request contains serializable values");
        self.push(BufferedFrame::record(encoded));
    }

    pub(crate) fn delivered(&self, index: usize) -> Option<dokime_domain::model::records::evidence::Record> {
        self.delivered.get(index.checked_sub(self.dropped)?).cloned()
    }

    pub(crate) fn next_record(&self, cursor: &mut usize) -> Option<dokime_domain::model::records::evidence::Record> {
        *cursor = (*cursor).max(self.dropped);
        let record = self.delivered(*cursor)?;
        *cursor += 1;
        Some(record)
    }

    pub(crate) fn addressed_by(&self, scope: Option<&dokime_domain::model::execution::RecordScope>) -> bool {
        self.addressed(scope)
    }

    fn addressed(&self, scope: Option<&dokime_domain::model::execution::RecordScope>) -> bool {
        let Some(scope) = scope else {
            return true;
        };
        let names = |filed: Option<&str>, mine: &str| filed.is_none_or(|filed| filed == mine);

        names(scope.test_cycle_id.as_ref().map(|id| id.as_str()), &self.request.test_cycle_id)
            && names(scope.test_execution_id.as_ref().map(|id| id.as_str()), &self.request.test_execution_id)
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
        if let Some((_, answer)) = &mut self.step_admission {
            answer.get_or_insert_with(|| Err("execution cancelled before step acknowledgement".to_owned()));
        }
        // A producer parked on the artifact backlog gate must not outlive the
        // cancellation either: nothing is going to drain it now.
        self.artifact_condvar.notify_all();
        self.settled.notify_waiters();
    }

    pub(crate) fn artifact_condvar(&self) -> Arc<Condvar> {
        Arc::clone(&self.artifact_condvar)
    }

    pub(crate) fn cancelled(&self) -> bool {
        *self.cancel.borrow()
    }

    pub(crate) fn request_step(&mut self, step: &str, start: &dokime_domain::model::execution::result::StepAttemptStart, encoded: String) {
        self.local_scope.origin = dokime_domain::model::execution::EvidenceOrigin::Step { attempt_id: start.attempt_id.clone() };
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
            *answer = Some(if request.accepted { Ok(()) } else { Err(request.message.clone()) });
            self.settled.notify_waiters();
        }
    }

    pub(crate) fn step_answer(&self) -> Option<Result<(), String>> {
        self.step_admission.as_ref().and_then(|(_, answer)| answer.clone())
    }

    pub(crate) fn push(&mut self, mut frame: BufferedFrame) {
        if self.local_inputs {
            if let Some(encoded) = &frame.capture_phase {
                let phase: dokime_domain::model::execution::CapturePhase =
                    serde_json::from_str(encoded).expect("runner capture phase serializes");
                self.local_scope.origin = phase.into();
            }
            if frame.step_attempt.is_some() {
                self.local_scope.origin = dokime_domain::model::execution::EvidenceOrigin::Default;
            }
            if frame.record.is_some() {
                frame.local_scope = Some(self.local_scope.clone());
            }
        }
        let progress = frame.progress.is_some();
        if progress && !self.admit_progress(&mut frame) {
            return;
        }
        if !progress && self.frames.len() >= RETAINED_FRAMES {
            self.yield_progress_slot();
        }
        if self.frames.len() >= RETAINED_FRAMES
            && let Some(evicted) = self.frames.pop_front()
        {
            self.oldest_cursor = evicted.cursor.saturating_add(1);
            if self.progress_cursor == Some(evicted.cursor) {
                self.progress_cursor = None;
            }
            // An evicted artifact chunk cannot still count against a parked
            // producer's backlog gate: it is gone from the buffer outright,
            // not merely acknowledged, so nothing will ever drain it.
            if let Some(chunk) = &evicted.artifact_chunk {
                self.buffered_artifact_bytes = self.buffered_artifact_bytes.saturating_sub(chunk.bytes.len() as u64);
                self.artifact_condvar.notify_all();
            }
        }
        if progress {
            self.progress_cursor = Some(self.next_cursor);
        }
        frame.cursor = self.next_cursor;
        frame.phase.clone_from(&self.phase);
        self.next_cursor = self.next_cursor.saturating_add(1);
        self.terminal |= frame.outcome.is_some();
        if self.terminal {
            // Criteria are settled. Retain replay frames, not the staging copy
            // of cleanup or early-failure records until this run is evicted.
            self.captured_records = Default::default();
            self.settled.notify_waiters();
        }
        if let Some(chunk) = &frame.artifact_chunk {
            self.buffered_artifact_bytes = self.buffered_artifact_bytes.saturating_add(chunk.bytes.len() as u64);
        }
        self.frames.push_back(frame);
        self.changed.notify_waiters();
        self.settled.notify_waiters();
    }

    // Progress Is A Latest Value: The Newer Update Replaces The Retained One Instead Of Queuing Behind It,
    // And It Never Evicts Evidence The Host Has Not Confirmed; It Waits For The Next Drain Instead.
    fn admit_progress(&mut self, frame: &mut BufferedFrame) -> bool {
        // A Settled Run Reports No Further Progress After Its Outcome.
        if self.terminal {
            self.deferred_progress = None;
            return false;
        }
        self.take_progress_frame();

        let full = self.frames.len() >= RETAINED_FRAMES;
        let unconfirmed = self.frames.front().is_some_and(|front| front.cursor > self.confirmed_cursor);
        self.deferred_progress = (full && unconfirmed).then(|| frame.progress.take()).flatten();
        self.deferred_progress.is_none()
    }

    // Evidence Arriving At A Full Ring Of Unconfirmed Frames Takes The Progress Slot Before Any Evidence Is Evicted.
    fn yield_progress_slot(&mut self) {
        if self.frames.front().is_some_and(|front| front.cursor <= self.confirmed_cursor) {
            return;
        }
        let progress = self.take_progress_frame();
        self.deferred_progress = progress.or(self.deferred_progress.take());
    }

    fn take_progress_frame(&mut self) -> Option<crate::execution::ExecutionProgress> {
        let cursor = self.progress_cursor.take()?;
        let index = self.frames.binary_search_by_key(&cursor, |frame| frame.cursor).ok()?;
        self.frames.remove(index).and_then(|frame| frame.progress)
    }

    pub(crate) fn artifact_backlog(&self) -> u64 {
        self.buffered_artifact_bytes
    }

    pub(crate) fn raise(&mut self, prompt_id: PromptId, request: String) {
        self.prompt = Some(PromptExchange { prompt_id, outcome: None });
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
            self.settled.notify_waiters();
        }
    }

    pub(crate) fn settled(&self, prompt_id: &PromptId) -> Option<&PromptOutcome> {
        self.prompt.as_ref().filter(|exchange| &exchange.prompt_id == prompt_id).and_then(|exchange| exchange.outcome.as_ref())
    }

    pub(crate) fn stage(&mut self, sample: crate::telemetry::TelemetrySample) -> Result<bool, String> {
        if !self.samples_open() {
            return Ok(false);
        }
        self.validate_sample(&sample)?;
        let stream = self.sample_streams.entry(sample.stream.to_string()).or_default();
        if let Some(previous) = stream.recent.iter_mut().find(|previous| previous.sample_id == sample.sample_id.as_str()) {
            if self.subscriptions_settling && !previous.confirmed {
                previous.confirmed = true;
                self.confirmed_samples += 1;
                self.settled.notify_waiters();
            }
            return Ok(false);
        }
        let row = self.stream_rows.retain(&sample)?;
        stream.high_water = Some(sample.sequence);
        if stream.recent.len() == RECENT_STREAM_SAMPLES {
            stream.recent.pop_front();
        }
        stream.recent.push_back(RecentSample {
            sample_id: sample.sample_id.to_string(),
            row,
            confirmed: false,
        });
        *self.produced.entry(sample.stream.to_string()).or_default() += 1;
        self.settled.notify_waiters();
        Ok(true)
    }

    pub(crate) fn produced(&self, stream: &str) -> u64 {
        self.produced.get(stream).copied().unwrap_or_default()
    }

    pub(crate) fn take_staged(&mut self) -> crate::execution::evidence_rows::EvidenceRows<crate::telemetry::TelemetrySample> {
        self.samples_closed = true;
        std::mem::take(&mut self.stream_rows)
    }

    pub(crate) fn admit_local_inputs(
        &mut self, through: u64, source: &str, contracts: &[crate::records::RecordContract],
        admitted_at: &dokime_domain::model::time::Timestamp,
    ) -> Result<(), Refusal> {
        if !self.local_inputs {
            return Ok(());
        }
        if self.local_admitted.saturating_add(1) < self.oldest_cursor {
            return Err(Refusal::of("local input admission expired before retained publications"));
        }
        let frames = self
            .frames
            .iter()
            .filter(|frame| frame.cursor > self.local_admitted && frame.cursor <= through)
            .filter_map(|frame| frame.record.as_ref().map(|encoded| (frame.cursor, encoded.clone(), frame.local_scope.clone())))
            .collect::<Vec<_>>();
        let mut admitted = Vec::with_capacity(frames.len());
        for (cursor, encoded, scope) in frames {
            let mut request: dokime_domain::model::records::publication::RecordPublishRequest =
                serde_json::from_str(&encoded).map_err(|error| Refusal::of(error.to_string()))?;
            // The host's own record types, such as `process.exited`, are
            // registered wherever a run is, beside the template's.
            let contract = contracts.iter().find(|contract| contract.record_type == request.record_type).cloned().or_else(|| {
                crate::records::builtin_record_contracts().into_iter().find(|contract| contract.record_type == request.record_type)
            });
            let contract = contract.as_ref();
            let version = contract.map_or(1, |contract| contract.schema_version);
            request.scope = scope.clone();
            let mut record = request.into_record(
                dokime_domain::model::identity::RecordId::from_seed(format!("{}:local:{cursor}", self.request.execution_token)),
                version,
                admitted_at.clone(),
                source,
            );
            record.details = crate::records::validate_record(contract, &record, dokime_domain::model::validation::ValidationMode::Strict)
                .map_err(|error| Refusal::of(error.to_string()))?;
            admitted.push((record, scope));
        }
        for (record, scope) in admitted {
            let ordinal = self.record_ordinal.checked_add(1).ok_or_else(|| Refusal::of("local record ordinal overflows"))?;
            self.deliver_captured(ordinal, &record, source, scope.as_ref());
        }
        self.local_admitted = self.local_admitted.max(through);
        Ok(())
    }

    pub(crate) fn settle_local_inputs(
        &mut self, source: &str, contracts: &[crate::records::RecordContract], admitted_at: &dokime_domain::model::time::Timestamp,
    ) -> Result<(), Refusal> {
        self.admit_local_inputs(self.next_cursor.saturating_sub(1), source, contracts, admitted_at)?;
        self.settle_record_subscriptions();
        Ok(())
    }

    pub(crate) fn drain_after(&mut self, cursor: u64) -> Result<EvidenceBatch, Refusal> {
        if cursor.saturating_add(1) < self.oldest_cursor {
            return Err(Refusal::of(format!("execution cursor {cursor} expired before buffered cursor {}", self.oldest_cursor)));
        }
        self.confirmed_cursor = self.confirmed_cursor.max(cursor);
        if let Some(progress) = self.deferred_progress.take() {
            self.push(BufferedFrame::progress(progress));
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
            if page_fits(&reply) || (alone && Frame::request_fits::<crate::transport::EvidenceBatch>(&reply)) {
                if frame.step_attempt.is_some() || frame.step_begin.is_some() || frame.capture_phase.is_some() {
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
                return Err(Refusal::of("one execution output item exceeds the Harmos transport frame"));
            }
            break;
        }

        // Newly assembled pages retire their artifact chunks from the
        // back-pressure backlog. Frames already covered by an earlier
        // acknowledgment are skipped, so replaying a repeated cursor cannot
        // subtract the same bytes twice.
        let previously_acknowledged = self.acknowledged_cursor;
        if reply.cursor > previously_acknowledged {
            for frame in self.frames.iter().filter(|frame| frame.cursor > previously_acknowledged && frame.cursor <= reply.cursor) {
                if let Some(chunk) = &frame.artifact_chunk {
                    self.buffered_artifact_bytes = self.buffered_artifact_bytes.saturating_sub(chunk.bytes.len() as u64);
                }
            }
            self.acknowledged_cursor = reply.cursor;
            // The producer parked on this execution's gate may now find it
            // open; wake it up to re-check the gate predicate.
            self.artifact_condvar.notify_all();
        }

        reply.record_subscriptions_settling = self.subscriptions_settling && reply.cursor.saturating_add(1) == self.next_cursor;
        Ok(reply)
    }
}

// An Evicted Run Settles Every Exchange Still Waiting On It.
impl Drop for ExecutionState {
    fn drop(&mut self) {
        self.artifact_condvar.notify_all();
        self.changed.notify_waiters();
        self.settled.notify_waiters();
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
        Self { record: Some(record), ..Self::default() }
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
                    if outcome.passed { crate::execution::log::Level::Info } else { crate::execution::log::Level::Error },
                    message,
                )
            }),
            outcome: Some(outcome),
            ..Self::default()
        }
    }
}

pub(crate) fn lock(executions: &Executions) -> MutexGuard<'_, BTreeMap<String, ExecutionState>> {
    executions.lock().unwrap_or_else(PoisonError::into_inner)
}

// Waits Without Polling Until `check` Settles The Run's Exchange; `gone` Answers An Evicted Run.
// The waiter registers before each check, so a settlement between check and wait still wakes it.
pub(crate) async fn settled<T>(
    executions: &Executions, token: &str, gone: impl Fn() -> T, check: impl Fn(&ExecutionState) -> Option<T>,
) -> T {
    let mut signal = match lock(executions).get(token) {
        Some(state) => Arc::clone(&state.settled),
        None => return gone(),
    };
    loop {
        let current = {
            let notified = signal.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let current = {
                let live = lock(executions);
                let Some(state) = live.get(token) else {
                    return gone();
                };
                if let Some(outcome) = check(state) {
                    return outcome;
                }
                (!Arc::ptr_eq(&state.settled, &signal)).then(|| Arc::clone(&state.settled))
            };
            if current.is_none() {
                notified.await;
            }
            current
        };
        // A Reused Token Replaced The Run; Wait On Its Own Signal.
        if let Some(current) = current {
            signal = current;
        }
    }
}

// Host Admission Adds Receipt Time And Full Ownership To Locally Produced Rows.
// Canonical Payload Bytes Are Internal And May Be Absent After Wire Decoding.
fn sample_difference(
    previous: &crate::telemetry::TelemetrySample, incoming: &crate::telemetry::TelemetrySample, local: bool,
) -> Option<&'static str> {
    let scope_matches = if local {
        match (&previous.scope, &incoming.scope) {
            (Some(left), Some(right)) => {
                left.test_cycle_id == right.test_cycle_id
                    && left.test_execution_id == right.test_execution_id
                    && left.origin == right.origin
            }
            (None, None) => true,
            _ => false,
        }
    } else {
        previous.scope == incoming.scope
    };
    [
        ("sample_id", previous.sample_id == incoming.sample_id),
        ("stream", previous.stream == incoming.stream),
        ("schema_version", previous.schema_version == incoming.schema_version),
        ("sequence", previous.sequence == incoming.sequence),
        ("timestamp", previous.timestamp == incoming.timestamp),
        ("received_at", local || previous.received_at == incoming.received_at),
        ("source", previous.source == incoming.source),
        ("scope", scope_matches),
        ("values", values_equivalent(&previous.values, &incoming.values)),
        ("summary", previous.summary == incoming.summary),
        ("metadata", previous.metadata == incoming.metadata),
        (
            "storage_payload",
            match (&previous.storage_payload, &incoming.storage_payload) {
                (Some(left), Some(right)) => left == right,
                _ => true,
            },
        ),
    ]
    .into_iter()
    .find_map(|(field, matches)| (!matches).then_some(field))
}

// A typed sample's absent optional field and its null are the same datum: the
// live sample omits it, its stored projection names it null.
fn values_equivalent(
    left: &BTreeMap<String, dokime_domain::model::values::Value>, right: &BTreeMap<String, dokime_domain::model::values::Value>,
) -> bool {
    left.keys().chain(right.keys()).all(|key| value_equivalent(left.get(key), right.get(key)))
}

fn value_equivalent(left: Option<&dokime_domain::model::values::Value>, right: Option<&dokime_domain::model::values::Value>) -> bool {
    use dokime_domain::model::values::Value;
    match (left, right) {
        (None | Some(Value::Null), None | Some(Value::Null)) => true,
        (Some(Value::Object(left)), Some(Value::Object(right))) => values_equivalent(left, right),
        (Some(Value::Array(left)), Some(Value::Array(right))) => {
            left.len() == right.len() && left.iter().zip(right).all(|(left, right)| value_equivalent(Some(left), Some(right)))
        }
        (left, right) => left == right,
    }
}

fn page_fits(page: &crate::transport::EvidenceBatch) -> bool {
    use harmos::Message;
    page.encoded_size() <= crate::transport::protocol::MAX_EVIDENCE_PAGE_BYTES
        && Frame::request_fits::<crate::transport::EvidenceBatch>(page)
}

#[cfg(test)]
mod tests {
    use super::sample_difference;
    use crate::telemetry::TelemetrySample;
    use dokime_domain::model::execution::RecordScope;
    use dokime_domain::model::identity::Identifier;
    use dokime_domain::model::identity::TestExecutionId;
    use dokime_domain::model::time::Timestamp;
    use dokime_domain::model::values::Value;

    #[test]
    fn output_wakes_readers() {
        let (cancel, _) = tokio::sync::watch::channel(false);
        let mut state = super::ExecutionState::new(Default::default(), cancel);
        let changed = std::sync::Arc::clone(&state.changed);
        let mut first = Box::pin(changed.notified());
        let mut second = Box::pin(changed.notified());
        first.as_mut().enable();
        second.as_mut().enable();

        state.push(super::BufferedFrame::log(crate::execution::log::Level::Info, String::new()));
        let mut context = std::task::Context::from_waker(std::task::Waker::noop());
        assert!(std::future::Future::poll(first.as_mut(), &mut context).is_ready());
        assert!(std::future::Future::poll(second.as_mut(), &mut context).is_ready());
    }

    fn progress(completed: u64) -> crate::execution::ExecutionProgress {
        crate::execution::ExecutionProgress {
            phase: "flood".into(),
            completed,
            total: u64::MAX,
            elapsed_ms: 0,
            remaining_ms: None,
        }
    }

    // Drains Every Retained Frame From `cursor`, Returning The Records, The Last Progress Seen And The Final Cursor.
    fn drain_all(state: &mut super::ExecutionState, mut cursor: u64) -> (Vec<String>, Option<u64>, u64) {
        let (mut records, mut latest) = (Vec::new(), None);
        loop {
            let reply = state.drain_after(cursor).unwrap();
            if reply.cursor == cursor {
                return (records, latest, cursor);
            }
            records.extend(reply.records);
            latest = reply.progress.map(|progress| progress.completed).or(latest);
            cursor = reply.cursor;
        }
    }

    #[test]
    fn progress_floods_never_evict_evidence_frames() {
        let (cancel, _) = tokio::sync::watch::channel(false);
        let mut state = super::ExecutionState::new(Default::default(), cancel);
        // Evidence Interleaved With A Progress Flood Keeps Every Frame And Only The Latest Progress.
        for index in 0..super::RETAINED_FRAMES as u64 {
            state.push(super::BufferedFrame::record(format!("record-{index}")));
            for completed in 0..64 {
                state.push(super::BufferedFrame::progress(progress(index * 64 + completed)));
            }
        }
        let (records, latest, cursor) = drain_all(&mut state, 0);
        let expected = (0..super::RETAINED_FRAMES).map(|index| format!("record-{index}")).collect::<Vec<_>>();
        assert_eq!(records, expected);
        assert_eq!(latest, Some(super::RETAINED_FRAMES as u64 * 64 - 1));

        // A Ring Full Of Unconfirmed Evidence Defers Progress Until The Host Confirms A Drain.
        for index in 0..super::RETAINED_FRAMES {
            state.push(super::BufferedFrame::record(format!("late-{index}")));
        }
        for completed in 0..100_000 {
            state.push(super::BufferedFrame::progress(progress(completed)));
        }
        let (records, latest, _) = drain_all(&mut state, cursor);
        assert_eq!(records.len(), super::RETAINED_FRAMES);
        assert_eq!(records.first().map(String::as_str), Some("late-0"));
        assert_eq!(latest, Some(99_999));
    }

    #[test]
    fn lease_gaps_stay_bounded_per_stream() {
        let (cancel, _) = tokio::sync::watch::channel(false);
        let mut state = super::ExecutionState::new(Default::default(), cancel);
        let gap = |stream: &str, reason: usize| crate::telemetry::LeaseGap {
            stream: Identifier::new(stream).unwrap(),
            owner: "owner".into(),
            observed_at: Timestamp::new("2000-01-01T00:00:00Z"),
            reason: reason.to_string(),
        };
        for reason in 0..10_000 {
            state.interrupted(gap("busy", reason));
        }
        state.interrupted(gap("quiet", 0));
        let gaps = state.all_gaps();
        let busy = gaps.iter().filter(|held| held.stream.as_str() == "busy").map(|held| held.reason.clone()).collect::<Vec<_>>();
        let earliest = (0..super::RETAINED_STREAM_GAPS - 1).map(|reason| reason.to_string()).collect::<Vec<_>>();
        assert_eq!(busy.len(), super::RETAINED_STREAM_GAPS);
        assert_eq!(busy[..super::RETAINED_STREAM_GAPS - 1], earliest);
        assert_eq!(busy.last().map(String::as_str), Some("9999"));
        assert_eq!(gaps.last().map(|held| held.stream.as_str()), Some("quiet"));
        assert_eq!(state.gap_counts().get("busy"), Some(&10_000));
        assert_eq!(state.gap_counts().get("quiet"), Some(&1));
    }

    #[test]
    fn settlement_counts_each_live_sample_confirmation_once() {
        let (cancel, _) = tokio::sync::watch::channel(false);
        let mut state = super::ExecutionState::new(Default::default(), cancel);
        let stream = Identifier::new("reading").unwrap();
        let sample = TelemetrySample::new(crate::model::identity::new_id(), stream, 1, crate::model::time::now_utc(), "fixture");
        assert!(state.receive(sample.clone()).unwrap());
        assert!(!state.receive(sample.clone()).unwrap());
        assert_eq!(state.settlement_progress(), 1);

        state.begin_record_settle();
        let mut changed = sample.clone();
        changed.values.insert("value".into(), Value::from(2.0));
        assert!(state.receive(changed).is_err());
        assert_eq!(state.settlement_progress(), 1);
        assert!(!state.receive(sample.clone()).unwrap());
        assert_eq!(state.settlement_progress(), 2);
        assert!(!state.receive(sample.clone()).unwrap());
        assert_eq!(state.settlement_progress(), 2);

        let mut late = sample;
        late.sample_id = crate::model::identity::new_id();
        late.sequence += 1;
        assert!(state.receive(late.clone()).unwrap());
        assert_eq!(state.settlement_progress(), 3);
        assert!(!state.receive(late.clone()).unwrap());
        assert_eq!(state.settlement_progress(), 4);
        assert!(!state.receive(late).unwrap());
        assert_eq!(state.settlement_progress(), 4);
    }

    #[test]
    fn seven_day_records_keep_bounded_receipts_and_exact_evidence() {
        let (cancel, _) = tokio::sync::watch::channel(false);
        let mut state = super::ExecutionState::new(Default::default(), cancel);
        let scope = RecordScope::default();
        let record = |ordinal| {
            dokime_domain::model::records::evidence::Record::new(
                dokime_domain::model::identity::RecordId::from_seed(format!("record:{ordinal}")),
                Timestamp::new("2000-01-01T00:00:00Z"),
                Identifier::new("reading").unwrap(),
                1,
                "fixture",
                dokime_domain::model::records::RecordSeverity::Info,
            )
        };
        for ordinal in 1..=7 * 1440_u64 {
            let row = record(ordinal);
            assert!(state.deliver_captured(ordinal, &row, "host", Some(&scope)));
            assert!(!state.deliver_captured(ordinal, &row, "host", Some(&scope)));
            assert_eq!(state.record_ordinal, ordinal);
            assert!(state.recent_records.len() <= super::RECENT_STREAM_SAMPLES);
        }
        assert_eq!(state.captured_records.len(), 7 * 1440);
        assert_eq!(state.captured_records.retained_rows(), 0);
        assert_eq!(state.captured_records.iter().count(), 7 * 1440);
        let captured = state.captured_records.clone();
        assert!(!state.deliver_captured(1, &record(1), "host", Some(&scope)));
        assert!(state.cancelled());
        assert_eq!(captured.len(), 7 * 1440);
    }

    #[test]
    fn seven_day_stream_bounds_dedup_and_rejects_out_of_order_rows() {
        let (cancel, _) = tokio::sync::watch::channel(false);
        let mut state = super::ExecutionState::new(Default::default(), cancel);
        let stream = Identifier::new("reading").unwrap();
        let timestamp = Timestamp::new("2000-01-01T00:00:00Z");
        let first = TelemetrySample::new(
            dokime_domain::model::identity::TelemetrySampleId::from_seed("minute:0"),
            stream.clone(),
            0,
            timestamp.clone(),
            "fixture",
        );
        for minute in 0..7 * 1440_u64 {
            let sample = TelemetrySample::new(
                dokime_domain::model::identity::TelemetrySampleId::from_seed(format!("minute:{minute}")),
                stream.clone(),
                minute,
                timestamp.clone(),
                "fixture",
            );
            assert!(state.receive(sample.clone()).unwrap());
            assert!(!state.receive(sample).unwrap());
            let held = &state.sample_streams["reading"];
            assert!(held.recent.len() <= super::RECENT_STREAM_SAMPLES);
            assert_eq!(held.high_water, Some(minute));
        }

        assert_eq!(state.sample_streams.len(), 1);
        let receipts = state.sample_receipts();
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0].stream, "reading");
        let last = dokime_domain::model::identity::TelemetrySampleId::from_seed(format!("minute:{}", 7 * 1440 - 1));
        assert_eq!(receipts[0].sample_id, last.as_str());
        assert_eq!(state.stream_rows.retained_rows(), 0);
        assert_eq!(state.stream_rows.len(), 7 * 1440);
        assert_eq!(state.receive(first.clone()).unwrap_err(), "sample stream sequence is out of order");
        let mut reused_sequence = first;
        reused_sequence.sample_id = dokime_domain::model::identity::TelemetrySampleId::from_seed("other");
        reused_sequence.sequence = 7 * 1440 - 1;
        assert_eq!(state.receive(reused_sequence).unwrap_err(), "sample stream sequence is out of order");
        let oldest_recent = 7 * 1440 - super::RECENT_STREAM_SAMPLES as u64;
        let recent = TelemetrySample::new(
            dokime_domain::model::identity::TelemetrySampleId::from_seed(format!("minute:{oldest_recent}")),
            stream,
            oldest_recent,
            timestamp,
            "fixture",
        );
        assert!(!state.receive(recent.clone()).unwrap());
        let mut conflicting = recent.clone();
        conflicting.values.insert("value".into(), Value::from(2.0));
        assert!(state.receive(conflicting).unwrap_err().starts_with("sample identity was reused"));
        assert_eq!(state.produced("reading"), 7 * 1440);

        let progress = state.settlement_progress();
        state.subscriptions_settling = true;
        assert!(!state.receive(recent.clone()).unwrap());
        assert_eq!(state.settlement_progress(), progress + 1);
        assert!(!state.receive(recent).unwrap());
        assert_eq!(state.settlement_progress(), progress + 1);

        let staged = state.take_staged();
        assert_eq!(staged.retained_rows(), 0);
        for (minute, sample) in staged.iter().enumerate() {
            assert_eq!(sample.sequence, minute as u64);
        }
        drop(staged);
        for receipt in &state.sample_streams["reading"].recent {
            assert_eq!(receipt.row.read().unwrap().sample_id.as_str(), receipt.sample_id);
        }
    }

    #[test]
    fn sample_replay_preserves_producer_identity() {
        let mut original = TelemetrySample::new(
            crate::model::identity::new_id(),
            Identifier::new("reading").unwrap(),
            1,
            crate::model::time::now_utc(),
            "fixture",
        )
        .with_storage_payload(vec![1, 2]);
        original.scope = Some(RecordScope {
            test_execution_id: Some(crate::model::identity::new_id::<TestExecutionId>()),
            ..Default::default()
        });
        let wire: TelemetrySample = serde_json::from_value(serde_json::to_value(&original).unwrap()).unwrap();
        assert!(sample_difference(&original, &wire, false).is_none());
        let mut admitted = wire.clone();
        admitted.received_at = Timestamp::new("2000-01-01T00:00:00Z");
        admitted.scope.as_mut().unwrap().session_id = Some(crate::model::identity::new_id::<dokime_domain::model::identity::SessionId>());
        assert!(sample_difference(&original, &admitted, true).is_none());
        assert!(sample_difference(&original, &admitted, false).is_some());
        let mut changed = wire.clone();
        changed.values.insert("value".into(), Value::from(2.0));
        assert!(sample_difference(&original, &changed, true).is_some());
        // The stored projection names an absent optional field null.
        let mut projected = wire.clone();
        projected.values.insert("probe_error".into(), Value::Null);
        assert!(sample_difference(&original, &projected, false).is_none());
        let mut changed = original.clone();
        changed.storage_payload = Some(vec![3]);
        assert!(sample_difference(&original, &changed, true).is_some());
        let mut changed = wire;
        changed.scope = Some(RecordScope {
            test_execution_id: Some(crate::model::identity::new_id::<TestExecutionId>()),
            ..Default::default()
        });
        assert!(sample_difference(&original, &changed, true).is_some());
    }

    #[test]
    fn local_sample_replay_keeps_automatic_timestamp() {
        let request = crate::transport::protocol::RunExecution {
            execution_token: "fixture".into(),
            test_cycle_id: crate::model::identity::new_id::<dokime_domain::model::identity::TestCycleId>().to_string(),
            test_execution_id: crate::model::identity::new_id::<TestExecutionId>().to_string(),
            template_identifier: "fixture".into(),
            workflow: "fixture".into(),
            parameters_json: "{}".into(),
            secrets_json: "{}".into(),
            mode: "automatic".into(),
        };
        let (cancel, _) = tokio::sync::watch::channel(false);
        let mut state = super::ExecutionState::new(request, cancel);
        state.local_inputs = true;
        for explicit in [None, Some(Value::Number(42.0))] {
            let mut sample = TelemetrySample::new(
                crate::model::identity::new_id(),
                Identifier::new("reading").unwrap(),
                state.produced("reading"),
                Timestamp::new("2000-01-01T00:00:00Z"),
                "fixture",
            );
            sample.values.clear();
            if let Some(value) = &explicit {
                sample.values.insert("timestamp".into(), value.clone());
            }
            state.local_sample(sample).unwrap();
            let admitted = state.take_local_samples().pop().unwrap();
            assert_eq!(admitted.values["timestamp"], explicit.unwrap_or(Value::Number(946684800.0)));
            let mut replay = admitted.clone();
            replay.received_at = crate::model::time::now_utc();
            state.validate_sample(&replay).unwrap();
            replay.values.insert("timestamp".into(), Value::Number(0.0));
            assert!(state.validate_sample(&replay).unwrap_err().ends_with(": values.timestamp"));
        }
    }
}
