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
use super::super::enums;
use super::{Budget, unit, unit_memory, value};
use dokime_app::{
    core::ValidationMode,
    records::{RecordContract, RecordIconKey, RecordSeverity, RecordTone},
    telemetry::{TelemetryContract, TelemetryRole},
};
use dokime_protocol::v1;
use value::projection;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn record(contract: &RecordContract) -> Result<v1::RecordSchema, Status> {
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
        Ok(v1::RecordField {
            metadata: Some(metadata(&field.metadata)?),
            value_type: enums::value_type(&field.value_type) as i32,
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
            format: value::optional_value_format(field.format.as_ref()),
        })
    }))?;
    Ok(v1::RecordSchema {
        record_type: contract.record_type.as_str().to_owned(),
        schema_version: contract.schema_version,
        metadata: Some(metadata(&contract.metadata)?),
        template_identifier: contract
            .template_id
            .as_ref()
            .map(|id| id.as_str().to_owned()),
        owner_suite_id: contract.owner.as_ref().map(|id| id.as_str().to_owned()),
        default_severity: match contract.default_severity {
            RecordSeverity::Debug => v1::Severity::Debug,
            RecordSeverity::Info => v1::Severity::Info,
            RecordSeverity::Warning => v1::Severity::Warn,
            RecordSeverity::Error => v1::Severity::Error,
        } as i32,
        fields,
        validation_mode: contract
            .validation_mode
            .as_ref()
            .map(|mode| validation(mode) as i32),
        allow_extra_fields: contract.allow_extra_fields,
        display: Some(v1::RecordDisplay {
            icon: icon(&contract.display.icon) as i32,
            tone: tone(&contract.display.tone) as i32,
        }),
    })
}

pub(crate) fn telemetry(contract: &TelemetryContract) -> Result<v1::TelemetrySchema, Status> {
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
        Ok(v1::TelemetryColumn {
            metadata: Some(metadata(&column.metadata)?),
            value_type: enums::value_type(&column.value_type) as i32,
            role: role(&column.role) as i32,
            unit: column.unit.as_ref().map(unit).transpose()?,
            display_unit: column.display_unit.clone(),
            required: column.required,
            nullable: column.nullable,
            default_value: column
                .default
                .as_ref()
                .map(|default| project_value(default, &mut budget, 0))
                .transpose()?,
            format: value::optional_value_format(column.format.as_ref()),
        })
    }))?;
    Ok(v1::TelemetrySchema {
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
        validation_mode: contract
            .validation_mode
            .as_ref()
            .map(|mode| validation(mode) as i32),
        foreign: contract.foreign,
    })
}

pub(crate) fn record_memory(source: &RecordContract) -> Result<usize, Status> {
    value::text(source.record_type.as_str())?;
    if let Some(id) = &source.template_id {
        value::text(id.as_str())?;
    }
    value::collection(source.fields.len())?;
    let mut bytes = std::mem::size_of::<v1::RecordSchema>()
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
                .saturating_mul(std::mem::size_of::<v1::RecordField>()),
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
                    .saturating_mul(std::mem::size_of::<v1::Value>()),
            )
            .saturating_add(format_heap(field.format.as_ref()));
        if let Some(unit) = &field.unit {
            bytes = bytes.saturating_add(
                unit_memory(unit).map_err(failure)? - std::mem::size_of::<v1::MeasurementUnit>(),
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
    value::collection(source.columns.len())?;
    value::text(source.stream.as_str())?;
    if let Some(id) = &source.template_id {
        value::text(id.as_str())?;
    }
    if let Some(x) = &source.default_x {
        value::text(x)?;
    }
    let mut bytes = std::mem::size_of::<v1::TelemetrySchema>()
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
                .saturating_mul(std::mem::size_of::<v1::TelemetryColumn>()),
        );
    let mut budget = Budget::new();
    for column in &source.columns {
        budget.node(0)?;
        bytes = bytes
            .saturating_add(metadata_heap(&column.metadata)?)
            .saturating_add(format_heap(column.format.as_ref()));
        if let Some(unit) = &column.unit {
            bytes = bytes.saturating_add(
                unit_memory(unit).map_err(failure)? - std::mem::size_of::<v1::MeasurementUnit>(),
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
    let mut target = projection::slots(source.len()).map_err(failure)?;
    for value in source {
        target.push(value?);
    }
    Ok(target)
}

fn metadata_heap(source: &dokime_app::core::Metadata) -> Result<usize, Status> {
    Ok(projection::metadata_count(source).map_err(failure)?.memory
        - std::mem::size_of::<v1::Metadata>())
}

fn metadata(source: &dokime_app::core::Metadata) -> Result<v1::Metadata, Status> {
    projection::metadata_count(source).map_err(failure)?;
    projection::metadata(source).map_err(failure)
}

fn value_heap(source: &dokime_app::core::Value, budget: &mut Budget) -> Result<usize, Status> {
    Ok(projection::count(source, budget, 0)
        .map_err(failure)?
        .memory
        - std::mem::size_of::<v1::Value>())
}

fn project_value(
    source: &dokime_app::core::Value,
    budget: &mut Budget,
    depth: usize,
) -> Result<v1::Value, Status> {
    projection::count(source, &mut Budget::new(), depth).map_err(failure)?;
    projection::project(source, budget, depth).map_err(failure)
}

fn format_heap(source: Option<&dokime_app::core::ValueFormat>) -> usize {
    match source {
        Some(dokime_app::core::ValueFormat::AbsoluteTime { pattern }) => pattern.len(),
        _ => 0,
    }
}

fn validation(value: &ValidationMode) -> v1::ValidationMode {
    match value {
        ValidationMode::Strict => v1::ValidationMode::Strict,
        ValidationMode::Prototype => v1::ValidationMode::Prototype,
    }
}

fn icon(value: &RecordIconKey) -> v1::RecordIcon {
    match value {
        RecordIconKey::Activity => v1::RecordIcon::Activity,
        RecordIconKey::AlertTriangle => v1::RecordIcon::AlertTriangle,
        RecordIconKey::Archive => v1::RecordIcon::Archive,
        RecordIconKey::Bell => v1::RecordIcon::Bell,
        RecordIconKey::CheckCircle => v1::RecordIcon::CheckCircle,
        RecordIconKey::CircleDashed => v1::RecordIcon::CircleDashed,
        RecordIconKey::ClipboardCheck => v1::RecordIcon::ClipboardCheck,
        RecordIconKey::Cpu => v1::RecordIcon::Cpu,
        RecordIconKey::FileText => v1::RecordIcon::FileText,
        RecordIconKey::Flag => v1::RecordIcon::Flag,
        RecordIconKey::Gauge => v1::RecordIcon::Gauge,
        RecordIconKey::HeartPulse => v1::RecordIcon::HeartPulse,
        RecordIconKey::HelpCircle => v1::RecordIcon::HelpCircle,
        RecordIconKey::MessageSquareText => v1::RecordIcon::MessageSquareText,
        RecordIconKey::Package => v1::RecordIcon::Package,
        RecordIconKey::Plug => v1::RecordIcon::Plug,
        RecordIconKey::RadioTower => v1::RecordIcon::RadioTower,
        RecordIconKey::RotateCcw => v1::RecordIcon::RotateCcw,
        RecordIconKey::ShieldAlert => v1::RecordIcon::ShieldAlert,
        RecordIconKey::Target => v1::RecordIcon::Target,
        RecordIconKey::TimerReset => v1::RecordIcon::TimerReset,
        RecordIconKey::Wrench => v1::RecordIcon::Wrench,
        RecordIconKey::Zap => v1::RecordIcon::Zap,
        RecordIconKey::Clock => v1::RecordIcon::Clock,
    }
}

fn tone(value: &RecordTone) -> v1::RecordTone {
    match value {
        RecordTone::Info => v1::RecordTone::Info,
        RecordTone::Success => v1::RecordTone::Success,
        RecordTone::Warning => v1::RecordTone::Warning,
        RecordTone::Danger => v1::RecordTone::Danger,
        RecordTone::Accent => v1::RecordTone::Accent,
        RecordTone::Muted => v1::RecordTone::Muted,
        RecordTone::Neutral => v1::RecordTone::Neutral,
    }
}

fn role(value: &TelemetryRole) -> v1::TelemetryRole {
    match value {
        TelemetryRole::Value => v1::TelemetryRole::Value,
        TelemetryRole::Counter => v1::TelemetryRole::Counter,
        TelemetryRole::State => v1::TelemetryRole::State,
        TelemetryRole::Distance => v1::TelemetryRole::Distance,
        TelemetryRole::Latency => v1::TelemetryRole::Latency,
        TelemetryRole::Throughput => v1::TelemetryRole::Throughput,
    }
}

/* === Tests === */
