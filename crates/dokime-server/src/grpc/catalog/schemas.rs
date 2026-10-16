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
use dokime_app::{
    core::ValidationMode,
    records::{RecordContract, RecordIconKey, RecordSeverity, RecordTone},
    telemetry::{TelemetryContract, TelemetryRole},
};
use dokime_contract::operations::{common, schema as wire, value as wire_value};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn record(contract: &RecordContract) -> Result<wire::RecordSchema, Status> {
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
        Ok(wire::RecordField {
            metadata: Some(metadata(&field.metadata)?),
            value_type: canonical::value_type(&field.value_type),
            required: field.required,
            nullable: field.nullable,
            unit: field.unit.as_ref().map(unit).transpose()?,
            choices: collect(
                field
                    .choices
                    .iter()
                    .map(|choice| project_value(choice, &mut budget, 0)),
            )?,
            default_value: field
                .default
                .as_ref()
                .map(|default| project_value(default, &mut budget, 0))
                .transpose()?,
            format: field.format.as_ref().map(canonical::value_format),
        })
    }))?;
    Ok(wire::RecordSchema {
        record_type: contract.record_type.as_str().to_owned(),
        schema_version: contract.schema_version,
        metadata: Some(metadata(&contract.metadata)?),
        template_identifier: contract
            .template_id
            .as_ref()
            .map(|id| id.as_str().to_owned()),
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
        display: Some(wire::RecordDisplay {
            icon: icon(&contract.display.icon),
            tone: tone(&contract.display.tone),
        }),
    })
}

pub(crate) fn telemetry(contract: &TelemetryContract) -> Result<wire::TelemetrySchema, Status> {
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
        Ok(wire::TelemetryColumn {
            metadata: Some(metadata(&column.metadata)?),
            value_type: canonical::value_type(&column.value_type),
            role: role(&column.role),
            unit: column.unit.as_ref().map(unit).transpose()?,
            display_unit: column.display_unit.clone(),
            required: column.required,
            nullable: column.nullable,
            default_value: column
                .default
                .as_ref()
                .map(|default| project_value(default, &mut budget, 0))
                .transpose()?,
            format: column.format.as_ref().map(canonical::value_format),
        })
    }))?;
    Ok(wire::TelemetrySchema {
        storage_schema: contract.storage_schema.clone(),
        stream: contract.stream.as_str().to_owned(),
        schema_version: contract.schema_version,
        metadata: Some(metadata(&contract.metadata)?),
        template_identifier: contract
            .template_id
            .as_ref()
            .map(|id| id.as_str().to_owned()),
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
    let mut bytes = std::mem::size_of::<wire::RecordSchema>()
        .saturating_add(source.record_type.as_str().len())
        .saturating_add(metadata_heap(&source.metadata)?)
        .saturating_add(
            source
                .template_id
                .as_ref()
                .map_or(0, |id| id.as_str().len()),
        )
        .saturating_add(source.owner.as_ref().map_or(0, |id| id.as_str().len()))
        .saturating_add(
            source
                .fields
                .len()
                .saturating_mul(std::mem::size_of::<wire::RecordField>()),
        );
    let mut budget = Budget::new();
    for field in &source.fields {
        budget.node(0)?;
        value::collection(field.choices.len())?;
        bytes = bytes
            .saturating_add(metadata_heap(&field.metadata)?)
            .saturating_add(
                field
                    .choices
                    .len()
                    .saturating_mul(std::mem::size_of::<wire_value::Value>()),
            )
            .saturating_add(format_heap(field.format.as_ref()));
        if let Some(unit) = &field.unit {
            bytes = bytes.saturating_add(
                unit_memory(unit).map_err(failure)? - std::mem::size_of::<wire::MeasurementUnit>(),
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
        graph
            .preparation(&harmos::config::SchemaSettings::default())
            .map_err(|error| Status::resource_exhausted(error.to_string()))?;
    }
    value::collection(source.columns.len())?;
    value::text(source.stream.as_str())?;
    if let Some(id) = &source.template_id {
        value::text(id.as_str())?;
    }
    if let Some(x) = &source.default_x {
        value::text(x)?;
    }
    let mut bytes = std::mem::size_of::<wire::TelemetrySchema>()
        .saturating_add(source.storage_schema.memory_size())
        .saturating_add(source.stream.as_str().len())
        .saturating_add(metadata_heap(&source.metadata)?)
        .saturating_add(
            source
                .template_id
                .as_ref()
                .map_or(0, |id| id.as_str().len()),
        )
        .saturating_add(source.owner.as_ref().map_or(0, |id| id.as_str().len()))
        .saturating_add(source.default_x.as_ref().map_or(0, String::len))
        .saturating_add(
            source
                .columns
                .len()
                .saturating_mul(std::mem::size_of::<wire::TelemetryColumn>()),
        );
    let mut budget = Budget::new();
    for column in &source.columns {
        budget.node(0)?;
        bytes = bytes
            .saturating_add(metadata_heap(&column.metadata)?)
            .saturating_add(format_heap(column.format.as_ref()));
        if let Some(unit) = &column.unit {
            bytes = bytes.saturating_add(
                unit_memory(unit).map_err(failure)? - std::mem::size_of::<wire::MeasurementUnit>(),
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
        harmos::Error::ResourceExhausted { .. } => {
            Status::resource_exhausted("canonical schema construction exceeds its bound")
        }
        _ => Status::internal("canonical schema projection failed"),
    }
}

fn bounded(bytes: usize) -> Result<usize, Status> {
    if bytes == usize::MAX {
        return Err(Status::resource_exhausted(
            "canonical schema construction overflows",
        ));
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

fn metadata_heap(source: &dokime_app::core::Metadata) -> Result<usize, Status> {
    Ok(canonical::metadata_memory(source)? - std::mem::size_of::<common::Metadata>())
}

fn metadata(source: &dokime_app::core::Metadata) -> Result<common::Metadata, Status> {
    canonical::metadata(source)
}

fn value_heap(source: &dokime_app::core::Value, budget: &mut Budget) -> Result<usize, Status> {
    Ok(canonical::memory(source, budget, 0)? - std::mem::size_of::<wire_value::Value>())
}

fn project_value(
    source: &dokime_app::core::Value,
    budget: &mut Budget,
    depth: usize,
) -> Result<wire_value::Value, Status> {
    canonical::encode(source, budget, depth)
}

fn format_heap(source: Option<&dokime_app::core::ValueFormat>) -> usize {
    match source {
        Some(dokime_app::core::ValueFormat::AbsoluteTime { pattern }) => pattern.len(),
        _ => 0,
    }
}

fn validation(value: &ValidationMode) -> wire::ValidationMode {
    match value {
        ValidationMode::Strict => wire::ValidationMode::Strict,
        ValidationMode::Prototype => wire::ValidationMode::Prototype,
    }
}

fn icon(value: &RecordIconKey) -> wire::RecordIcon {
    match value {
        RecordIconKey::Activity => wire::RecordIcon::Activity,
        RecordIconKey::AlertTriangle => wire::RecordIcon::AlertTriangle,
        RecordIconKey::Archive => wire::RecordIcon::Archive,
        RecordIconKey::Bell => wire::RecordIcon::Bell,
        RecordIconKey::CheckCircle => wire::RecordIcon::CheckCircle,
        RecordIconKey::CircleDashed => wire::RecordIcon::CircleDashed,
        RecordIconKey::ClipboardCheck => wire::RecordIcon::ClipboardCheck,
        RecordIconKey::Cpu => wire::RecordIcon::Cpu,
        RecordIconKey::FileText => wire::RecordIcon::FileText,
        RecordIconKey::Flag => wire::RecordIcon::Flag,
        RecordIconKey::Gauge => wire::RecordIcon::Gauge,
        RecordIconKey::HeartPulse => wire::RecordIcon::HeartPulse,
        RecordIconKey::HelpCircle => wire::RecordIcon::HelpCircle,
        RecordIconKey::MessageSquareText => wire::RecordIcon::MessageSquareText,
        RecordIconKey::Package => wire::RecordIcon::Package,
        RecordIconKey::Plug => wire::RecordIcon::Plug,
        RecordIconKey::RadioTower => wire::RecordIcon::RadioTower,
        RecordIconKey::RotateCcw => wire::RecordIcon::RotateCcw,
        RecordIconKey::ShieldAlert => wire::RecordIcon::ShieldAlert,
        RecordIconKey::Target => wire::RecordIcon::Target,
        RecordIconKey::TimerReset => wire::RecordIcon::TimerReset,
        RecordIconKey::Wrench => wire::RecordIcon::Wrench,
        RecordIconKey::Zap => wire::RecordIcon::Zap,
        RecordIconKey::Clock => wire::RecordIcon::Clock,
    }
}

fn tone(value: &RecordTone) -> wire::RecordTone {
    match value {
        RecordTone::Info => wire::RecordTone::Info,
        RecordTone::Success => wire::RecordTone::Success,
        RecordTone::Warning => wire::RecordTone::Warning,
        RecordTone::Danger => wire::RecordTone::Danger,
        RecordTone::Accent => wire::RecordTone::Accent,
        RecordTone::Muted => wire::RecordTone::Muted,
        RecordTone::Neutral => wire::RecordTone::Neutral,
    }
}

fn role(value: &TelemetryRole) -> wire::TelemetryRole {
    match value {
        TelemetryRole::Value => wire::TelemetryRole::Value,
        TelemetryRole::Counter => wire::TelemetryRole::Counter,
        TelemetryRole::State => wire::TelemetryRole::State,
        TelemetryRole::Distance => wire::TelemetryRole::Distance,
        TelemetryRole::Latency => wire::TelemetryRole::Latency,
        TelemetryRole::Throughput => wire::TelemetryRole::Throughput,
    }
}

/* === Tests === */
