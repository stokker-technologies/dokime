/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod admission;

/* === Re-Exports === */
pub(crate) use crate::runtime::operations::provider::ProviderCycleRequest;

/* === Imports === */
// Standard Library Imports
use std::{collections::BTreeMap, future::Future, pin::Pin, sync::Arc};
// Third Party Imports

// Local Imports
use crate::{
    evidence_runtime::{ArtifactUploadInput, PublishReceipt, RecordPublishRequest},
    execution::{CycleStatus, CycleView, TestExecution},
    execution_runtime::authoring::{ExecutionFileTarget, SidecarStepContext},
    handle::RuntimeHandle,
    state::RuntimeState,
};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::PromptId;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use crate::domains::{ExecutionProviderTarget, SidecarExecutionBinding};
use crate::prompts::{PromptRequest, PromptResponse};
use crate::telemetry::TelemetryBatch;
use crate::transport::{ProviderCycleOutcome, ProviderExecutionFailure};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type ExecutionFixtureFuture<'a> = Pin<
    Box<
        dyn Future<Output = std::result::Result<ProviderCycleOutcome, ProviderExecutionFailure>>
            + Send
            + 'a,
    >,
>;

/* === Structs === */
#[derive(Clone)]
pub struct ExecutionFixtureContext {
    pub(crate) runtime: RuntimeHandle,
    pub(crate) request_id: Option<dokime_domain::model::identity::CommandId>,
    cancellation: Option<tokio::sync::watch::Receiver<bool>>,
    pub(crate) finalization: super::run::Finalization,
}

#[derive(Clone)]
pub(crate) struct ProviderExecutorRegistry {
    fixtures: BTreeMap<String, Arc<dyn ExecutionFixture>>,
    sidecar: Arc<dyn ExecutionFixture>,
}

pub(crate) struct ProviderCycleAdmission {
    pub(crate) job: ProviderCycleRequest,
}

/* === Enums === */

/* === Traits === */

pub trait ExecutionFixture: Send + Sync + 'static {
    fn run_cycle<'a>(
        &'a self,
        context: ExecutionFixtureContext,
        cycle: &'a CycleView,
        target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a>;

    /// Refuses One Execution's Final Report Before It Is Materialized.
    ///
    /// A refusal is a permanent finalization failure: the execution is never
    /// sealed while this fixture serves the runtime.
    fn refuse_final_report(&self, _execution: &TestExecution) -> Option<String> {
        None
    }
}

/* === Implementations === */
impl ExecutionFixtureContext {
    pub(crate) fn new(runtime: RuntimeHandle) -> Self {
        Self {
            runtime,
            request_id: None,
            cancellation: None,
            finalization: Default::default(),
        }
    }

    pub(crate) fn with_request(mut self, request_id: dokime_domain::model::identity::CommandId) -> Self {
        self.request_id = Some(request_id);
        self
    }

    pub(crate) fn with_finalization(mut self, finalization: super::run::Finalization) -> Self {
        self.finalization = finalization;
        self
    }

    pub(crate) fn with_cancellation(
        mut self,
        cancellation: tokio::sync::watch::Receiver<bool>,
    ) -> Self {
        self.cancellation = Some(cancellation);
        self
    }

    /// True once the runtime shuts down, the work is cancelled, or an operator
    /// stopped the cycle run this context serves.
    pub fn is_cancelled(&self) -> bool {
        self.runtime.is_shutting_down()
            || self
                .cancellation
                .as_ref()
                .is_some_and(|cancelled| *cancelled.borrow())
            || self.request_id.as_ref().is_some_and(|request_id| {
                self.runtime
                    .state_read()
                    .run_requests
                    .get(request_id)
                    .is_some_and(|request| request.summary.stop_requested)
            })
    }

    /// Wait until the same cancellation conditions reported by `is_cancelled` hold.
    /// Closing the provider's cancellation channel also ends the wait.
    pub async fn cancelled(&self) {
        let mut cancellation = self.cancellation.clone();
        let mut shutdown = self.runtime.shutdown_requested.subscribe();
        let mut state = self.runtime.watch_state();
        loop {
            if self.is_cancelled() {
                return;
            }
            tokio::select! {
                () = async {
                    if let Some(cancelled) = cancellation.as_mut() {
                        let _ = cancelled.wait_for(|cancelled| *cancelled).await;
                    } else {
                        std::future::pending::<()>().await;
                    }
                } => return,
                _ = shutdown.wait_for(|shutdown| *shutdown) => return,
                changed = state.next() => {
                    if changed.is_none() {
                        return;
                    }
                }
            }
        }
    }

    /// True once an operator stopped this one execution of a cycle run. The
    /// provider cancels it cooperatively and keeps running its siblings.
    pub fn execution_stop_requested(&self, test_execution_id: &TestExecutionId) -> bool {
        let Some(request_id) = &self.request_id else {
            return false;
        };
        self.runtime
            .state_read()
            .run_requests
            .get(request_id)
            .is_some_and(|request| request.stopped_execution_ids.contains(test_execution_id))
    }

    /// Wait until this one execution is stopped. Select with `cancelled` when
    /// the provider also needs to heed cancellation of the whole cycle run.
    pub async fn execution_stopped(&self, test_execution_id: &TestExecutionId) {
        let Some(request_id) = &self.request_id else {
            return std::future::pending().await;
        };
        let mut state = self.runtime.watch_state();
        let _ = state
            .wait_until(|snapshot| {
                snapshot
                    .state()
                    .run_requests
                    .get(request_id)
                    .is_some_and(|request| request.stopped_execution_ids.contains(test_execution_id))
            })
            .await;
    }

    pub async fn cycle_lifecycle(&self, test_cycle_id: &TestCycleId) -> Option<CycleStatus> {
        self.runtime.cycle_lifecycle(test_cycle_id).await
    }

    pub fn step_context_with_source(
        &self,
        scope: RecordScope,
        source: impl Into<String>,
    ) -> SidecarStepContext {
        SidecarStepContext::with_source(self.runtime.clone(), scope, source)
    }

    pub(crate) async fn capture_publication_scope(
        &self,
        scope: RecordScope,
    ) -> crate::runtime::Result<RecordScope> {
        self.runtime.capture_publication_scope(scope).await
    }

    pub(crate) async fn begin_captured_artifact_upload(
        &self,
        input: crate::artifacts::ArtifactUploadInput,
        created_at: dokime_domain::model::time::Timestamp,
    ) -> crate::runtime::Result<crate::runtime::storage::artifacts::ArtifactUpload> {
        self.runtime
            .begin_captured_artifact_upload(input, created_at)
            .await
    }

    pub(crate) async fn capture_phase(
        &self,
        scope: RecordScope,
        phase: dokime_domain::model::execution::CapturePhase,
    ) -> crate::runtime::Result<()> {
        let _order = self.runtime.telemetry_admission.lock().await;
        self.runtime
            .commit(crate::transactions::SetCapturePhase {
                scope,
                phase,
                occurred_at: crate::model::time::now_utc(),
            })
            .await
            .map(|_| ())
            .map_err(|error| crate::unavailable_rejection(error.to_string()))
    }

    /// Admit one attempt of a declared step; its outcome follows through
    /// [`Self::record_step_attempt`].
    pub async fn begin_step_attempt(
        &self,
        scope: RecordScope,
        start: crate::execution::StepAttemptStart,
    ) -> crate::runtime::Result<()> {
        let _order = self.runtime.telemetry_admission.lock().await;
        self.runtime
            .commit(crate::transactions::BeginStepAttempt { scope, start })
            .await
            .map(|_| ())
            .map_err(|error| crate::unavailable_rejection(error.to_string()))
    }

    /// Record how an admitted step attempt ended; this is the step's outcome.
    pub async fn record_step_attempt(
        &self,
        scope: RecordScope,
        attempt: crate::execution::TestStepAttempt,
    ) -> crate::runtime::Result<()> {
        let _order = self.runtime.telemetry_admission.lock().await;
        self.runtime
            .commit(crate::transactions::RecordStepAttempt { scope, attempt })
            .await
            .map(|_| ())
            .map_err(|error| crate::unavailable_rejection(error.to_string()))
    }

    pub async fn update_execution_phase(
        &self,
        scope: RecordScope,
        identifier: Identifier,
        label: String,
        status: String,
    ) -> crate::runtime::Result<()> {
        self.runtime
            .update_execution_phase(scope, identifier, label, status)
            .await
    }

    pub async fn publish_telemetry_batch(
        &self,
        batch: TelemetryBatch,
    ) -> crate::runtime::Result<PublishReceipt> {
        self.runtime.publish_telemetry_batch(batch).await
    }

    pub async fn publish_record(
        &self,
        request: RecordPublishRequest,
    ) -> crate::runtime::Result<PublishReceipt> {
        self.runtime.publish_record(request).await
    }

    pub async fn add_document_parts(
        &self,
        scope: RecordScope,
        additions: Vec<crate::document::DocumentAddition>,
    ) -> crate::runtime::Result<()> {
        self.runtime
            .commit_execution_document_parts(crate::documents::AddExecutionDocumentParts {
                scope,
                additions,
                occurred_at: crate::model::time::now_utc(),
            })
            .await
    }

    pub async fn begin_artifact_upload(
        &self,
        input: ArtifactUploadInput,
    ) -> crate::runtime::Result<crate::runtime::ArtifactUpload> {
        self.runtime.begin_artifact_upload(input).await
    }

    pub async fn execution_file_target(
        &self,
        test_execution_id: &TestExecutionId,
        relative_path: String,
        text_hint: bool,
        streamed: bool,
    ) -> crate::runtime::Result<Option<ExecutionFileTarget>> {
        self.runtime
            .execution_file_target(test_execution_id, relative_path, text_hint, streamed)
            .await
    }

    pub async fn request_prompt(
        &self,
        prompt: PromptRequest,
    ) -> crate::runtime::Result<crate::prompts::PromptInstance> {
        self.runtime.request_prompt(prompt).await
    }

    pub async fn wait_for_prompt_response_for_cycle(
        &self,
        prompt_id: PromptId,
        test_cycle_id: TestCycleId,
    ) -> crate::runtime::Result<PromptResponse> {
        self.runtime
            .wait_for_prompt_response_for_cycle(prompt_id, test_cycle_id)
            .await
    }
}

impl ProviderExecutorRegistry {
    pub(crate) fn new(fixtures: BTreeMap<String, Arc<dyn ExecutionFixture>>) -> Self {
        Self {
            fixtures,
            sidecar: Arc::new(super::run::TemplateRunExecutor),
        }
    }

    pub(crate) fn get(
        &self,
        target: &ExecutionProviderTarget,
    ) -> Option<Arc<dyn ExecutionFixture>> {
        let SidecarExecutionBinding { .. } = &target.binding;
        if self.fixtures.len() == 1 {
            return self.fixtures.values().next().cloned();
        }
        self.fixtures
            .get(&target.source)
            .cloned()
            .or_else(|| Some(Arc::clone(&self.sidecar)))
    }

    /// The First Fixture Refusal Of An Execution's Final Report, If Any.
    pub(crate) fn final_report_refusal(&self, execution: &TestExecution) -> Option<String> {
        self.fixtures
            .values()
            .find_map(|fixture| fixture.refuse_final_report(execution))
    }
}

/* === Public Functions === */
pub(crate) fn provider_target_for_cycle(
    state: &RuntimeState,
    test_cycle_id: &TestCycleId,
    selected_execution_id: Option<&TestExecutionId>,
) -> Option<ExecutionProviderTarget> {
    provider_targets_for_cycle(state, test_cycle_id, selected_execution_id)
        .into_iter()
        .next()
}

pub(crate) fn provider_targets_for_cycle(
    state: &RuntimeState,
    test_cycle_id: &TestCycleId,
    selected_execution_id: Option<&TestExecutionId>,
) -> Vec<ExecutionProviderTarget> {
    let Some(cycle) = state.cycles.get(test_cycle_id) else {
        return Vec::new();
    };
    let mut targets: Vec<ExecutionProviderTarget> = Vec::new();
    let resolved = state
        .executions_for_cycle(cycle)
        .filter(|execution| {
            execution.status == crate::execution::ExecutionStatus::Skipped
                && execution.started_at.is_none()
                && execution.finished_at.is_none()
                && selected_execution_id.is_none_or(|id| &execution.test_execution_id == id)
        })
        .filter_map(|execution| {
            state
                .suites
                .provider_target_for_case(execution.snapshot.case())
        });
    for target in resolved {
        if !targets.contains(&target) {
            targets.push(target);
        }
    }

    targets
}

/* === Private Functions === */
/* === Tests === */
