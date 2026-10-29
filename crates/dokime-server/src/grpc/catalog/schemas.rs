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

// Third Party Imports
use tonic::Status;

// Local Imports
use super::super::value::canonical;
use super::{Budget, unit};
use dokime::domain::model::identity::wire as common;
use dokime::domain::model::validation::ValidationMode;
use dokime::records::RecordContract;
use dokime::records::RecordIconKey;
use dokime::records::RecordSeverity;
use dokime::records::RecordTone;
use dokime::telemetry::TelemetryContract;
use dokime::telemetry::TelemetryRole;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn record(contract: &RecordContract) -> Result<dokime::domain::model::definition::wire::RecordSchema, Status> {
    let mut budget = Budget::new()?;
    let fields = collect(contract.fields.iter().map(|field| {
        budget.node(0)?;
        Ok(dokime::domain::model::definition::wire::RecordField {
            metadata: Some(metadata(&field.metadata)?),
            value_type: canonical::value_type(&field.value_type),
            required: field.required,
            nullable: field.nullable,
            unit: field.unit.as_ref().map(unit).transpose()?,
            choices: collect(field.choices.iter().map(|choice| project_value(choice, &mut budget, 0)))?,
            default_value: field.default.as_ref().map(|default| project_value(default, &mut budget, 0)).transpose()?,
            format: field.format.as_ref().map(canonical::value_format),
        })
    }))?;
    Ok(dokime::domain::model::definition::wire::RecordSchema {
        record_type: contract.record_type.as_str().to_owned(),
        schema_version: contract.schema_version,
        metadata: Some(metadata(&contract.metadata)?),
        template_identifier: contract.template_id.as_ref().map(|id| id.as_str().to_owned()),
        owner_suite_id: contract.owner.as_ref().map(|id| id.as_str().to_owned()),
        default_severity: match contract.default_severity {
            RecordSeverity::Debug => common::Severity::Debug,
            RecordSeverity::Info => common::Severity::Info,
            RecordSeverity::Warning => common::Severity::Warn,
            RecordSeverity::Error => common::Severity::Error,
        },
        fields,
        validation_mode: contract.validation_mode.as_ref().map(validation),
        allow_extra_fields: contract.allow_extra_fields,
        display: Some(dokime::domain::model::definition::wire::RecordDisplay {
            icon: icon(&contract.display.icon),
            tone: tone(&contract.display.tone),
        }),
    })
}

pub(crate) fn telemetry(
    contract: &TelemetryContract, schema_settings: &dokime::harmos::config::SchemaSettings,
) -> Result<dokime::domain::model::definition::wire::TelemetrySchema, Status> {
    if let Some(graph) = &contract.storage_schema {
        graph.preparation(schema_settings).map_err(|error| Status::resource_exhausted(error.to_string()))?;
    }
    let mut budget = Budget::new()?;
    let columns = collect(contract.columns.iter().map(|column| {
        budget.node(0)?;
        Ok(dokime::domain::model::definition::wire::TelemetryColumn {
            metadata: Some(metadata(&column.metadata)?),
            value_type: canonical::value_type(&column.value_type),
            role: role(&column.role),
            unit: column.unit.as_ref().map(unit).transpose()?,
            display_unit: column.display_unit.clone(),
            required: column.required,
            nullable: column.nullable,
            default_value: column.default.as_ref().map(|default| project_value(default, &mut budget, 0)).transpose()?,
            format: column.format.as_ref().map(canonical::value_format),
        })
    }))?;
    Ok(dokime::domain::model::definition::wire::TelemetrySchema {
        storage_schema: contract.storage_schema.clone(),
        stream: contract.stream.as_str().to_owned(),
        schema_version: contract.schema_version,
        metadata: Some(metadata(&contract.metadata)?),
        template_identifier: contract.template_id.as_ref().map(|id| id.as_str().to_owned()),
        owner_suite_id: contract.owner.as_ref().map(|id| id.as_str().to_owned()),
        columns,
        default_x: contract.default_x.clone(),
        allow_extra_fields: contract.allow_extra_fields,
        scalar: contract.scalar,
        validation_mode: contract.validation_mode.as_ref().map(validation),
        foreign: contract.foreign,
    })
}

/* === Private Functions === */

fn collect<T>(source: impl ExactSizeIterator<Item = Result<T, Status>>) -> Result<Vec<T>, Status> {
    let mut target = canonical::slots(source.len())?;
    for value in source {
        target.push(value?);
    }
    Ok(target)
}

fn metadata(source: &dokime::domain::model::identity::Metadata) -> Result<common::Metadata, Status> {
    canonical::metadata(source)
}

fn project_value(
    source: &dokime::domain::model::values::Value, budget: &mut Budget, depth: usize,
) -> Result<dokime::domain::model::values::wire::Value, Status> {
    canonical::encode(source, budget, depth)
}

fn validation(value: &ValidationMode) -> dokime::domain::model::definition::wire::ValidationMode {
    match value {
        ValidationMode::Strict => dokime::domain::model::definition::wire::ValidationMode::Strict,
        ValidationMode::Prototype => dokime::domain::model::definition::wire::ValidationMode::Prototype,
    }
}

fn icon(value: &RecordIconKey) -> dokime::domain::model::definition::wire::RecordIcon {
    match value {
        RecordIconKey::Activity => dokime::domain::model::definition::wire::RecordIcon::Activity,
        RecordIconKey::AlertTriangle => dokime::domain::model::definition::wire::RecordIcon::AlertTriangle,
        RecordIconKey::Archive => dokime::domain::model::definition::wire::RecordIcon::Archive,
        RecordIconKey::Bell => dokime::domain::model::definition::wire::RecordIcon::Bell,
        RecordIconKey::CheckCircle => dokime::domain::model::definition::wire::RecordIcon::CheckCircle,
        RecordIconKey::CircleDashed => dokime::domain::model::definition::wire::RecordIcon::CircleDashed,
        RecordIconKey::ClipboardCheck => dokime::domain::model::definition::wire::RecordIcon::ClipboardCheck,
        RecordIconKey::Cpu => dokime::domain::model::definition::wire::RecordIcon::Cpu,
        RecordIconKey::FileText => dokime::domain::model::definition::wire::RecordIcon::FileText,
        RecordIconKey::Flag => dokime::domain::model::definition::wire::RecordIcon::Flag,
        RecordIconKey::Gauge => dokime::domain::model::definition::wire::RecordIcon::Gauge,
        RecordIconKey::HeartPulse => dokime::domain::model::definition::wire::RecordIcon::HeartPulse,
        RecordIconKey::HelpCircle => dokime::domain::model::definition::wire::RecordIcon::HelpCircle,
        RecordIconKey::MessageSquareText => dokime::domain::model::definition::wire::RecordIcon::MessageSquareText,
        RecordIconKey::Package => dokime::domain::model::definition::wire::RecordIcon::Package,
        RecordIconKey::Plug => dokime::domain::model::definition::wire::RecordIcon::Plug,
        RecordIconKey::RadioTower => dokime::domain::model::definition::wire::RecordIcon::RadioTower,
        RecordIconKey::RotateCcw => dokime::domain::model::definition::wire::RecordIcon::RotateCcw,
        RecordIconKey::ShieldAlert => dokime::domain::model::definition::wire::RecordIcon::ShieldAlert,
        RecordIconKey::Target => dokime::domain::model::definition::wire::RecordIcon::Target,
        RecordIconKey::TimerReset => dokime::domain::model::definition::wire::RecordIcon::TimerReset,
        RecordIconKey::Wrench => dokime::domain::model::definition::wire::RecordIcon::Wrench,
        RecordIconKey::Zap => dokime::domain::model::definition::wire::RecordIcon::Zap,
        RecordIconKey::Clock => dokime::domain::model::definition::wire::RecordIcon::Clock,
    }
}

fn tone(value: &RecordTone) -> dokime::domain::model::definition::wire::RecordTone {
    match value {
        RecordTone::Info => dokime::domain::model::definition::wire::RecordTone::Info,
        RecordTone::Success => dokime::domain::model::definition::wire::RecordTone::Success,
        RecordTone::Warning => dokime::domain::model::definition::wire::RecordTone::Warning,
        RecordTone::Danger => dokime::domain::model::definition::wire::RecordTone::Danger,
        RecordTone::Accent => dokime::domain::model::definition::wire::RecordTone::Accent,
        RecordTone::Muted => dokime::domain::model::definition::wire::RecordTone::Muted,
        RecordTone::Neutral => dokime::domain::model::definition::wire::RecordTone::Neutral,
    }
}

fn role(value: &TelemetryRole) -> dokime::domain::model::definition::wire::TelemetryRole {
    match value {
        TelemetryRole::Value => dokime::domain::model::definition::wire::TelemetryRole::Value,
        TelemetryRole::Counter => dokime::domain::model::definition::wire::TelemetryRole::Counter,
        TelemetryRole::State => dokime::domain::model::definition::wire::TelemetryRole::State,
        TelemetryRole::Distance => dokime::domain::model::definition::wire::TelemetryRole::Distance,
        TelemetryRole::Latency => dokime::domain::model::definition::wire::TelemetryRole::Latency,
        TelemetryRole::Throughput => dokime::domain::model::definition::wire::TelemetryRole::Throughput,
    }
}

/* === Tests === */
