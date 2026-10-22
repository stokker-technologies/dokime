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
use std::sync::LazyLock;

// Third Party Imports

// Local Imports
use crate::model::identity::Identifier;
use crate::model::identity::Metadata;
use crate::model::records::RecordContract;
use crate::model::records::RecordDisplay;
use crate::model::records::RecordField;
use crate::model::records::RecordIconKey;
use crate::model::records::RecordSeverity;
use crate::model::records::RecordTone;
use crate::model::values::Value;
use crate::model::values::ValueType;

/* === Constants === */

pub const PRODUCT_CONNECTION_STATES: &[&str] = &["connected", "disconnected"];
pub const PRODUCT_READINESS_STATES: &[&str] = &["ready", "not_ready", "busy", "unknown"];
pub const PRODUCT_TARGET_LOCK_STATES: &[&str] = &["locked", "lost", "unknown"];
pub const PRODUCT_CALIBRATION_KINDS: &[&str] = &["field", "tilt"];
pub const PRODUCT_CALIBRATION_PHASES: &[&str] = &["started", "activity", "completed"];

/* === Macros === */

macro_rules! builtin_record_types {
    ($($variant:ident => $identifier:literal,)+) => {
                                #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum BuiltinRecordType {
            $($variant,)+
        }

        impl BuiltinRecordType {
                        pub const ALL: &'static [Self] = &[$(Self::$variant,)+];

            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $identifier,)+
                }
            }
        }
    };
}

builtin_record_types! {
    SessionStarted => "session.started",
    SessionRunning => "session.running",
    SessionStopping => "session.stopping",
    SessionCompleted => "session.completed",
    SessionFailed => "session.failed",
    SessionDisconnected => "session.disconnected",
    TestCyclePrepared => "test.cycle.prepared",
    TestCycleQueued => "test.cycle.queued",
    TestExecutionPrepared => "test.execution.prepared",
    TestCycleStarted => "test.cycle.started",
    TestCycleCompleted => "test.cycle.completed",
    TestCycleFailed => "test.cycle.failed",
    TestCycleFinalizationFailed => "test.cycle.finalization.failed",
    TestCycleStopRequested => "test.cycle.stop_requested",
    TestCycleStopped => "test.cycle.stopped",
    TestExecutionStarted => "test.execution.started",
    TestExecutionPaused => "test.execution.paused",
    TestExecutionResumed => "test.execution.resumed",
    TestExecutionCompleted => "test.execution.completed",
    TestExecutionFailed => "test.execution.failed",
    TestExecutionFinalizationFailed => "test.execution.finalization.failed",
    TestExecutionInterrupted => "test.execution.interrupted",
    TestExecutionStopped => "test.execution.stopped",
    TestExecutionStopRequested => "test.execution.stop_requested",
    TestExecutionBlocked => "test.execution.blocked",
    TestExecutionPauseRequested => "test.execution.pause_requested",
    TestExecutionResumeRequested => "test.execution.resume_requested",
    TestStepStarted => "test.step.started",
    TestStepCompleted => "test.step.completed",
    TestStepFailed => "test.step.failed",
    TestStepHandleCancelled => "test.step.handle_cancelled",
    TestCriterionStarted => "test.criterion.started",
    TestCriterionCompleted => "test.criterion.completed",
    TestCriterionFailed => "test.criterion.failed",
    TestInitializeStarted => "test.initialize.started",
    TestInitializeCompleted => "test.initialize.completed",
    TestInitializeFailed => "test.initialize.failed",
    TestFinalizeStarted => "test.finalize.started",
    TestFinalizeCompleted => "test.finalize.completed",
    TestFinalizeFailed => "test.finalize.failed",
    TestInterruptStarted => "test.interrupt.started",
    TestInterruptCompleted => "test.interrupt.completed",
    TestInterruptFailed => "test.interrupt.failed",
    TestMeasurementFailed => "test.measurement.failed",
    ArtifactCreated => "artifact.created",
    ArtifactCompleted => "artifact.completed",
    ReportRequested => "report.requested",
    ReportCompleted => "report.completed",
    ReportFailed => "report.failed",
    WorkerStarted => "worker.started",
    WorkerFailed => "worker.failed",
    InstalledPluginActivated => "installed_plugin.activated",
    InstalledPluginFailed => "installed_plugin.failed",
    ProviderExecutionStarted => "provider.execution.started",
    ProviderExecutionCompleted => "provider.execution.completed",
    ProviderExecutionFailed => "provider.execution.failed",
    OperatorIdentified => "operator.identified",
    ResourceRegistered => "resource.registered",
    ResourceAttached => "resource.attached",
    ResourceOpening => "resource.opening",
    ResourceOpened => "resource.opened",
    ResourceTraceabilityUpdated => "resource.traceability.updated",
    ResourceClosed => "resource.closed",
    ResourceFailed => "resource.failed",
    ResourceRemoved => "resource.removed",
    ToolConnectionAcquired => "tool.connection.acquired",
    ToolConnectionLost => "tool.connection.lost",
    TargetLost => "target.lost",
    TargetAcquired => "target.acquired",
    CalibrationFieldStarted => "calibration.field.started",
    CalibrationFieldActivity => "calibration.field.activity",
    CalibrationFieldCompleted => "calibration.field.completed",
    CalibrationTiltStarted => "calibration.tilt.started",
    CalibrationTiltActivity => "calibration.tilt.activity",
    CalibrationTiltCompleted => "calibration.tilt.completed",
    ControlTriggered => "control.triggered",
    ControlCallbackCompleted => "control.callback_completed",
    ControlCallbackFailed => "control.callback_failed",
    PromptRequestConfirmation => "prompt.request.confirmation",
    PromptRequestForm => "prompt.request.form",
    PromptRequestChoice => "prompt.request.choice",
    PromptRequestStatus => "prompt.request.status",
    PromptResponseSubmitted => "prompt.response.submitted",
    PromptResponseCancelled => "prompt.response.cancelled",
    PromptResponseExpired => "prompt.response.expired",
    OperatorNoteAdded => "operator.note_added",
    OperatorNoteEdited => "operator.note_edited",
    OperatorLabelEdited => "operator.label_edited",
    ReportOmissions => "report.omissions",
    HealthAdded => "health.added",
    ProgressMilestone => "progress.milestone",
    ProcessExited => "process.exited",
}

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl BuiltinRecordType {
    pub fn identifier(self) -> Identifier {
        identifier(self.as_str())
    }

    pub fn from_identifier(record_type: &Identifier) -> Option<Self> {
        Self::ALL.iter().copied().find(|kind| kind.as_str() == record_type.as_str())
    }

    pub const fn schema_version(self) -> u32 {
        match self {
            Self::ToolConnectionAcquired
            | Self::ToolConnectionLost
            | Self::TargetLost
            | Self::TargetAcquired
            | Self::CalibrationFieldStarted
            | Self::CalibrationFieldActivity
            | Self::CalibrationFieldCompleted
            | Self::CalibrationTiltStarted
            | Self::CalibrationTiltActivity
            | Self::CalibrationTiltCompleted => 2,
            _ => 1,
        }
    }

    pub fn contract(self) -> RecordContract {
        let record_type = self.as_str();
        match self {
            Self::SessionStarted
            | Self::SessionRunning
            | Self::SessionStopping
            | Self::SessionCompleted
            | Self::SessionFailed
            | Self::SessionDisconnected
            | Self::TestCyclePrepared
            | Self::TestCycleQueued
            | Self::TestExecutionPrepared
            | Self::TestCycleStarted
            | Self::TestCycleCompleted
            | Self::TestCycleFailed
            | Self::TestCycleFinalizationFailed
            | Self::TestCycleStopRequested
            | Self::TestCycleStopped
            | Self::TestExecutionStarted
            | Self::TestExecutionPaused
            | Self::TestExecutionResumed
            | Self::TestExecutionCompleted
            | Self::TestExecutionFailed
            | Self::TestExecutionFinalizationFailed
            | Self::TestExecutionInterrupted
            | Self::TestExecutionStopped
            | Self::TestExecutionStopRequested
            | Self::TestExecutionBlocked
            | Self::TestExecutionPauseRequested
            | Self::TestExecutionResumeRequested
            | Self::TestStepStarted
            | Self::TestStepCompleted
            | Self::TestStepFailed
            | Self::TestCriterionStarted
            | Self::TestCriterionCompleted
            | Self::TestCriterionFailed
            | Self::TestInitializeStarted
            | Self::TestInitializeCompleted
            | Self::TestInitializeFailed
            | Self::TestFinalizeStarted
            | Self::TestFinalizeCompleted
            | Self::TestFinalizeFailed
            | Self::TestInterruptStarted
            | Self::TestInterruptCompleted
            | Self::TestInterruptFailed
            | Self::TestMeasurementFailed
            | Self::ArtifactCreated
            | Self::ArtifactCompleted
            | Self::ReportRequested
            | Self::ReportCompleted
            | Self::ReportFailed
            | Self::WorkerStarted
            | Self::WorkerFailed
            | Self::InstalledPluginActivated
            | Self::InstalledPluginFailed
            | Self::ProviderExecutionStarted
            | Self::ProviderExecutionCompleted
            | Self::ProviderExecutionFailed
            | Self::OperatorIdentified
            | Self::ResourceRegistered
            | Self::ResourceAttached
            | Self::ResourceOpening
            | Self::ResourceOpened
            | Self::ResourceTraceabilityUpdated
            | Self::ResourceClosed
            | Self::ResourceFailed
            | Self::ResourceRemoved => simple_contract(record_type, RecordSeverity::Info),
            Self::ToolConnectionAcquired | Self::ToolConnectionLost => product_connection_contract(record_type),
            Self::TargetLost | Self::TargetAcquired => product_target_contract(record_type),
            Self::CalibrationFieldStarted
            | Self::CalibrationFieldActivity
            | Self::CalibrationFieldCompleted
            | Self::CalibrationTiltStarted
            | Self::CalibrationTiltActivity
            | Self::CalibrationTiltCompleted => product_calibration_contract(record_type),
            Self::ControlTriggered => {
                contract(record_type, RecordSeverity::Info, vec![field("control_id", ValueType::Text), field("values", ValueType::Object)])
            }
            Self::ControlCallbackCompleted => contract(record_type, RecordSeverity::Info, vec![field("control_id", ValueType::Text)]),
            Self::ControlCallbackFailed => {
                contract(record_type, RecordSeverity::Error, vec![field("control_id", ValueType::Text), field("error", ValueType::Text)])
            }
            Self::PromptRequestConfirmation | Self::PromptRequestForm | Self::PromptRequestChoice | Self::PromptRequestStatus => {
                prompt_request_contract(record_type)
            }
            Self::PromptResponseSubmitted => prompt_response_contract(record_type, RecordSeverity::Info),
            Self::PromptResponseCancelled => prompt_cancel_contract(record_type, RecordSeverity::Info),
            Self::PromptResponseExpired => prompt_cancel_contract(record_type, RecordSeverity::Warning),
            Self::OperatorNoteAdded | Self::OperatorNoteEdited => {
                contract(record_type, RecordSeverity::Info, vec![field("note_id", ValueType::Text)])
            }
            // An operator renamed a cycle or an execution; the record scope names which.
            Self::OperatorLabelEdited => {
                contract(record_type, RecordSeverity::Info, vec![field("old_label", ValueType::Text), field("new_label", ValueType::Text)])
            }
            // A sealed report that left evidence out to fit its budget.
            Self::ReportOmissions => contract(
                record_type,
                RecordSeverity::Warning,
                vec![
                    field("test_cycle_id", ValueType::Text),
                    field("test_execution_id", ValueType::Text),
                    field("omissions", ValueType::Array),
                ],
            ),
            Self::HealthAdded => contract(
                record_type,
                RecordSeverity::Warning,
                vec![
                    field("health_id", ValueType::Text),
                    field("title", ValueType::Text),
                    field("level", ValueType::Text).with_default(Value::from("warning")),
                    field("related_artifact_id", ValueType::Text).nullable(),
                ],
            ),
            // A process a step ran exited, by itself or cancelled.
            Self::ProcessExited => contract(
                record_type,
                RecordSeverity::Info,
                vec![
                    field("program", ValueType::Text),
                    field("exit_code", ValueType::Integer).nullable(),
                    field("duration_ms", ValueType::Integer),
                    field("cancelled", ValueType::Boolean).with_default(Value::from(false)),
                ],
            ),
            // A process, watch or submitted operation still running when its
            // step ended, which the step-end sweep cancelled.
            Self::TestStepHandleCancelled => contract(
                record_type,
                RecordSeverity::Warning,
                vec![field("step", ValueType::Text), field("kind", ValueType::Text), field("name", ValueType::Text)],
            ),
            Self::ProgressMilestone => contract(
                record_type,
                RecordSeverity::Info,
                vec![field("phase", ValueType::Text), field("progress", ValueType::Number).optional()],
            ),
        }
    }
}

impl From<BuiltinRecordType> for Identifier {
    fn from(kind: BuiltinRecordType) -> Self {
        kind.identifier()
    }
}

impl std::fmt::Display for BuiltinRecordType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/* === Public Functions === */

pub fn builtin_record_types() -> &'static [Identifier] {
    static TYPES: LazyLock<Vec<Identifier>> =
        LazyLock::new(|| builtin_record_contracts().into_iter().map(|contract| contract.record_type).collect());

    &TYPES
}

pub fn builtin_record_contracts() -> Vec<RecordContract> {
    BuiltinRecordType::ALL.iter().map(|kind| kind.contract()).collect()
}

/* === Private Functions === */

fn simple_contract(record_type: &str, severity: RecordSeverity) -> RecordContract {
    contract(record_type, severity, Vec::new())
}

fn contract(record_type: &str, default_severity: RecordSeverity, fields: Vec<RecordField>) -> RecordContract {
    RecordContract {
        record_type: identifier(record_type),
        schema_version: 1,
        metadata: metadata(record_type),
        display: display_for(record_type, default_severity),
        template_id: None,
        owner: None,
        default_severity,
        fields,
        validation_mode: None,
        allow_extra_fields: true,
    }
}

fn product_connection_contract(record_type: &str) -> RecordContract {
    let mut contract = product_contract(
        record_type,
        vec![
            field("resource_id", ValueType::Text),
            field("resource_kind", ValueType::Text),
            choice_field("connection_state", PRODUCT_CONNECTION_STATES),
            choice_field("previous_connection_state", PRODUCT_CONNECTION_STATES).optional(),
            choice_field("readiness_state", PRODUCT_READINESS_STATES).optional(),
            choice_field("target_lock_state", PRODUCT_TARGET_LOCK_STATES).optional(),
            field("connection_transport", ValueType::Text).optional(),
            field("connection_endpoint", ValueType::Text).optional(),
            field("status_source", ValueType::Text).optional(),
            field("status_message", ValueType::Text).optional(),
            field("device_timestamp", ValueType::Text).optional(),
        ],
    );
    contract.display = if record_type.ends_with("lost") {
        RecordDisplay::new(RecordIconKey::Plug, RecordTone::Warning)
    } else {
        RecordDisplay::new(RecordIconKey::Plug, RecordTone::Success)
    };
    contract
}

fn product_target_contract(record_type: &str) -> RecordContract {
    let mut contract = product_contract(
        record_type,
        vec![
            field("resource_id", ValueType::Text),
            field("resource_kind", ValueType::Text),
            choice_field("target_lock_state", PRODUCT_TARGET_LOCK_STATES),
            choice_field("connection_state", PRODUCT_CONNECTION_STATES).optional(),
            choice_field("readiness_state", PRODUCT_READINESS_STATES).optional(),
            field("status_source", ValueType::Text).optional(),
            field("status_message", ValueType::Text).optional(),
            field("device_timestamp", ValueType::Text).optional(),
        ],
    );
    contract.display = if record_type.ends_with("lost") {
        RecordDisplay::new(RecordIconKey::Target, RecordTone::Danger)
    } else {
        RecordDisplay::new(RecordIconKey::Target, RecordTone::Success)
    };
    contract
}

fn product_calibration_contract(record_type: &str) -> RecordContract {
    let mut contract = product_contract(
        record_type,
        vec![
            field("resource_id", ValueType::Text),
            field("resource_kind", ValueType::Text),
            choice_field("calibration_kind", PRODUCT_CALIBRATION_KINDS),
            choice_field("calibration_phase", PRODUCT_CALIBRATION_PHASES),
            field("calibration_state", ValueType::Text).optional(),
            field("calibration_source", ValueType::Text).optional(),
            field("calibration_confidence", ValueType::Number).optional(),
            choice_field("connection_state", PRODUCT_CONNECTION_STATES).optional(),
            choice_field("readiness_state", PRODUCT_READINESS_STATES).optional(),
            field("status_source", ValueType::Text).optional(),
            field("status_message", ValueType::Text).optional(),
            field("device_timestamp", ValueType::Text).optional(),
            field("raw_calibration_kind", ValueType::Text).optional(),
        ],
    );
    contract.display = if record_type.ends_with("completed") {
        RecordDisplay::new(RecordIconKey::Gauge, RecordTone::Success)
    } else if record_type.ends_with("activity") {
        RecordDisplay::new(RecordIconKey::Gauge, RecordTone::Accent)
    } else {
        RecordDisplay::new(RecordIconKey::Gauge, RecordTone::Info)
    };
    contract
}

fn product_contract(record_type: &str, fields: Vec<RecordField>) -> RecordContract {
    let mut contract = contract(record_type, RecordSeverity::Info, fields);
    contract.schema_version = 2;
    contract.allow_extra_fields = false;
    contract
}

fn prompt_request_contract(record_type: &str) -> RecordContract {
    let mut contract = contract(
        record_type,
        RecordSeverity::Info,
        vec![
            field("prompt_id", ValueType::Text),
            field("contract_id", ValueType::Text),
            field("contract_identifier", ValueType::Text),
            field("prompt_type", ValueType::Text),
            field("template_identifier", ValueType::Text).optional(),
            field("title", ValueType::Text),
            field("message", ValueType::Text).optional(),
            field("kind", ValueType::Text),
            field("fields", ValueType::Array),
            field("actions", ValueType::Array),
            field("allow_timeout", ValueType::Boolean),
            field("created_at", ValueType::Text),
            field("expires_at", ValueType::Text).optional(),
            field("context", ValueType::Object),
        ],
    );
    contract.allow_extra_fields = false;
    contract
}

fn prompt_response_contract(record_type: &str, severity: RecordSeverity) -> RecordContract {
    let mut contract = contract(
        record_type,
        severity,
        vec![
            field("prompt_id", ValueType::Text),
            field("action", ValueType::Text),
            field("responder", ValueType::Text),
            field("source", ValueType::Text),
            field("accepted", ValueType::Boolean),
            field("values", ValueType::Object),
        ],
    );
    contract.allow_extra_fields = false;
    contract
}

fn prompt_cancel_contract(record_type: &str, severity: RecordSeverity) -> RecordContract {
    let mut contract = prompt_response_contract(record_type, severity);
    contract.fields.push(field("reason", ValueType::Text).optional());
    contract
}

fn field(name: &str, value_type: ValueType) -> RecordField {
    RecordField::new(metadata(name), value_type)
}

fn choice_field(name: &str, choices: &[&str]) -> RecordField {
    field(name, ValueType::Text).with_choices(choices.iter().copied().map(Value::from))
}

fn metadata(identifier: &str) -> Metadata {
    Metadata::new(
        Identifier::new(identifier).expect("built-in identifiers are valid"),
        label(identifier),
        Some(format!("Built-in record contract for {identifier}.")),
    )
}

fn identifier(value: &str) -> Identifier {
    Identifier::new(value).expect("built-in identifiers are valid")
}

fn display_for(record_type: &str, severity: RecordSeverity) -> RecordDisplay {
    if record_type.contains("failed") {
        return RecordDisplay::new(RecordIconKey::AlertTriangle, RecordTone::Danger);
    }
    if record_type.contains("completed") {
        return RecordDisplay::new(RecordIconKey::CheckCircle, RecordTone::Success);
    }
    if record_type.contains("stopped")
        || record_type.contains("stopping")
        || record_type.contains("stop_requested")
        || record_type.contains("blocked")
    {
        return RecordDisplay::new(RecordIconKey::CircleDashed, RecordTone::Warning);
    }

    let icon = if record_type.starts_with("run.") {
        RecordIconKey::Activity
    } else if record_type.starts_with("test.cycle.") {
        RecordIconKey::Flag
    } else if record_type.starts_with("test.execution.") {
        RecordIconKey::ClipboardCheck
    } else if record_type.starts_with("test.step.") || record_type.starts_with("test.criterion.") {
        RecordIconKey::CheckCircle
    } else if record_type.starts_with("test.initialize.")
        || record_type.starts_with("test.finalize.")
        || record_type.starts_with("test.interrupt.")
    {
        RecordIconKey::Wrench
    } else if record_type.starts_with("test.measurement.") {
        RecordIconKey::Gauge
    } else if record_type.starts_with("artifact.") {
        RecordIconKey::FileText
    } else if record_type.starts_with("report.") {
        RecordIconKey::Archive
    } else if record_type.starts_with("worker.") || record_type.starts_with("installed_plugin.") {
        RecordIconKey::Cpu
    } else if record_type.starts_with("operator.") {
        RecordIconKey::MessageSquareText
    } else if record_type.starts_with("control.") {
        RecordIconKey::Zap
    } else if record_type.starts_with("prompt.") {
        RecordIconKey::Bell
    } else if record_type.starts_with("health.") {
        RecordIconKey::HeartPulse
    } else if record_type.starts_with("progress.") {
        RecordIconKey::TimerReset
    } else if record_type.starts_with("process.") {
        RecordIconKey::Cpu
    } else {
        RecordIconKey::Clock
    };

    RecordDisplay::new(icon, tone_for(severity))
}

fn tone_for(severity: RecordSeverity) -> RecordTone {
    match severity {
        RecordSeverity::Debug => RecordTone::Muted,
        RecordSeverity::Info => RecordTone::Info,
        RecordSeverity::Warning => RecordTone::Warning,
        RecordSeverity::Error => RecordTone::Danger,
    }
}

fn label(identifier: &str) -> String {
    identifier
        .split(['.', '_', '-'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            first.to_uppercase().chain(chars).collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_catalog_unique() {
        let catalog = builtin_record_contracts();
        assert_eq!(catalog.len(), BuiltinRecordType::ALL.len());
        for kind in BuiltinRecordType::ALL {
            let declared = catalog.iter().filter(|contract| contract.record_type.as_str() == kind.as_str()).collect::<Vec<_>>();
            assert_eq!(declared.len(), 1, "{kind} is registered exactly once");
            assert_eq!(declared[0], &kind.contract());
            assert_eq!(declared[0].schema_version, kind.schema_version());
            assert_eq!(BuiltinRecordType::from_identifier(&kind.identifier()), Some(*kind));
            assert!(builtin_record_types().contains(&kind.identifier()));
        }
        for contract in &catalog {
            assert!(
                BuiltinRecordType::from_identifier(&contract.record_type).is_some(),
                "{} is declared as a built-in record type",
                contract.record_type
            );
        }
    }
}
