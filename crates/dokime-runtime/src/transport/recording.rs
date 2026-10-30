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
use std::sync::{Arc, Mutex};

// Third Party Imports

// Local Imports
use super::protocol::ArtifactChunk;
use super::{ExecutionTransport, Journal, TransportFuture};
use crate::artifacts::ArtifactSink;
use crate::execution::lease::{LeaseHold, LeaseRelease};
use crate::execution::log::Level;
use crate::execution::{ExecutionProgress, ProgressSink};
use crate::prompts::{PromptRequest, PromptResponse};
use crate::records::RecordSource;
use crate::telemetry::{LeaseGap, LeaseRequest, SampleSink, TelemetrySample};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::records::evidence::Record;
use dokime_domain::model::records::publication::RecordPublishRequest;
use dokime_domain::model::traceability::TraceabilitySection;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, Default)]
pub struct RecordingTransport {
    answers: VecDeque<PromptResponse>,
    prompts: Vec<PromptRequest>,
    records: Arc<Mutex<Vec<RecordPublishRequest>>>,
    diagnostics: Arc<Mutex<Vec<(Level, String)>>>,
    progress: Arc<Mutex<Vec<ExecutionProgress>>>,
    steps: Vec<Identifier>,
    step_attempts: Vec<dokime_domain::model::execution::result::TestStepAttempt>,
    traceability: Mutex<Vec<TraceabilitySection>>,
    documents: Mutex<Vec<crate::document::DocumentAddition>>,
    leasable: Vec<Identifier>,
    leases: Vec<Identifier>,
    gaps: Vec<LeaseGap>,
    released: Arc<Mutex<Vec<String>>>,
}

#[derive(Debug, Default)]
struct RecordingLeaseRelease {
    released: Arc<Mutex<Vec<String>>>,
}

#[derive(Debug, Default)]
pub struct RecordingSink {
    chunks: Mutex<Vec<ArtifactChunk>>,
    samples: Mutex<Vec<TelemetrySample>>,
    delivered: Mutex<Vec<Record>>,

    #[cfg(feature = "native")]
    changed: Arc<tokio::sync::Notify>,
}

struct RecordingJournal {
    records: Arc<Mutex<Vec<RecordPublishRequest>>>,
    diagnostics: Arc<Mutex<Vec<(Level, String)>>>,
}

struct RecordingProgressSink {
    updates: Arc<Mutex<Vec<ExecutionProgress>>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RecordingTransport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn answering(responses: impl IntoIterator<Item = PromptResponse>) -> Self {
        Self {
            answers: responses.into_iter().collect(),
            ..Self::default()
        }
    }

    pub fn answer(&mut self, response: PromptResponse) {
        self.answers.push_back(response);
    }

    pub fn prompts(&self) -> &[PromptRequest] {
        &self.prompts
    }

    pub fn records(&self) -> Vec<RecordPublishRequest> {
        self.records.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
    }

    pub fn diagnostics(&self) -> Vec<(Level, String)> {
        self.diagnostics.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
    }

    pub fn step_attempts(&self) -> &[dokime_domain::model::execution::result::TestStepAttempt] {
        &self.step_attempts
    }

    pub fn steps_reached(&self) -> &[Identifier] {
        &self.steps
    }

    pub fn traceability(&self) -> Vec<TraceabilitySection> {
        self.traceability.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
    }

    pub fn leasing(streams: impl IntoIterator<Item = Identifier>) -> Self {
        Self {
            leasable: streams.into_iter().collect(),
            ..Self::default()
        }
    }

    pub fn interrupt(&mut self, gap: LeaseGap) {
        self.gaps.push(gap);
    }
}

impl Journal for RecordingJournal {
    fn log(&self, level: Level, message: String) {
        self.diagnostics.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push((level, message));
    }

    fn record(&self, record: RecordPublishRequest) {
        self.records.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(record);
    }
}

impl ExecutionTransport for RecordingTransport {
    fn execution(&self) -> &str {
        "dokime.recording"
    }

    fn progress(&self) -> Arc<dyn ProgressSink> {
        Arc::new(RecordingProgressSink { updates: Arc::clone(&self.progress) })
    }

    fn prompt<'a>(&'a mut self, request: PromptRequest) -> TransportFuture<'a, crate::Result<PromptResponse>> {
        let answered = self.answers.pop_front();
        let identifier = request.prompt_id.to_string();
        self.prompts.push(request);
        Box::pin(async move { answered.ok_or(crate::Error::PromptRefused(format!("no answer staged for {identifier}"))) })
    }

    fn lease<'a>(&'a mut self, request: LeaseRequest) -> TransportFuture<'a, crate::Result<LeaseHold>> {
        self.leases.push(request.stream.clone());
        let granted = self.leasable.contains(&request.stream);
        let released = Arc::clone(&self.released);
        Box::pin(async move {
            if !granted {
                return Err(crate::Error::LeaseRefused(format!("no stream {} is leasable in this test", request.stream)));
            }

            Ok(LeaseHold {
                lease_token: request.lease_token,
                release: Arc::new(RecordingLeaseRelease { released }) as Arc<dyn LeaseRelease>,
            })
        })
    }

    fn gaps(&self) -> Vec<LeaseGap> {
        self.gaps.clone()
    }

    fn emit(&mut self, record: RecordPublishRequest) {
        self.records.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(record);
    }

    fn journal(&self) -> Arc<dyn Journal> {
        Arc::new(RecordingJournal {
            records: Arc::clone(&self.records),
            diagnostics: Arc::clone(&self.diagnostics),
        })
    }

    fn log(&self, level: Level, message: String) {
        self.diagnostics.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push((level, message));
    }

    fn finish_step(&mut self, attempt: dokime_domain::model::execution::result::TestStepAttempt) {
        self.step_attempts.push(attempt);
    }

    fn capture_phase(&mut self, _phase: dokime_domain::model::execution::CapturePhase) {}

    fn begin_step<'a>(
        &'a mut self, step: Identifier, _start: dokime_domain::model::execution::result::StepAttemptStart,
    ) -> TransportFuture<'a, Result<(), crate::diagnostics::Diagnostic>> {
        self.steps.push(step);
        Box::pin(async { Ok(()) })
    }

    fn traceability(&self, sections: Vec<TraceabilitySection>) {
        let mut captured = self.traceability.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        for section in sections {
            dokime_domain::model::traceability::upsert_section(&mut captured, section);
        }
    }

    fn document(&self, addition: crate::document::DocumentAddition) {
        self.documents.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(addition);
    }
}

impl ProgressSink for RecordingProgressSink {
    fn update(&self, progress: ExecutionProgress) {
        self.updates.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(progress);
    }
}

impl RecordingSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn chunks(&self) -> Vec<ArtifactChunk> {
        self.chunks.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
    }

    pub fn rows(&self, stream: &str) -> Vec<BTreeMap<String, Value>> {
        self.published().into_iter().filter(|sample| sample.stream.as_str() == stream).map(|sample| sample.values).collect()
    }

    pub fn published(&self) -> Vec<TelemetrySample> {
        self.samples.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
    }

    pub fn deliver(&self, record: Record) {
        self.delivered.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(record);
        #[cfg(feature = "native")]
        self.changed.notify_waiters();
    }

    pub fn assembled(&self, name: &str) -> Vec<u8> {
        let mut bytes = Vec::new();
        for chunk in self.chunks().into_iter().filter(|chunk| chunk.name == name) {
            if chunk.replace {
                bytes.clear();
            }
            bytes.extend(chunk.bytes);
        }

        bytes
    }
}

impl SampleSink for RecordingSink {
    fn publish<'a>(
        &'a self, stream: &'a Identifier, values: BTreeMap<String, Value>, storage_payload: Option<Vec<u8>>,
    ) -> crate::transport::TransportFuture<'a, Result<(), String>> {
        Box::pin(async move {
            let mut samples = self.samples.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let sequence = samples.iter().filter(|sample| sample.stream == *stream).count() as u64;
            let mut sample = TelemetrySample::new(
                crate::model::identity::new_id(),
                stream.clone(),
                sequence,
                crate::model::time::now_utc(),
                "recording",
            );
            sample.values = values;
            sample.storage_payload = storage_payload;
            samples.push(sample);
            Ok(())
        })
    }

    fn produced(&self, stream: &str) -> u64 {
        self.samples
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter(|sample| sample.stream.as_str() == stream)
            .count() as u64
    }
}

impl RecordSource for RecordingSink {
    #[cfg(feature = "native")]
    fn changed(&self) -> Arc<tokio::sync::Notify> {
        Arc::clone(&self.changed)
    }

    #[cfg(feature = "native")]
    fn closed(&self) -> bool {
        false
    }

    fn delivered(&self, index: usize) -> Option<Record> {
        self.delivered.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get(index).cloned()
    }
}

impl ArtifactSink for RecordingSink {
    fn push(&self, chunk: ArtifactChunk) {
        self.chunks.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(chunk);
    }
}

impl LeaseRelease for RecordingLeaseRelease {
    fn release(&self, lease_token: &str) {
        self.released.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(lease_token.to_owned());
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
