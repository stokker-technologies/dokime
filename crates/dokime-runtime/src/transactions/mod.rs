/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub(crate) mod history;

pub mod archive;
pub mod cycle;
pub mod documents;
pub mod effects;
pub mod evidence;
pub mod execution_cancel;
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
pub use cycle::{ControlCycle, CycleControlAction, CycleResetAction, ResetCycle, UpdateCyclePolicy};
pub use documents::{AddExecutionDocumentParts, UpdatePlotSettings};
pub use effects::RecordEffect;
pub use evidence::{CloseRecordSubscriptions, PublishRecord, RecordHealth};
pub use execution_cancel::CancelExecution;
pub use execution_control::ControlExecution;
pub use installation::{InstallPlugin, RemovePlugin};
pub use invalidation::InvalidateExecutions;
pub use labels::{LabelSubject, RenameLabel};
pub use lifecycle::{
    BeginCycleSeal, CaptureReportLifecycle, CycleReportFinalization, EstablishNodeIdentity, FailCycleSeal, FailReport, MarkRuntimeStarted,
    RecoverInterruptedCycles, ReportFinalization, ReportLifecycle, RequestCycleReportFinalization, RequestReportFinalization, SealCycle,
    SealExecution, StageQueuedStop, StartShutdown,
};
pub use notes::{AddOperatorNote, CreateOperatorNote, EditOperatorNote};
pub use outputs::FinalizeExecutionOutputs;
pub use parameters::UpsertScopedParameter;
pub use planning::{AppendExecutions, CreateCycle, CycleDefinition, RegisterCases};
pub use plugins::{InstallHostPlugin, ReconcileProviders, ReloadProviderCatalog};
pub use prompts::{RequestPrompt, SubmitPromptResponse};
pub use provider::RunProvider;
pub use provider::request::{CompleteRun, DispatchProvider, PrepareAttempt, SettleRun, Unsealed};
pub use provider_finish::FinishProvider;
pub use suite_sets::{RemoveSuiteSet, SetSuiteSet};
pub use traceability::{BeginStepAttempt, RecordExecutionTraceability, RecordStepAttempt, SetCapturePhase, UpdateExecutionPhase};

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::Identified;
use serde::{Deserialize, Serialize};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Transactions {
    RecordEffect(RecordEffect),
    RequestCycleReportFinalization(RequestCycleReportFinalization),
    RequestReportFinalization(RequestReportFinalization),
    CaptureReportLifecycle(CaptureReportLifecycle),
    FailReport(FailReport),
    InvalidateExecutions(InvalidateExecutions),
    InstallPlugin(InstallPlugin),
    InstallHostPlugin(InstallHostPlugin),
    ReconcileProviders(ReconcileProviders),
    ReloadProviderCatalog(ReloadProviderCatalog),
    RemovePlugin(RemovePlugin),
    UpdateExecutionPhase(UpdateExecutionPhase),
    RecordStepAttempt(RecordStepAttempt),
    BeginStepAttempt(BeginStepAttempt),
    SetCapturePhase(SetCapturePhase),
    ControlExecution(ControlExecution),
    CancelExecution(CancelExecution),
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
    SealCycle(SealCycle),
    SealExecution(SealExecution),
    BeginCycleSeal(BeginCycleSeal),
    FailCycleSeal(FailCycleSeal),
    StageQueuedStop(StageQueuedStop),
    EstablishNodeIdentity(EstablishNodeIdentity),
    StartShutdown(StartShutdown),
    ControlCycle(ControlCycle),
    FinalizeExecutionOutputs(FinalizeExecutionOutputs),
    ResetCycle(ResetCycle),
    UpdateCyclePolicy(UpdateCyclePolicy),
    SetArchived(SetArchived),
    RenameLabel(RenameLabel),
    AppendExecutions(AppendExecutions),
    CreateCycle(CreateCycle),
    RunProvider(RunProvider),
    PrepareAttempt(PrepareAttempt),
    DispatchProvider(DispatchProvider),
    CompleteRun(CompleteRun),
    SettleRun(SettleRun),
}

/* === Traits === */

/* === Implementations === */

impl From<RecordEffect> for Transactions {
    fn from(value: RecordEffect) -> Self {
        Self::RecordEffect(value)
    }
}

impl From<RequestCycleReportFinalization> for Transactions {
    fn from(value: RequestCycleReportFinalization) -> Self {
        Self::RequestCycleReportFinalization(value)
    }
}

impl From<RequestReportFinalization> for Transactions {
    fn from(value: RequestReportFinalization) -> Self {
        Self::RequestReportFinalization(value)
    }
}

impl From<CaptureReportLifecycle> for Transactions {
    fn from(value: CaptureReportLifecycle) -> Self {
        Self::CaptureReportLifecycle(value)
    }
}

impl From<FailReport> for Transactions {
    fn from(value: FailReport) -> Self {
        Self::FailReport(value)
    }
}

impl From<InvalidateExecutions> for Transactions {
    fn from(value: InvalidateExecutions) -> Self {
        Self::InvalidateExecutions(value)
    }
}

impl From<InstallPlugin> for Transactions {
    fn from(value: InstallPlugin) -> Self {
        Self::InstallPlugin(value)
    }
}

impl From<InstallHostPlugin> for Transactions {
    fn from(value: InstallHostPlugin) -> Self {
        Self::InstallHostPlugin(value)
    }
}

impl From<ReconcileProviders> for Transactions {
    fn from(value: ReconcileProviders) -> Self {
        Self::ReconcileProviders(value)
    }
}

impl From<ReloadProviderCatalog> for Transactions {
    fn from(value: ReloadProviderCatalog) -> Self {
        Self::ReloadProviderCatalog(value)
    }
}

impl From<RemovePlugin> for Transactions {
    fn from(value: RemovePlugin) -> Self {
        Self::RemovePlugin(value)
    }
}

impl From<UpdateExecutionPhase> for Transactions {
    fn from(value: UpdateExecutionPhase) -> Self {
        Self::UpdateExecutionPhase(value)
    }
}

impl From<RecordStepAttempt> for Transactions {
    fn from(value: RecordStepAttempt) -> Self {
        Self::RecordStepAttempt(value)
    }
}

impl From<BeginStepAttempt> for Transactions {
    fn from(value: BeginStepAttempt) -> Self {
        Self::BeginStepAttempt(value)
    }
}

impl From<SetCapturePhase> for Transactions {
    fn from(value: SetCapturePhase) -> Self {
        Self::SetCapturePhase(value)
    }
}

impl From<ControlExecution> for Transactions {
    fn from(value: ControlExecution) -> Self {
        Self::ControlExecution(value)
    }
}

impl From<CancelExecution> for Transactions {
    fn from(value: CancelExecution) -> Self {
        Self::CancelExecution(value)
    }
}

impl From<RecordExecutionTraceability> for Transactions {
    fn from(value: RecordExecutionTraceability) -> Self {
        Self::RecordExecutionTraceability(value)
    }
}

impl From<AddExecutionDocumentParts> for Transactions {
    fn from(value: AddExecutionDocumentParts) -> Self {
        Self::AddExecutionDocumentParts(value)
    }
}

impl From<UpdatePlotSettings> for Transactions {
    fn from(value: UpdatePlotSettings) -> Self {
        Self::UpdatePlotSettings(value)
    }
}

impl From<RecordHealth> for Transactions {
    fn from(value: RecordHealth) -> Self {
        Self::RecordHealth(value)
    }
}

impl From<PublishRecord> for Transactions {
    fn from(value: PublishRecord) -> Self {
        Self::PublishRecord(value)
    }
}

impl From<CloseRecordSubscriptions> for Transactions {
    fn from(value: CloseRecordSubscriptions) -> Self {
        Self::CloseRecordSubscriptions(value)
    }
}

impl From<UpsertScopedParameter> for Transactions {
    fn from(value: UpsertScopedParameter) -> Self {
        Self::UpsertScopedParameter(value)
    }
}

impl From<RegisterCases> for Transactions {
    fn from(value: RegisterCases) -> Self {
        Self::RegisterCases(value)
    }
}

impl From<SetSuiteSet> for Transactions {
    fn from(value: SetSuiteSet) -> Self {
        Self::SetSuiteSet(value)
    }
}

impl From<RemoveSuiteSet> for Transactions {
    fn from(value: RemoveSuiteSet) -> Self {
        Self::RemoveSuiteSet(value)
    }
}

impl From<RequestPrompt> for Transactions {
    fn from(value: RequestPrompt) -> Self {
        Self::RequestPrompt(value)
    }
}

impl From<SubmitPromptResponse> for Transactions {
    fn from(value: SubmitPromptResponse) -> Self {
        Self::SubmitPromptResponse(value)
    }
}

impl From<FinishProvider> for Transactions {
    fn from(value: FinishProvider) -> Self {
        Self::FinishProvider(value)
    }
}

impl From<AddOperatorNote> for Transactions {
    fn from(value: AddOperatorNote) -> Self {
        Self::AddOperatorNote(value)
    }
}

impl From<CreateOperatorNote> for Transactions {
    fn from(value: CreateOperatorNote) -> Self {
        Self::CreateOperatorNote(value)
    }
}

impl From<EditOperatorNote> for Transactions {
    fn from(value: EditOperatorNote) -> Self {
        Self::EditOperatorNote(value)
    }
}

impl From<MarkRuntimeStarted> for Transactions {
    fn from(value: MarkRuntimeStarted) -> Self {
        Self::MarkRuntimeStarted(value)
    }
}

impl From<RecoverInterruptedCycles> for Transactions {
    fn from(value: RecoverInterruptedCycles) -> Self {
        Self::RecoverInterruptedCycles(value)
    }
}

impl From<SealCycle> for Transactions {
    fn from(value: SealCycle) -> Self {
        Self::SealCycle(value)
    }
}

impl From<SealExecution> for Transactions {
    fn from(value: SealExecution) -> Self {
        Self::SealExecution(value)
    }
}

impl From<BeginCycleSeal> for Transactions {
    fn from(value: BeginCycleSeal) -> Self {
        Self::BeginCycleSeal(value)
    }
}

impl From<FailCycleSeal> for Transactions {
    fn from(value: FailCycleSeal) -> Self {
        Self::FailCycleSeal(value)
    }
}

impl From<StageQueuedStop> for Transactions {
    fn from(value: StageQueuedStop) -> Self {
        Self::StageQueuedStop(value)
    }
}

impl From<EstablishNodeIdentity> for Transactions {
    fn from(value: EstablishNodeIdentity) -> Self {
        Self::EstablishNodeIdentity(value)
    }
}

impl From<StartShutdown> for Transactions {
    fn from(value: StartShutdown) -> Self {
        Self::StartShutdown(value)
    }
}

impl From<ControlCycle> for Transactions {
    fn from(value: ControlCycle) -> Self {
        Self::ControlCycle(value)
    }
}

impl From<FinalizeExecutionOutputs> for Transactions {
    fn from(value: FinalizeExecutionOutputs) -> Self {
        Self::FinalizeExecutionOutputs(value)
    }
}

impl From<ResetCycle> for Transactions {
    fn from(value: ResetCycle) -> Self {
        Self::ResetCycle(value)
    }
}

impl From<UpdateCyclePolicy> for Transactions {
    fn from(value: UpdateCyclePolicy) -> Self {
        Self::UpdateCyclePolicy(value)
    }
}

impl From<SetArchived> for Transactions {
    fn from(value: SetArchived) -> Self {
        Self::SetArchived(value)
    }
}

impl From<RenameLabel> for Transactions {
    fn from(value: RenameLabel) -> Self {
        Self::RenameLabel(value)
    }
}

impl From<AppendExecutions> for Transactions {
    fn from(value: AppendExecutions) -> Self {
        Self::AppendExecutions(value)
    }
}

impl From<CreateCycle> for Transactions {
    fn from(value: CreateCycle) -> Self {
        Self::CreateCycle(value)
    }
}

impl From<RunProvider> for Transactions {
    fn from(value: RunProvider) -> Self {
        Self::RunProvider(value)
    }
}

impl From<PrepareAttempt> for Transactions {
    fn from(value: PrepareAttempt) -> Self {
        Self::PrepareAttempt(value)
    }
}

impl From<DispatchProvider> for Transactions {
    fn from(value: DispatchProvider) -> Self {
        Self::DispatchProvider(value)
    }
}

impl From<CompleteRun> for Transactions {
    fn from(value: CompleteRun) -> Self {
        Self::CompleteRun(value)
    }
}

impl From<SettleRun> for Transactions {
    fn from(value: SettleRun) -> Self {
        Self::SettleRun(value)
    }
}

impl Transactions {
    pub fn catalog() -> harmos::state::TransactionDefinitions<crate::state::RuntimeState, Self> {
        let definitions = vec![
            {
                let (id, version) = RecordEffect::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = RequestCycleReportFinalization::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = RequestReportFinalization::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = CaptureReportLifecycle::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = FailReport::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = InvalidateExecutions::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = InstallPlugin::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = InstallHostPlugin::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = ReconcileProviders::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = ReloadProviderCatalog::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = RemovePlugin::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = UpdateExecutionPhase::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = RecordStepAttempt::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = BeginStepAttempt::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = SetCapturePhase::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = ControlExecution::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = CancelExecution::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = RecordExecutionTraceability::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = AddExecutionDocumentParts::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = UpdatePlotSettings::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = RecordHealth::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = PublishRecord::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = CloseRecordSubscriptions::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = UpsertScopedParameter::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = RegisterCases::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = SetSuiteSet::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = RemoveSuiteSet::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = RequestPrompt::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = SubmitPromptResponse::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = FinishProvider::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = AddOperatorNote::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = CreateOperatorNote::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = EditOperatorNote::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = MarkRuntimeStarted::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = RecoverInterruptedCycles::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = SealCycle::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = SealExecution::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = BeginCycleSeal::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = FailCycleSeal::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = StageQueuedStop::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = EstablishNodeIdentity::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = StartShutdown::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = ControlCycle::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = FinalizeExecutionOutputs::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = ResetCycle::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = UpdateCyclePolicy::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = SetArchived::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = RenameLabel::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = AppendExecutions::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = CreateCycle::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = RunProvider::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = PrepareAttempt::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = DispatchProvider::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = CompleteRun::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
            {
                let (id, version) = SettleRun::definition();
                (harmos::state::Kind::Transaction, id, version)
            },
        ];
        harmos::state::TransactionDefinitions {
            definitions,
            decode: Self::decode,
            inverses: |_, _| None,
        }
    }

    fn decode(id: &harmos::DefinitionId, version: u32, bytes: &[u8]) -> Result<Option<Self>, harmos::state::DecodeError> {
        let malformed = || harmos::state::DecodeError::Malformed { id: id.clone(), version };
        let payload = serde_json::from_slice(bytes).map_err(|_| malformed())?;
        Self::restore(id, version, &payload).map_err(|_| malformed())
    }

    fn restore(id: &harmos::DefinitionId, version: u32, payload: &serde_json::Value) -> Result<Option<Self>, serde_json::Error> {
        if id.as_str() == RecordEffect::ID {
            if version != RecordEffect::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::RecordEffect).map(Some);
        }
        if id.as_str() == RequestCycleReportFinalization::ID {
            if version != RequestCycleReportFinalization::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::RequestCycleReportFinalization).map(Some);
        }
        if id.as_str() == RequestReportFinalization::ID {
            if version != RequestReportFinalization::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::RequestReportFinalization).map(Some);
        }
        if id.as_str() == CaptureReportLifecycle::ID {
            if version != CaptureReportLifecycle::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::CaptureReportLifecycle).map(Some);
        }
        if id.as_str() == FailReport::ID {
            if version != FailReport::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::FailReport).map(Some);
        }
        if id.as_str() == InvalidateExecutions::ID {
            if version != InvalidateExecutions::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::InvalidateExecutions).map(Some);
        }
        if id.as_str() == InstallPlugin::ID {
            if version != InstallPlugin::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::InstallPlugin).map(Some);
        }
        if id.as_str() == InstallHostPlugin::ID {
            if version != InstallHostPlugin::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::InstallHostPlugin).map(Some);
        }
        if id.as_str() == ReconcileProviders::ID {
            if version != ReconcileProviders::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::ReconcileProviders).map(Some);
        }
        if id.as_str() == ReloadProviderCatalog::ID {
            if version != ReloadProviderCatalog::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::ReloadProviderCatalog).map(Some);
        }
        if id.as_str() == RemovePlugin::ID {
            if version != RemovePlugin::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::RemovePlugin).map(Some);
        }
        if id.as_str() == UpdateExecutionPhase::ID {
            if version != UpdateExecutionPhase::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::UpdateExecutionPhase).map(Some);
        }
        if id.as_str() == RecordStepAttempt::ID {
            if version != RecordStepAttempt::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::RecordStepAttempt).map(Some);
        }
        if id.as_str() == BeginStepAttempt::ID {
            if version != BeginStepAttempt::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::BeginStepAttempt).map(Some);
        }
        if id.as_str() == SetCapturePhase::ID {
            if version != SetCapturePhase::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::SetCapturePhase).map(Some);
        }
        if id.as_str() == ControlExecution::ID {
            if version != ControlExecution::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::ControlExecution).map(Some);
        }
        if id.as_str() == CancelExecution::ID {
            if version != CancelExecution::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::CancelExecution).map(Some);
        }
        if id.as_str() == RecordExecutionTraceability::ID {
            if version != RecordExecutionTraceability::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::RecordExecutionTraceability).map(Some);
        }
        if id.as_str() == AddExecutionDocumentParts::ID {
            if version != AddExecutionDocumentParts::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::AddExecutionDocumentParts).map(Some);
        }
        if id.as_str() == UpdatePlotSettings::ID {
            if version != UpdatePlotSettings::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::UpdatePlotSettings).map(Some);
        }
        if id.as_str() == RecordHealth::ID {
            if version != RecordHealth::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::RecordHealth).map(Some);
        }
        if id.as_str() == PublishRecord::ID {
            if version != PublishRecord::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::PublishRecord).map(Some);
        }
        if id.as_str() == CloseRecordSubscriptions::ID {
            if version != CloseRecordSubscriptions::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::CloseRecordSubscriptions).map(Some);
        }
        if id.as_str() == UpsertScopedParameter::ID {
            if version != UpsertScopedParameter::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::UpsertScopedParameter).map(Some);
        }
        if id.as_str() == RegisterCases::ID {
            if version != RegisterCases::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::RegisterCases).map(Some);
        }
        if id.as_str() == SetSuiteSet::ID {
            if version != SetSuiteSet::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::SetSuiteSet).map(Some);
        }
        if id.as_str() == RemoveSuiteSet::ID {
            if version != RemoveSuiteSet::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::RemoveSuiteSet).map(Some);
        }
        if id.as_str() == RequestPrompt::ID {
            if version != RequestPrompt::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::RequestPrompt).map(Some);
        }
        if id.as_str() == SubmitPromptResponse::ID {
            if version != SubmitPromptResponse::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::SubmitPromptResponse).map(Some);
        }
        if id.as_str() == FinishProvider::ID {
            if version != FinishProvider::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::FinishProvider).map(Some);
        }
        if id.as_str() == AddOperatorNote::ID {
            if version != AddOperatorNote::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::AddOperatorNote).map(Some);
        }
        if id.as_str() == CreateOperatorNote::ID {
            if version != CreateOperatorNote::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::CreateOperatorNote).map(Some);
        }
        if id.as_str() == EditOperatorNote::ID {
            if version != EditOperatorNote::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::EditOperatorNote).map(Some);
        }
        if id.as_str() == MarkRuntimeStarted::ID {
            if version != MarkRuntimeStarted::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::MarkRuntimeStarted).map(Some);
        }
        if id.as_str() == RecoverInterruptedCycles::ID {
            if version != RecoverInterruptedCycles::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::RecoverInterruptedCycles).map(Some);
        }
        if id.as_str() == SealCycle::ID {
            if version != SealCycle::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::SealCycle).map(Some);
        }
        if id.as_str() == SealExecution::ID {
            if version != SealExecution::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::SealExecution).map(Some);
        }
        if id.as_str() == BeginCycleSeal::ID {
            if version != BeginCycleSeal::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::BeginCycleSeal).map(Some);
        }
        if id.as_str() == FailCycleSeal::ID {
            if version != FailCycleSeal::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::FailCycleSeal).map(Some);
        }
        if id.as_str() == StageQueuedStop::ID {
            if version != StageQueuedStop::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::StageQueuedStop).map(Some);
        }
        if id.as_str() == EstablishNodeIdentity::ID {
            if version != EstablishNodeIdentity::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::EstablishNodeIdentity).map(Some);
        }
        if id.as_str() == StartShutdown::ID {
            if version != StartShutdown::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::StartShutdown).map(Some);
        }
        if id.as_str() == ControlCycle::ID {
            if version != ControlCycle::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::ControlCycle).map(Some);
        }
        if id.as_str() == FinalizeExecutionOutputs::ID {
            if version != FinalizeExecutionOutputs::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::FinalizeExecutionOutputs).map(Some);
        }
        if id.as_str() == ResetCycle::ID {
            if version != ResetCycle::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::ResetCycle).map(Some);
        }
        if id.as_str() == UpdateCyclePolicy::ID {
            if version != UpdateCyclePolicy::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::UpdateCyclePolicy).map(Some);
        }
        if id.as_str() == SetArchived::ID {
            if version != SetArchived::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::SetArchived).map(Some);
        }
        if id.as_str() == RenameLabel::ID {
            if version != RenameLabel::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::RenameLabel).map(Some);
        }
        if id.as_str() == AppendExecutions::ID {
            if version != AppendExecutions::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::AppendExecutions).map(Some);
        }
        if id.as_str() == CreateCycle::ID {
            if version != CreateCycle::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::CreateCycle).map(Some);
        }
        if id.as_str() == RunProvider::ID {
            if version != RunProvider::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::RunProvider).map(Some);
        }
        if id.as_str() == PrepareAttempt::ID {
            if version != PrepareAttempt::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::PrepareAttempt).map(Some);
        }
        if id.as_str() == DispatchProvider::ID {
            if version != DispatchProvider::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::DispatchProvider).map(Some);
        }
        if id.as_str() == CompleteRun::ID {
            if version != CompleteRun::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::CompleteRun).map(Some);
        }
        if id.as_str() == SettleRun::ID {
            if version != SettleRun::definition().1 {
                return Ok(None);
            }
            return serde::Deserialize::deserialize(payload).map(Self::SettleRun).map(Some);
        }
        Ok(None)
    }
}

impl harmos::state::CatalogCodec for Transactions {
    fn identify(&self) -> (harmos::DefinitionId, u32) {
        match self {
            Self::RecordEffect(_) => RecordEffect::definition(),
            Self::RequestCycleReportFinalization(_) => RequestCycleReportFinalization::definition(),
            Self::RequestReportFinalization(_) => RequestReportFinalization::definition(),
            Self::CaptureReportLifecycle(_) => CaptureReportLifecycle::definition(),
            Self::FailReport(_) => FailReport::definition(),
            Self::InvalidateExecutions(_) => InvalidateExecutions::definition(),
            Self::InstallPlugin(_) => InstallPlugin::definition(),
            Self::InstallHostPlugin(_) => InstallHostPlugin::definition(),
            Self::ReconcileProviders(_) => ReconcileProviders::definition(),
            Self::ReloadProviderCatalog(_) => ReloadProviderCatalog::definition(),
            Self::RemovePlugin(_) => RemovePlugin::definition(),
            Self::UpdateExecutionPhase(_) => UpdateExecutionPhase::definition(),
            Self::RecordStepAttempt(_) => RecordStepAttempt::definition(),
            Self::BeginStepAttempt(_) => BeginStepAttempt::definition(),
            Self::SetCapturePhase(_) => SetCapturePhase::definition(),
            Self::ControlExecution(_) => ControlExecution::definition(),
            Self::CancelExecution(_) => CancelExecution::definition(),
            Self::RecordExecutionTraceability(_) => RecordExecutionTraceability::definition(),
            Self::AddExecutionDocumentParts(_) => AddExecutionDocumentParts::definition(),
            Self::UpdatePlotSettings(_) => UpdatePlotSettings::definition(),
            Self::RecordHealth(_) => RecordHealth::definition(),
            Self::PublishRecord(_) => PublishRecord::definition(),
            Self::CloseRecordSubscriptions(_) => CloseRecordSubscriptions::definition(),
            Self::UpsertScopedParameter(_) => UpsertScopedParameter::definition(),
            Self::RegisterCases(_) => RegisterCases::definition(),
            Self::SetSuiteSet(_) => SetSuiteSet::definition(),
            Self::RemoveSuiteSet(_) => RemoveSuiteSet::definition(),
            Self::RequestPrompt(_) => RequestPrompt::definition(),
            Self::SubmitPromptResponse(_) => SubmitPromptResponse::definition(),
            Self::FinishProvider(_) => FinishProvider::definition(),
            Self::AddOperatorNote(_) => AddOperatorNote::definition(),
            Self::CreateOperatorNote(_) => CreateOperatorNote::definition(),
            Self::EditOperatorNote(_) => EditOperatorNote::definition(),
            Self::MarkRuntimeStarted(_) => MarkRuntimeStarted::definition(),
            Self::RecoverInterruptedCycles(_) => RecoverInterruptedCycles::definition(),
            Self::SealCycle(_) => SealCycle::definition(),
            Self::SealExecution(_) => SealExecution::definition(),
            Self::BeginCycleSeal(_) => BeginCycleSeal::definition(),
            Self::FailCycleSeal(_) => FailCycleSeal::definition(),
            Self::StageQueuedStop(_) => StageQueuedStop::definition(),
            Self::EstablishNodeIdentity(_) => EstablishNodeIdentity::definition(),
            Self::StartShutdown(_) => StartShutdown::definition(),
            Self::ControlCycle(_) => ControlCycle::definition(),
            Self::FinalizeExecutionOutputs(_) => FinalizeExecutionOutputs::definition(),
            Self::ResetCycle(_) => ResetCycle::definition(),
            Self::UpdateCyclePolicy(_) => UpdateCyclePolicy::definition(),
            Self::SetArchived(_) => SetArchived::definition(),
            Self::RenameLabel(_) => RenameLabel::definition(),
            Self::AppendExecutions(_) => AppendExecutions::definition(),
            Self::CreateCycle(_) => CreateCycle::definition(),
            Self::RunProvider(_) => RunProvider::definition(),
            Self::PrepareAttempt(_) => PrepareAttempt::definition(),
            Self::DispatchProvider(_) => DispatchProvider::definition(),
            Self::CompleteRun(_) => CompleteRun::definition(),
            Self::SettleRun(_) => SettleRun::definition(),
        }
    }

    fn encode<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::RecordEffect(value) => value.serialize(serializer),
            Self::RequestCycleReportFinalization(value) => value.serialize(serializer),
            Self::RequestReportFinalization(value) => value.serialize(serializer),
            Self::CaptureReportLifecycle(value) => value.serialize(serializer),
            Self::FailReport(value) => value.serialize(serializer),
            Self::InvalidateExecutions(value) => value.serialize(serializer),
            Self::InstallPlugin(value) => value.serialize(serializer),
            Self::InstallHostPlugin(value) => value.serialize(serializer),
            Self::ReconcileProviders(value) => value.serialize(serializer),
            Self::ReloadProviderCatalog(value) => value.serialize(serializer),
            Self::RemovePlugin(value) => value.serialize(serializer),
            Self::UpdateExecutionPhase(value) => value.serialize(serializer),
            Self::RecordStepAttempt(value) => value.serialize(serializer),
            Self::BeginStepAttempt(value) => value.serialize(serializer),
            Self::SetCapturePhase(value) => value.serialize(serializer),
            Self::ControlExecution(value) => value.serialize(serializer),
            Self::CancelExecution(value) => value.serialize(serializer),
            Self::RecordExecutionTraceability(value) => value.serialize(serializer),
            Self::AddExecutionDocumentParts(value) => value.serialize(serializer),
            Self::UpdatePlotSettings(value) => value.serialize(serializer),
            Self::RecordHealth(value) => value.serialize(serializer),
            Self::PublishRecord(value) => value.serialize(serializer),
            Self::CloseRecordSubscriptions(value) => value.serialize(serializer),
            Self::UpsertScopedParameter(value) => value.serialize(serializer),
            Self::RegisterCases(value) => value.serialize(serializer),
            Self::SetSuiteSet(value) => value.serialize(serializer),
            Self::RemoveSuiteSet(value) => value.serialize(serializer),
            Self::RequestPrompt(value) => value.serialize(serializer),
            Self::SubmitPromptResponse(value) => value.serialize(serializer),
            Self::FinishProvider(value) => value.serialize(serializer),
            Self::AddOperatorNote(value) => value.serialize(serializer),
            Self::CreateOperatorNote(value) => value.serialize(serializer),
            Self::EditOperatorNote(value) => value.serialize(serializer),
            Self::MarkRuntimeStarted(value) => value.serialize(serializer),
            Self::RecoverInterruptedCycles(value) => value.serialize(serializer),
            Self::SealCycle(value) => value.serialize(serializer),
            Self::SealExecution(value) => value.serialize(serializer),
            Self::BeginCycleSeal(value) => value.serialize(serializer),
            Self::FailCycleSeal(value) => value.serialize(serializer),
            Self::StageQueuedStop(value) => value.serialize(serializer),
            Self::EstablishNodeIdentity(value) => value.serialize(serializer),
            Self::StartShutdown(value) => value.serialize(serializer),
            Self::ControlCycle(value) => value.serialize(serializer),
            Self::FinalizeExecutionOutputs(value) => value.serialize(serializer),
            Self::ResetCycle(value) => value.serialize(serializer),
            Self::UpdateCyclePolicy(value) => value.serialize(serializer),
            Self::SetArchived(value) => value.serialize(serializer),
            Self::RenameLabel(value) => value.serialize(serializer),
            Self::AppendExecutions(value) => value.serialize(serializer),
            Self::CreateCycle(value) => value.serialize(serializer),
            Self::RunProvider(value) => value.serialize(serializer),
            Self::PrepareAttempt(value) => value.serialize(serializer),
            Self::DispatchProvider(value) => value.serialize(serializer),
            Self::CompleteRun(value) => value.serialize(serializer),
            Self::SettleRun(value) => value.serialize(serializer),
        }
    }
}

impl harmos::state::Transaction<crate::state::RuntimeState> for Transactions {
    type Error = String;

    fn check(&self, state: &crate::state::RuntimeState) -> Result<(), String> {
        match self {
            Self::RecordEffect(value) => harmos::state::Transaction::check(value, state),
            Self::RequestCycleReportFinalization(value) => harmos::state::Transaction::check(value, state),
            Self::RequestReportFinalization(value) => harmos::state::Transaction::check(value, state),
            Self::CaptureReportLifecycle(value) => harmos::state::Transaction::check(value, state),
            Self::FailReport(value) => harmos::state::Transaction::check(value, state),
            Self::InvalidateExecutions(value) => harmos::state::Transaction::check(value, state),
            Self::InstallPlugin(value) => harmos::state::Transaction::check(value, state),
            Self::InstallHostPlugin(value) => harmos::state::Transaction::check(value, state),
            Self::ReconcileProviders(value) => harmos::state::Transaction::check(value, state),
            Self::ReloadProviderCatalog(value) => harmos::state::Transaction::check(value, state),
            Self::RemovePlugin(value) => harmos::state::Transaction::check(value, state),
            Self::UpdateExecutionPhase(value) => harmos::state::Transaction::check(value, state),
            Self::RecordStepAttempt(value) => harmos::state::Transaction::check(value, state),
            Self::BeginStepAttempt(value) => harmos::state::Transaction::check(value, state),
            Self::SetCapturePhase(value) => harmos::state::Transaction::check(value, state),
            Self::ControlExecution(value) => harmos::state::Transaction::check(value, state),
            Self::CancelExecution(value) => harmos::state::Transaction::check(value, state),
            Self::RecordExecutionTraceability(value) => harmos::state::Transaction::check(value, state),
            Self::AddExecutionDocumentParts(value) => harmos::state::Transaction::check(value, state),
            Self::UpdatePlotSettings(value) => harmos::state::Transaction::check(value, state),
            Self::RecordHealth(value) => harmos::state::Transaction::check(value, state),
            Self::PublishRecord(value) => harmos::state::Transaction::check(value, state),
            Self::CloseRecordSubscriptions(value) => harmos::state::Transaction::check(value, state),
            Self::UpsertScopedParameter(value) => harmos::state::Transaction::check(value, state),
            Self::RegisterCases(value) => harmos::state::Transaction::check(value, state),
            Self::SetSuiteSet(value) => harmos::state::Transaction::check(value, state),
            Self::RemoveSuiteSet(value) => harmos::state::Transaction::check(value, state),
            Self::RequestPrompt(value) => harmos::state::Transaction::check(value, state),
            Self::SubmitPromptResponse(value) => harmos::state::Transaction::check(value, state),
            Self::FinishProvider(value) => harmos::state::Transaction::check(value, state),
            Self::AddOperatorNote(value) => harmos::state::Transaction::check(value, state),
            Self::CreateOperatorNote(value) => harmos::state::Transaction::check(value, state),
            Self::EditOperatorNote(value) => harmos::state::Transaction::check(value, state),
            Self::MarkRuntimeStarted(value) => harmos::state::Transaction::check(value, state),
            Self::RecoverInterruptedCycles(value) => harmos::state::Transaction::check(value, state),
            Self::SealCycle(value) => harmos::state::Transaction::check(value, state),
            Self::SealExecution(value) => harmos::state::Transaction::check(value, state),
            Self::BeginCycleSeal(value) => harmos::state::Transaction::check(value, state),
            Self::FailCycleSeal(value) => harmos::state::Transaction::check(value, state),
            Self::StageQueuedStop(value) => harmos::state::Transaction::check(value, state),
            Self::EstablishNodeIdentity(value) => harmos::state::Transaction::check(value, state),
            Self::StartShutdown(value) => harmos::state::Transaction::check(value, state),
            Self::ControlCycle(value) => harmos::state::Transaction::check(value, state),
            Self::FinalizeExecutionOutputs(value) => harmos::state::Transaction::check(value, state),
            Self::ResetCycle(value) => harmos::state::Transaction::check(value, state),
            Self::UpdateCyclePolicy(value) => harmos::state::Transaction::check(value, state),
            Self::SetArchived(value) => harmos::state::Transaction::check(value, state),
            Self::RenameLabel(value) => harmos::state::Transaction::check(value, state),
            Self::AppendExecutions(value) => harmos::state::Transaction::check(value, state),
            Self::CreateCycle(value) => harmos::state::Transaction::check(value, state),
            Self::RunProvider(value) => harmos::state::Transaction::check(value, state),
            Self::PrepareAttempt(value) => harmos::state::Transaction::check(value, state),
            Self::DispatchProvider(value) => harmos::state::Transaction::check(value, state),
            Self::CompleteRun(value) => harmos::state::Transaction::check(value, state),
            Self::SettleRun(value) => harmos::state::Transaction::check(value, state),
        }
    }

    fn records(&self, state: &crate::state::RuntimeState, out: &mut harmos::state::Records) {
        match self {
            Self::RecordEffect(value) => harmos::state::Transaction::records(value, state, out),
            Self::RequestCycleReportFinalization(value) => harmos::state::Transaction::records(value, state, out),
            Self::RequestReportFinalization(value) => harmos::state::Transaction::records(value, state, out),
            Self::CaptureReportLifecycle(value) => harmos::state::Transaction::records(value, state, out),
            Self::FailReport(value) => harmos::state::Transaction::records(value, state, out),
            Self::InvalidateExecutions(value) => harmos::state::Transaction::records(value, state, out),
            Self::InstallPlugin(value) => harmos::state::Transaction::records(value, state, out),
            Self::InstallHostPlugin(value) => harmos::state::Transaction::records(value, state, out),
            Self::ReconcileProviders(value) => harmos::state::Transaction::records(value, state, out),
            Self::ReloadProviderCatalog(value) => harmos::state::Transaction::records(value, state, out),
            Self::RemovePlugin(value) => harmos::state::Transaction::records(value, state, out),
            Self::UpdateExecutionPhase(value) => harmos::state::Transaction::records(value, state, out),
            Self::RecordStepAttempt(value) => harmos::state::Transaction::records(value, state, out),
            Self::BeginStepAttempt(value) => harmos::state::Transaction::records(value, state, out),
            Self::SetCapturePhase(value) => harmos::state::Transaction::records(value, state, out),
            Self::ControlExecution(value) => harmos::state::Transaction::records(value, state, out),
            Self::CancelExecution(value) => harmos::state::Transaction::records(value, state, out),
            Self::RecordExecutionTraceability(value) => harmos::state::Transaction::records(value, state, out),
            Self::AddExecutionDocumentParts(value) => harmos::state::Transaction::records(value, state, out),
            Self::UpdatePlotSettings(value) => harmos::state::Transaction::records(value, state, out),
            Self::RecordHealth(value) => harmos::state::Transaction::records(value, state, out),
            Self::PublishRecord(value) => harmos::state::Transaction::records(value, state, out),
            Self::CloseRecordSubscriptions(value) => harmos::state::Transaction::records(value, state, out),
            Self::UpsertScopedParameter(value) => harmos::state::Transaction::records(value, state, out),
            Self::RegisterCases(value) => harmos::state::Transaction::records(value, state, out),
            Self::SetSuiteSet(value) => harmos::state::Transaction::records(value, state, out),
            Self::RemoveSuiteSet(value) => harmos::state::Transaction::records(value, state, out),
            Self::RequestPrompt(value) => harmos::state::Transaction::records(value, state, out),
            Self::SubmitPromptResponse(value) => harmos::state::Transaction::records(value, state, out),
            Self::FinishProvider(value) => harmos::state::Transaction::records(value, state, out),
            Self::AddOperatorNote(value) => harmos::state::Transaction::records(value, state, out),
            Self::CreateOperatorNote(value) => harmos::state::Transaction::records(value, state, out),
            Self::EditOperatorNote(value) => harmos::state::Transaction::records(value, state, out),
            Self::MarkRuntimeStarted(value) => harmos::state::Transaction::records(value, state, out),
            Self::RecoverInterruptedCycles(value) => harmos::state::Transaction::records(value, state, out),
            Self::SealCycle(value) => harmos::state::Transaction::records(value, state, out),
            Self::SealExecution(value) => harmos::state::Transaction::records(value, state, out),
            Self::BeginCycleSeal(value) => harmos::state::Transaction::records(value, state, out),
            Self::FailCycleSeal(value) => harmos::state::Transaction::records(value, state, out),
            Self::StageQueuedStop(value) => harmos::state::Transaction::records(value, state, out),
            Self::EstablishNodeIdentity(value) => harmos::state::Transaction::records(value, state, out),
            Self::StartShutdown(value) => harmos::state::Transaction::records(value, state, out),
            Self::ControlCycle(value) => harmos::state::Transaction::records(value, state, out),
            Self::FinalizeExecutionOutputs(value) => harmos::state::Transaction::records(value, state, out),
            Self::ResetCycle(value) => harmos::state::Transaction::records(value, state, out),
            Self::UpdateCyclePolicy(value) => harmos::state::Transaction::records(value, state, out),
            Self::SetArchived(value) => harmos::state::Transaction::records(value, state, out),
            Self::RenameLabel(value) => harmos::state::Transaction::records(value, state, out),
            Self::AppendExecutions(value) => harmos::state::Transaction::records(value, state, out),
            Self::CreateCycle(value) => harmos::state::Transaction::records(value, state, out),
            Self::RunProvider(value) => harmos::state::Transaction::records(value, state, out),
            Self::PrepareAttempt(value) => harmos::state::Transaction::records(value, state, out),
            Self::DispatchProvider(value) => harmos::state::Transaction::records(value, state, out),
            Self::CompleteRun(value) => harmos::state::Transaction::records(value, state, out),
            Self::SettleRun(value) => harmos::state::Transaction::records(value, state, out),
        }
    }

    fn apply(&self, state: &mut crate::state::RuntimeState) {
        match self {
            Self::RecordEffect(value) => harmos::state::Transaction::apply(value, state),
            Self::RequestCycleReportFinalization(value) => harmos::state::Transaction::apply(value, state),
            Self::RequestReportFinalization(value) => harmos::state::Transaction::apply(value, state),
            Self::CaptureReportLifecycle(value) => harmos::state::Transaction::apply(value, state),
            Self::FailReport(value) => harmos::state::Transaction::apply(value, state),
            Self::InvalidateExecutions(value) => harmos::state::Transaction::apply(value, state),
            Self::InstallPlugin(value) => harmos::state::Transaction::apply(value, state),
            Self::InstallHostPlugin(value) => harmos::state::Transaction::apply(value, state),
            Self::ReconcileProviders(value) => harmos::state::Transaction::apply(value, state),
            Self::ReloadProviderCatalog(value) => harmos::state::Transaction::apply(value, state),
            Self::RemovePlugin(value) => harmos::state::Transaction::apply(value, state),
            Self::UpdateExecutionPhase(value) => harmos::state::Transaction::apply(value, state),
            Self::RecordStepAttempt(value) => harmos::state::Transaction::apply(value, state),
            Self::BeginStepAttempt(value) => harmos::state::Transaction::apply(value, state),
            Self::SetCapturePhase(value) => harmos::state::Transaction::apply(value, state),
            Self::ControlExecution(value) => harmos::state::Transaction::apply(value, state),
            Self::CancelExecution(value) => harmos::state::Transaction::apply(value, state),
            Self::RecordExecutionTraceability(value) => harmos::state::Transaction::apply(value, state),
            Self::AddExecutionDocumentParts(value) => harmos::state::Transaction::apply(value, state),
            Self::UpdatePlotSettings(value) => harmos::state::Transaction::apply(value, state),
            Self::RecordHealth(value) => harmos::state::Transaction::apply(value, state),
            Self::PublishRecord(value) => harmos::state::Transaction::apply(value, state),
            Self::CloseRecordSubscriptions(value) => harmos::state::Transaction::apply(value, state),
            Self::UpsertScopedParameter(value) => harmos::state::Transaction::apply(value, state),
            Self::RegisterCases(value) => harmos::state::Transaction::apply(value, state),
            Self::SetSuiteSet(value) => harmos::state::Transaction::apply(value, state),
            Self::RemoveSuiteSet(value) => harmos::state::Transaction::apply(value, state),
            Self::RequestPrompt(value) => harmos::state::Transaction::apply(value, state),
            Self::SubmitPromptResponse(value) => harmos::state::Transaction::apply(value, state),
            Self::FinishProvider(value) => harmos::state::Transaction::apply(value, state),
            Self::AddOperatorNote(value) => harmos::state::Transaction::apply(value, state),
            Self::CreateOperatorNote(value) => harmos::state::Transaction::apply(value, state),
            Self::EditOperatorNote(value) => harmos::state::Transaction::apply(value, state),
            Self::MarkRuntimeStarted(value) => harmos::state::Transaction::apply(value, state),
            Self::RecoverInterruptedCycles(value) => harmos::state::Transaction::apply(value, state),
            Self::SealCycle(value) => harmos::state::Transaction::apply(value, state),
            Self::SealExecution(value) => harmos::state::Transaction::apply(value, state),
            Self::BeginCycleSeal(value) => harmos::state::Transaction::apply(value, state),
            Self::FailCycleSeal(value) => harmos::state::Transaction::apply(value, state),
            Self::StageQueuedStop(value) => harmos::state::Transaction::apply(value, state),
            Self::EstablishNodeIdentity(value) => harmos::state::Transaction::apply(value, state),
            Self::StartShutdown(value) => harmos::state::Transaction::apply(value, state),
            Self::ControlCycle(value) => harmos::state::Transaction::apply(value, state),
            Self::FinalizeExecutionOutputs(value) => harmos::state::Transaction::apply(value, state),
            Self::ResetCycle(value) => harmos::state::Transaction::apply(value, state),
            Self::UpdateCyclePolicy(value) => harmos::state::Transaction::apply(value, state),
            Self::SetArchived(value) => harmos::state::Transaction::apply(value, state),
            Self::RenameLabel(value) => harmos::state::Transaction::apply(value, state),
            Self::AppendExecutions(value) => harmos::state::Transaction::apply(value, state),
            Self::CreateCycle(value) => harmos::state::Transaction::apply(value, state),
            Self::RunProvider(value) => harmos::state::Transaction::apply(value, state),
            Self::PrepareAttempt(value) => harmos::state::Transaction::apply(value, state),
            Self::DispatchProvider(value) => harmos::state::Transaction::apply(value, state),
            Self::CompleteRun(value) => harmos::state::Transaction::apply(value, state),
            Self::SettleRun(value) => harmos::state::Transaction::apply(value, state),
        }
    }
}

impl harmos::state::TransactionCatalog<crate::state::RuntimeState> for Transactions {
    fn replay(&self, prepared: &harmos::state::Prepared, state: &mut crate::state::RuntimeState) -> Result<(), harmos::state::DecodeError> {
        let malformed = || harmos::state::DecodeError::Malformed {
            id: prepared.id.clone(),
            version: prepared.version,
        };
        let value = Self::restore(&prepared.id, prepared.version, &prepared.payload).map_err(|_| malformed())?.ok_or_else(malformed)?;
        harmos::state::Transaction::apply(&value, state);
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
