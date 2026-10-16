/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod archive;
pub mod cycle;
pub mod documents;
pub mod evidence;
pub mod execution_control;
pub mod installation;
pub mod invalidation;
pub mod labels;
pub mod lifecycle;
pub mod notes;
pub mod outputs;
pub mod parameters;
pub mod planning;
mod plugins;
pub mod prompts;
pub mod provider;
pub mod provider_finish;

pub(crate) mod records;
mod suite_sets;
pub mod traceability;

/* === Re-Exports === */
pub use archive::{ArchiveSubject, SetArchived};
pub use cycle::{
    ControlCycle, CycleControlAction, CycleResetAction, ResetCycle, UpdateCyclePolicy,
};
pub use documents::{AddExecutionDocumentParts, UpdatePlotSettings};
pub use evidence::{CloseRecordSubscriptions, PublishRecord, RecordHealth};
pub use execution_control::ControlExecution;
pub use installation::{InstallPlugin, RemovePlugin};
pub use invalidation::InvalidateExecutions;
pub use labels::{LabelSubject, RenameLabel};
pub use lifecycle::{
    CaptureReportLifecycle, CycleReportFinalization, EstablishNodeIdentity, MarkRuntimeStarted,
    RecoverInterruptedCycles, ReportFinalization, ReportLifecycle, RequestCycleReportFinalization,
    RequestReportFinalization, StartShutdown,
};
pub use notes::{AddOperatorNote, CreateOperatorNote, EditOperatorNote};
pub use outputs::FinalizeExecutionOutputs;
pub use parameters::UpsertScopedParameter;
pub use planning::{
    AppendCycle, AppendCycleSource, CopyCycle, CreateCycle, CreateCycleMode, CycleDefinition,
    RegisterCases,
};
pub use plugins::{InstallHostPlugin, ReconcileProviders, ReloadProviderCatalog};
pub use prompts::{RequestPrompt, SubmitPromptResponse};
pub use provider::RunProvider;
pub use provider::request::{
    AdvanceRunGroup, CompleteRun, DispatchProvider, PrepareAttempt, SettleRun, Unsealed,
};
pub use provider_finish::FinishProvider;
pub use suite_sets::{RemoveSuiteSet, SetSuiteSet};
pub use traceability::{
    BeginStepAttempt, RecordExecutionTraceability, RecordStepAttempt, SetCapturePhase,
    UpdateExecutionPhase,
};

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
#[harmos::transactions(state = crate::state::RuntimeState, error = String)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Transactions {
    RequestCycleReportFinalization(RequestCycleReportFinalization),
    RequestReportFinalization(RequestReportFinalization),
    CaptureReportLifecycle(CaptureReportLifecycle),
    InvalidateExecutions(InvalidateExecutions),
    #[serde(alias = "InstallArtifact")]
    InstallPlugin(InstallPlugin),
    #[serde(alias = "InstallHostArtifact")]
    InstallHostPlugin(InstallHostPlugin),
    ReconcileProviders(ReconcileProviders),
    ReloadProviderCatalog(ReloadProviderCatalog),
    #[serde(alias = "RemoveArtifact")]
    RemovePlugin(RemovePlugin),
    UpdateExecutionPhase(UpdateExecutionPhase),
    RecordStepAttempt(RecordStepAttempt),
    BeginStepAttempt(BeginStepAttempt),
    SetCapturePhase(SetCapturePhase),
    ControlExecution(ControlExecution),
    RecordExecutionTraceability(RecordExecutionTraceability),
    AddExecutionDocumentParts(AddExecutionDocumentParts),
    UpdatePlotSettings(UpdatePlotSettings),
    RecordHealth(RecordHealth),
    PublishRecord(PublishRecord),
    CloseRecordSubscriptions(CloseRecordSubscriptions),
    UpsertScopedParameter(UpsertScopedParameter),
    RegisterCases(RegisterCases),
    SetSuiteSet(SetSuiteSet),
    RemoveSuiteSet(RemoveSuiteSet),
    RequestPrompt(RequestPrompt),
    SubmitPromptResponse(SubmitPromptResponse),
    FinishProvider(FinishProvider),
    AddOperatorNote(AddOperatorNote),
    CreateOperatorNote(CreateOperatorNote),
    EditOperatorNote(EditOperatorNote),
    MarkRuntimeStarted(MarkRuntimeStarted),
    RecoverInterruptedCycles(RecoverInterruptedCycles),
    EstablishNodeIdentity(EstablishNodeIdentity),
    StartShutdown(StartShutdown),
    ControlCycle(ControlCycle),
    FinalizeExecutionOutputs(FinalizeExecutionOutputs),
    ResetCycle(ResetCycle),
    UpdateCyclePolicy(UpdateCyclePolicy),
    SetArchived(SetArchived),
    RenameLabel(RenameLabel),
    AppendCycle(AppendCycle),
    CreateCycle(CreateCycle),
    CopyCycle(CopyCycle),
    RunProvider(RunProvider),
    PrepareAttempt(PrepareAttempt),
    DispatchProvider(DispatchProvider),
    AdvanceRunGroup(AdvanceRunGroup),
    CompleteRun(CompleteRun),
    SettleRun(SettleRun),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
