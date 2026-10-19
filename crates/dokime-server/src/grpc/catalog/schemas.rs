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
use harmos::MemorySize;
use tonic::Status;

// Local Imports
use super::super::value::canonical;
use super::{Budget, unit, unit_memory, value};
use dokime_domain::model::identity::wire as common;
use dokime_domain::model::validation::ValidationMode;
use dokime_runtime::records::RecordContract;
use dokime_runtime::records::RecordIconKey;
use dokime_runtime::records::RecordSeverity;
use dokime_runtime::records::RecordTone;
use dokime_runtime::telemetry::TelemetryContract;
use dokime_runtime::telemetry::TelemetryRole;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn record(contract: &RecordContract) -> Result<dokime_domain::model::definition::wire::RecordSchema, Status> {
    record_memory(contract)?;
    value::text(contract.record_type.as_str())?;
    if let Some(id) = &contract.template_id {
        value::text(id.as_str())?;
    }
    value::collection(contract.fields.len())?;
    let mut budget = Budget::new();
    let fields = collect(contract.fields.iter().map(|field| {
        budget.node(0)?;
        value::collection(field.choices.len())?;
        Ok(dokime_domain::model::definition::wire::RecordField {
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
    Ok(dokime_domain::model::definition::wire::RecordSchema {
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
        display: Some(dokime_domain::model::definition::wire::RecordDisplay {
            icon: icon(&contract.display.icon),
            tone: tone(&contract.display.tone),
        }),
    })
}

pub(crate) fn telemetry(contract: &TelemetryContract) -> Result<dokime_domain::model::definition::wire::TelemetrySchema, Status> {
    telemetry_memory(contract)?;
    value::collection(contract.columns.len())?;
    value::text(contract.stream.as_str())?;
    if let Some(id) = &contract.template_id {
        value::text(id.as_str())?;
    }
    if let Some(x) = &contract.default_x {
        value::text(x)?;
    }
    let mut budget = Budget::new();
    let columns = collect(contract.columns.iter().map(|column| {
        budget.node(0)?;
        if let Some(unit) = &column.display_unit {
            value::text(unit)?;
        }
        Ok(dokime_domain::model::definition::wire::TelemetryColumn {
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
    Ok(dokime_domain::model::definition::wire::TelemetrySchema {
        storage_schema: contract.storage_schema.clone(),
        stream: contract.stream.as_str().to_owned(),
        schema_version: contract.schema_version,
        metadata: Some(metadata(&contract.metadata)?),
        template_identifier: contract.template_id.as_ref().map(|id| id.as_str().to_owned()),
        owner_suite_id: contract.owner.as_ref().map(|id| id.as_str().to_owned()),
        columns,
        default_x: contract.default_x.clone(),
        plot_window_ms: contract.plot_window.map(|window| window.0),
        allow_extra_fields: contract.allow_extra_fields,
        scalar: contract.scalar,
        validation_mode: contract.validation_mode.as_ref().map(validation),
        foreign: contract.foreign,
    })
}

pub(crate) fn record_memory(source: &RecordContract) -> Result<usize, Status> {
    value::text(source.record_type.as_str())?;
    if let Some(id) = &source.template_id {
        value::text(id.as_str())?;
    }
    value::collection(source.fields.len())?;
    let mut bytes = std::mem::size_of::<dokime_domain::model::definition::wire::RecordSchema>()
        .saturating_add(source.record_type.as_str().len())
        .saturating_add(metadata_heap(&source.metadata)?)
        .saturating_add(source.template_id.as_ref().map_or(0, |id| id.as_str().len()))
        .saturating_add(source.owner.as_ref().map_or(0, |id| id.as_str().len()))
        .saturating_add(source.fields.len().saturating_mul(std::mem::size_of::<dokime_domain::model::definition::wire::RecordField>()));
    let mut budget = Budget::new();
    for field in &source.fields {
        budget.node(0)?;
        value::collection(field.choices.len())?;
        bytes = bytes
            .saturating_add(metadata_heap(&field.metadata)?)
            .saturating_add(field.choices.len().saturating_mul(std::mem::size_of::<dokime_domain::model::values::wire::Value>()))
            .saturating_add(format_heap(field.format.as_ref()));
        if let Some(unit) = &field.unit {
            bytes = bytes.saturating_add(
                unit_memory(unit).map_err(failure)? - std::mem::size_of::<dokime_domain::model::definition::wire::MeasurementUnit>(),
            );
        }
        for choice in &field.choices {
            bytes = bytes.saturating_add(value_heap(choice, &mut budget)?);
        }
        if let Some(default) = &field.default {
            bytes = bytes.saturating_add(value_heap(default, &mut budget)?);
        }
    }
    bounded(bytes)
}

pub(crate) fn telemetry_memory(source: &TelemetryContract) -> Result<usize, Status> {
    if let Some(graph) = &source.storage_schema {
        graph.preparation(&harmos::config::SchemaSettings::default()).map_err(|error| Status::resource_exhausted(error.to_string()))?;
    }
    value::collection(source.columns.len())?;
    value::text(source.stream.as_str())?;
    if let Some(id) = &source.template_id {
        value::text(id.as_str())?;
    }
    if let Some(x) = &source.default_x {
        value::text(x)?;
    }
    let mut bytes = std::mem::size_of::<dokime_domain::model::definition::wire::TelemetrySchema>()
        .saturating_add(source.storage_schema.memory_size())
        .saturating_add(source.stream.as_str().len())
        .saturating_add(metadata_heap(&source.metadata)?)
        .saturating_add(source.template_id.as_ref().map_or(0, |id| id.as_str().len()))
        .saturating_add(source.owner.as_ref().map_or(0, |id| id.as_str().len()))
        .saturating_add(source.default_x.as_ref().map_or(0, String::len))
        .saturating_add(
            source.columns.len().saturating_mul(std::mem::size_of::<dokime_domain::model::definition::wire::TelemetryColumn>()),
        );
    let mut budget = Budget::new();
    for column in &source.columns {
        budget.node(0)?;
        bytes = bytes.saturating_add(metadata_heap(&column.metadata)?).saturating_add(format_heap(column.format.as_ref()));
        if let Some(unit) = &column.unit {
            bytes = bytes.saturating_add(
                unit_memory(unit).map_err(failure)? - std::mem::size_of::<dokime_domain::model::definition::wire::MeasurementUnit>(),
            );
        }
        if let Some(unit) = &column.display_unit {
            value::text(unit)?;
            bytes = bytes.saturating_add(unit.len());
        }
        if let Some(default) = &column.default {
            bytes = bytes.saturating_add(value_heap(default, &mut budget)?);
        }
    }
    bounded(bytes)
}

/* === Private Functions === */

fn failure(error: harmos::Error) -> Status {
    match error {
        harmos::Error::ResourceExhausted { .. } => Status::resource_exhausted("canonical schema construction exceeds its bound"),
        _ => Status::internal("canonical schema projection failed"),
    }
}

fn bounded(bytes: usize) -> Result<usize, Status> {
    if bytes == usize::MAX {
        return Err(Status::resource_exhausted("canonical schema construction overflows"));
    }
    Ok(bytes)
}

fn collect<T>(source: impl ExactSizeIterator<Item = Result<T, Status>>) -> Result<Vec<T>, Status> {
    let mut target = canonical::slots(source.len())?;
    for value in source {
        target.push(value?);
    }
    Ok(target)
}

fn metadata_heap(source: &dokime_domain::model::identity::Metadata) -> Result<usize, Status> {
    Ok(canonical::metadata_memory(source)? - std::mem::size_of::<common::Metadata>())
}

fn metadata(source: &dokime_domain::model::identity::Metadata) -> Result<common::Metadata, Status> {
    canonical::metadata(source)
}

fn value_heap(source: &dokime_domain::model::values::Value, budget: &mut Budget) -> Result<usize, Status> {
    Ok(canonical::memory(source, budget, 0)? - std::mem::size_of::<dokime_domain::model::values::wire::Value>())
}

fn project_value(
    source: &dokime_domain::model::values::Value, budget: &mut Budget, depth: usize,
) -> Result<dokime_domain::model::values::wire::Value, Status> {
    canonical::encode(source, budget, depth)
}

fn format_heap(source: Option<&dokime_domain::model::values::ValueFormat>) -> usize {
    match source {
        Some(dokime_domain::model::values::ValueFormat::AbsoluteTime { pattern }) => pattern.len(),
        _ => 0,
    }
}

fn validation(value: &ValidationMode) -> dokime_domain::model::definition::wire::ValidationMode {
    match value {
        ValidationMode::Strict => dokime_domain::model::definition::wire::ValidationMode::Strict,
        ValidationMode::Prototype => dokime_domain::model::definition::wire::ValidationMode::Prototype,
    }
}

fn icon(value: &RecordIconKey) -> dokime_domain::model::definition::wire::RecordIcon {
    match value {
        RecordIconKey::Activity => dokime_domain::model::definition::wire::RecordIcon::Activity,
        RecordIconKey::AlertTriangle => dokime_domain::model::definition::wire::RecordIcon::AlertTriangle,
        RecordIconKey::Archive => dokime_domain::model::definition::wire::RecordIcon::Archive,
        RecordIconKey::Bell => dokime_domain::model::definition::wire::RecordIcon::Bell,
        RecordIconKey::CheckCircle => dokime_domain::model::definition::wire::RecordIcon::CheckCircle,
        RecordIconKey::CircleDashed => dokime_domain::model::definition::wire::RecordIcon::CircleDashed,
        RecordIconKey::ClipboardCheck => dokime_domain::model::definition::wire::RecordIcon::ClipboardCheck,
        RecordIconKey::Cpu => dokime_domain::model::definition::wire::RecordIcon::Cpu,
        RecordIconKey::FileText => dokime_domain::model::definition::wire::RecordIcon::FileText,
        RecordIconKey::Flag => dokime_domain::model::definition::wire::RecordIcon::Flag,
        RecordIconKey::Gauge => dokime_domain::model::definition::wire::RecordIcon::Gauge,
        RecordIconKey::HeartPulse => dokime_domain::model::definition::wire::RecordIcon::HeartPulse,
        RecordIconKey::HelpCircle => dokime_domain::model::definition::wire::RecordIcon::HelpCircle,
        RecordIconKey::MessageSquareText => dokime_domain::model::definition::wire::RecordIcon::MessageSquareText,
        RecordIconKey::Package => dokime_domain::model::definition::wire::RecordIcon::Package,
        RecordIconKey::Plug => dokime_domain::model::definition::wire::RecordIcon::Plug,
        RecordIconKey::RadioTower => dokime_domain::model::definition::wire::RecordIcon::RadioTower,
        RecordIconKey::RotateCcw => dokime_domain::model::definition::wire::RecordIcon::RotateCcw,
        RecordIconKey::ShieldAlert => dokime_domain::model::definition::wire::RecordIcon::ShieldAlert,
        RecordIconKey::Target => dokime_domain::model::definition::wire::RecordIcon::Target,
        RecordIconKey::TimerReset => dokime_domain::model::definition::wire::RecordIcon::TimerReset,
        RecordIconKey::Wrench => dokime_domain::model::definition::wire::RecordIcon::Wrench,
        RecordIconKey::Zap => dokime_domain::model::definition::wire::RecordIcon::Zap,
        RecordIconKey::Clock => dokime_domain::model::definition::wire::RecordIcon::Clock,
    }
}

fn tone(value: &RecordTone) -> dokime_domain::model::definition::wire::RecordTone {
    match value {
        RecordTone::Info => dokime_domain::model::definition::wire::RecordTone::Info,
        RecordTone::Success => dokime_domain::model::definition::wire::RecordTone::Success,
        RecordTone::Warning => dokime_domain::model::definition::wire::RecordTone::Warning,
        RecordTone::Danger => dokime_domain::model::definition::wire::RecordTone::Danger,
        RecordTone::Accent => dokime_domain::model::definition::wire::RecordTone::Accent,
        RecordTone::Muted => dokime_domain::model::definition::wire::RecordTone::Muted,
        RecordTone::Neutral => dokime_domain::model::definition::wire::RecordTone::Neutral,
    }
}

fn role(value: &TelemetryRole) -> dokime_domain::model::definition::wire::TelemetryRole {
    match value {
        TelemetryRole::Value => dokime_domain::model::definition::wire::TelemetryRole::Value,
        TelemetryRole::Counter => dokime_domain::model::definition::wire::TelemetryRole::Counter,
        TelemetryRole::State => dokime_domain::model::definition::wire::TelemetryRole::State,
        TelemetryRole::Distance => dokime_domain::model::definition::wire::TelemetryRole::Distance,
        TelemetryRole::Latency => dokime_domain::model::definition::wire::TelemetryRole::Latency,
        TelemetryRole::Throughput => dokime_domain::model::definition::wire::TelemetryRole::Throughput,
    }
}

/* === Tests === */
