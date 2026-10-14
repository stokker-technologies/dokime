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
use crate::core::{Identifier, Metadata, Value, ValueType};
use crate::records::{
    RecordContract, RecordDisplay, RecordField, RecordIconKey, RecordSeverity, RecordTone,
};

/* === Constants === */
const SIMPLE_LIFECYCLE_RECORDS: &[&str] = &[
    "session.started",
    "session.running",
    "session.stopping",
    "session.completed",
    "session.failed",
    "session.disconnected",
    "test.cycle.prepared",
    "test.cycle.queued",
    "test.execution.prepared",
    "test.cycle.started",
    "test.cycle.completed",
    "test.cycle.failed",
    "test.cycle.finalization.failed",
    "test.cycle.stop_requested",
    "test.cycle.stopped",
    "test.execution.started",
    "test.execution.paused",
    "test.execution.resumed",
    "test.execution.completed",
    "test.execution.failed",
    "test.execution.finalization.failed",
    "test.execution.interrupted",
    "test.execution.stopped",
    "test.execution.stop_requested",
    "test.execution.blocked",
    "test.execution.pause_requested",
    "test.execution.resume_requested",
    "test.step.started",
    "test.step.completed",
    "test.step.failed",
    "test.criterion.started",
    "test.criterion.completed",
    "test.criterion.failed",
    "test.initialize.started",
    "test.initialize.completed",
    "test.initialize.failed",
    "test.finalize.started",
    "test.finalize.completed",
    "test.finalize.failed",
    "test.interrupt.started",
    "test.interrupt.completed",
    "test.interrupt.failed",
    "test.measurement.failed",
    "artifact.created",
    "artifact.completed",
    "report.requested",
    "report.completed",
    "report.failed",
    "worker.started",
    "worker.failed",
    "installed_plugin.activated",
    "installed_plugin.failed",
    "provider.execution.started",
    "provider.execution.completed",
    "provider.execution.failed",
    "operator.identified",
    "resource.registered",
    "resource.attached",
    "resource.opening",
    "resource.opened",
    "resource.traceability.updated",
    "resource.closed",
    "resource.failed",
    "resource.removed",
];
pub const PRODUCT_CONNECTION_STATES: &[&str] = &["connected", "disconnected"];
pub const PRODUCT_READINESS_STATES: &[&str] = &["ready", "not_ready", "busy", "unknown"];
pub const PRODUCT_TARGET_LOCK_STATES: &[&str] = &["locked", "lost", "unknown"];
pub const PRODUCT_CALIBRATION_KINDS: &[&str] = &["field", "tilt"];
pub const PRODUCT_CALIBRATION_PHASES: &[&str] = &["started", "activity", "completed"];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn builtin_record_types() -> &'static [Identifier] {
    static TYPES: LazyLock<Vec<Identifier>> = LazyLock::new(|| {
        builtin_record_contracts()
            .into_iter()
            .map(|contract| contract.record_type)
            .collect()
    });

    &TYPES
}

pub fn builtin_record_contracts() -> Vec<RecordContract> {
    let mut contracts: Vec<RecordContract> = SIMPLE_LIFECYCLE_RECORDS
        .iter()
        .map(|record_type| simple_contract(record_type, RecordSeverity::Info))
        .collect();

    contracts.extend([
        product_connection_contract("tool.connection.acquired"),
        product_connection_contract("tool.connection.lost"),
        product_target_contract("target.lost"),
        product_target_contract("target.acquired"),
        product_calibration_contract("calibration.field.started"),
        product_calibration_contract("calibration.field.activity"),
        product_calibration_contract("calibration.field.completed"),
        product_calibration_contract("calibration.tilt.started"),
        product_calibration_contract("calibration.tilt.activity"),
        product_calibration_contract("calibration.tilt.completed"),
        contract(
            "control.triggered",
            RecordSeverity::Info,
            vec![
                field("control_id", ValueType::Text),
                field("values", ValueType::Object),
            ],
        ),
        contract(
            "control.callback_completed",
            RecordSeverity::Info,
            vec![field("control_id", ValueType::Text)],
        ),
        contract(
            "control.callback_failed",
            RecordSeverity::Error,
            vec![
                field("control_id", ValueType::Text),
                field("error", ValueType::Text),
            ],
        ),
        prompt_request_contract("prompt.request.confirmation"),
        prompt_request_contract("prompt.request.form"),
        prompt_request_contract("prompt.request.choice"),
        prompt_request_contract("prompt.request.status"),
        prompt_response_contract("prompt.response.submitted", RecordSeverity::Info),
        prompt_cancel_contract("prompt.response.cancelled", RecordSeverity::Info),
        prompt_cancel_contract("prompt.response.expired", RecordSeverity::Warning),
        contract(
            "operator.note_added",
            RecordSeverity::Info,
            vec![field("note_id", ValueType::Text)],
        ),
        contract(
            "operator.note_edited",
            RecordSeverity::Info,
            vec![field("note_id", ValueType::Text)],
        ),
        contract(
            "health.added",
            RecordSeverity::Warning,
            vec![
                field("health_id", ValueType::Text),
                field("title", ValueType::Text),
                field("level", ValueType::Text).with_default(Value::from("warning")),
                field("related_artifact_id", ValueType::Text).nullable(),
            ],
        ),
        contract(
            "progress.milestone",
            RecordSeverity::Info,
            vec![
                field("phase", ValueType::Text),
                field("progress", ValueType::Number).optional(),
            ],
        ),
    ]);

    contracts
}

/* === Private Functions === */
fn simple_contract(record_type: &str, severity: RecordSeverity) -> RecordContract {
    contract(record_type, severity, Vec::new())
}

fn contract(
    record_type: &str,
    default_severity: RecordSeverity,
    fields: Vec<RecordField>,
) -> RecordContract {
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
    contract
        .fields
        .push(field("reason", ValueType::Text).optional());
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
