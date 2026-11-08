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
pub use planning::{AppendExecutions, CopyCycle, CreateCycle, CycleDefinition, RegisterCases};
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

// Keep The Journal Catalog Independent Of Described For The Retained Members.
macro_rules! transaction_catalog {
    (pub enum $name:ident { $($variant:ident($member:ty)),* $(,)? }) => {
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub enum $name { $($variant($member)),* }

        $(impl From<$member> for $name {
            fn from(value: $member) -> Self { Self::$variant(value) }
        })*

        impl $name {
            pub fn identify(&self) -> (harmos::DefinitionId, u32) {
                match self { $(Self::$variant(_) => <$member>::definition()),* }
            }

            pub fn encode_with<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                match self { $(Self::$variant(value) => value.serialize(serializer)),* }
            }

            pub fn restore<'de, D>(id: &harmos::DefinitionId, version: u32, payload: D) -> Result<Option<Self>, D::Error>
            where D: serde::Deserializer<'de> + Copy {
                $(if id.as_str() == <$member>::ID {
                    return Ok(<$member>::restore(version, payload)?.map(Self::$variant));
                })*
                Ok(None)
            }

            pub fn catalog() -> harmos::state::TransactionDefinitions<crate::state::RuntimeState, Self> {
                let definitions = vec![$({
                    let (id, version) = <$member>::definition();
                    (harmos::state::Kind::Transaction, id, version)
                }),*];
                harmos::state::TransactionDefinitions {
                    definitions,
                    decode: |id, version, bytes| {
                        let malformed = || harmos::state::DecodeError::Malformed { id: id.clone(), version };
                        let payload: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| malformed())?;
                        Self::restore(id, version, &payload).map_err(|_| malformed())
                    },
                    inverses: |_, _| None,
                }
            }
        }

        impl harmos::state::CatalogCodec for $name {
            fn identify(&self) -> (harmos::DefinitionId, u32) { self.identify() }
            fn encode<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { self.encode_with(serializer) }
        }

        impl harmos::state::Transaction<crate::state::RuntimeState> for $name {
            type Error = String;
            fn check(&self, state: &crate::state::RuntimeState) -> Result<(), String> {
                match self { $(Self::$variant(value) => harmos::state::Transaction::check(value, state)),* }
            }
            fn records(&self, state: &crate::state::RuntimeState, out: &mut harmos::state::Records) {
                match self { $(Self::$variant(value) => harmos::state::Transaction::records(value, state, out)),* }
            }
            fn apply(&self, state: &mut crate::state::RuntimeState) {
                match self { $(Self::$variant(value) => harmos::state::Transaction::apply(value, state)),* }
            }
        }

        impl harmos::state::TransactionCatalog<crate::state::RuntimeState> for $name {
            fn replay(&self, prepared: &harmos::state::Prepared, state: &mut crate::state::RuntimeState) -> Result<(), harmos::state::DecodeError> {
                let malformed = || harmos::state::DecodeError::Malformed { id: prepared.id.clone(), version: prepared.version };
                let value = Self::restore(&prepared.id, prepared.version, &prepared.payload).map_err(|_| malformed())?.ok_or_else(malformed)?;
                harmos::state::Transaction::apply(&value, state);
                Ok(())
            }
        }
    };
}

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

transaction_catalog! {
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
    CopyCycle(CopyCycle),
    RunProvider(RunProvider),
    PrepareAttempt(PrepareAttempt),
    DispatchProvider(DispatchProvider),
    CompleteRun(CompleteRun),
    SettleRun(SettleRun),
}
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
