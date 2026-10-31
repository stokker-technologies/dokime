/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Captured Query Fields

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};

// Third Party Imports

// Local Imports
use super::{Envelope, FieldRef, SelectionRequest};
use crate::config::QueryConfig;
use dokime_domain::model::{
    query::{
        error::{Failure, Reason},
        schema::{Field, FieldType, Schema},
    },
    records::RecordContract,
    telemetry::TelemetryContract,
    values::{MeasurementUnit, ValueType},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Declared {
    field: Field,
    unit: Option<MeasurementUnit>,
}

struct Captured {
    version: u32,
    fields: Vec<Declared>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn records(
    contracts: &[RecordContract], id: &str, request: &SelectionRequest<'_>, config: &QueryConfig,
) -> Result<Schema, Box<Failure>> {
    let selected = select(contracts.iter().filter(|c| c.record_type.as_str() == id).map(|c| c.schema_version), id, request, config)?;
    let mut versions = Vec::new();
    let mut schema_bytes = 0_usize;
    for contract in contracts.iter().filter(|c| c.record_type.as_str() == id && selected.contains(&c.schema_version)) {
        config.bound(contract.fields.len(), config.limits().fields, "fields", request.operation)?;
        config.bound(versions.len() + 1, config.limits().versions, "versions", request.operation)?;
        schema_bytes = schema_bytes.saturating_add(harmos::MemorySize::memory_size(contract));
        config.bound(schema_bytes, config.limits().schema_bytes as usize, "schema_bytes", request.operation)?;
        let fields = contract
            .fields
            .iter()
            .map(|field| {
                declared(field.metadata.identifier.as_str(), &field.value_type, !field.required || field.nullable, field.unit.as_ref())
            })
            .collect();
        versions.push(Captured { version: contract.schema_version, fields });
    }
    merge(versions, id, request, config)
}

pub(super) fn samples(
    contracts: &[TelemetryContract], id: &str, request: &SelectionRequest<'_>, config: &QueryConfig,
) -> Result<Schema, Box<Failure>> {
    let selected = select(contracts.iter().filter(|c| c.stream.as_str() == id).map(|c| c.schema_version), id, request, config)?;
    let mut versions = Vec::new();
    let mut schema_bytes = 0_usize;
    for contract in contracts.iter().filter(|c| c.stream.as_str() == id && selected.contains(&c.schema_version)) {
        config.bound(contract.columns.len(), config.limits().fields, "fields", request.operation)?;
        config.bound(versions.len() + 1, config.limits().versions, "versions", request.operation)?;
        schema_bytes = schema_bytes.saturating_add(harmos::MemorySize::memory_size(contract));
        config.bound(schema_bytes, config.limits().schema_bytes as usize, "schema_bytes", request.operation)?;
        let fields = contract
            .columns
            .iter()
            .map(|field| {
                declared(field.metadata.identifier.as_str(), &field.value_type, !field.required || field.nullable, field.unit.as_ref())
            })
            .collect();
        versions.push(Captured { version: contract.schema_version, fields });
    }
    merge(versions, id, request, config)
}

pub(super) fn resolve(schema: &Schema, id: &str, records: bool, operation: &str) -> Result<(FieldRef, FieldType), Box<Failure>> {
    let envelope = match id {
        "@id" => Some((Envelope::Id, FieldType::Text)),
        "@timestamp" => Some((Envelope::Timestamp, FieldType::Datetime)),
        "@received_at" => Some((Envelope::ReceivedAt, FieldType::Datetime)),
        "@source" => Some((Envelope::Source, FieldType::Text)),
        "@schema_version" => Some((Envelope::SchemaVersion, FieldType::Uint64)),
        "@severity" if records => Some((Envelope::Severity, FieldType::Severity)),
        _ => None,
    };
    if let Some((field, kind)) = envelope {
        return Ok((FieldRef::Envelope(field), kind));
    }
    if let Some(field) = schema.fields.iter().find(|field| field.id == id) {
        return Ok((FieldRef::Declared(field.id.clone()), field.value_type));
    }
    let mut error = Failure::new(Reason::UnknownField, operation, "Field is not declared in the captured contract; use a literal field ID");
    error.field = Some(id.to_owned());
    Err(error)
}

/* === Private Functions === */

fn select(
    available: impl Iterator<Item = u32>, id: &str, request: &SelectionRequest<'_>, config: &QueryConfig,
) -> Result<BTreeSet<u32>, Box<Failure>> {
    config.bound(request.versions.len(), config.limits().versions, "versions", request.operation)?;
    if id.is_empty() || request.versions.contains(&0) || request.versions.iter().collect::<BTreeSet<_>>().len() != request.versions.len() {
        return Err(Failure::new(Reason::InvalidSelection, request.operation, "Use a concrete contract ID and unique positive versions"));
    }
    let mut found = false;
    let mut selected = BTreeSet::new();
    let mut advertised = BTreeSet::new();
    for version in available {
        found = true;
        if advertised.len() < config.limits().versions {
            advertised.insert(version);
        }
        if request.versions.is_empty() || request.versions.contains(&version) {
            selected.insert(version);
            config.bound(selected.len(), config.limits().versions, "versions", request.operation)?;
        }
    }
    if !found {
        return Err(Failure::new(Reason::ContractNotFound, request.operation, "Contract is not captured at this snapshot"));
    }
    if selected.is_empty() || request.versions.iter().any(|version| !selected.contains(version)) {
        let mut error = Failure::new(
            Reason::IncompatibleSchema,
            request.operation,
            "Requested versions are unavailable; narrow to an available captured version",
        );
        error.available_versions = advertised.into_iter().collect();
        return Err(error);
    }
    Ok(selected)
}

fn declared(id: &str, kind: &ValueType, nullable: bool, unit: Option<&MeasurementUnit>) -> Declared {
    let value_type = match kind {
        ValueType::Boolean => FieldType::Boolean,
        ValueType::Integer => FieldType::Integer,
        ValueType::Number => FieldType::Float64,
        ValueType::Text | ValueType::File => FieldType::Text,
        ValueType::Datetime => FieldType::Datetime,
        ValueType::Object => FieldType::Object,
        ValueType::Array => FieldType::Array,
    };
    Declared {
        field: Field {
            id: id.to_owned(),
            value_type,
            nullable,
            unit: unit.map(|unit| unit.symbol.clone()),
        },
        unit: unit.cloned(),
    }
}

fn merge(mut versions: Vec<Captured>, id: &str, request: &SelectionRequest<'_>, config: &QueryConfig) -> Result<Schema, Box<Failure>> {
    versions.sort_by_key(|version| version.version);
    let version_count = versions.len();
    let available: Vec<_> = versions.iter().map(|version| version.version).collect();
    let mut merged: BTreeMap<String, (Declared, usize)> = BTreeMap::new();
    let mut version_ids = BTreeSet::new();
    let mut field_order = Vec::new();
    for version in versions {
        if version.version == 0 || !version_ids.insert(version.version) {
            return Err(Failure::new(Reason::Corruption, request.operation, "Captured schema version is invalid or duplicated"));
        }
        let mut fields = BTreeSet::new();
        for field in version.fields {
            if field.field.id.is_empty() || field.field.id.starts_with('@') || !fields.insert(field.field.id.clone()) {
                return Err(Failure::new(Reason::Corruption, request.operation, "Captured schema field is invalid or duplicated"));
            }
            if let Some((previous, count)) = merged.get_mut(&field.field.id) {
                let compatible_unit = match (&previous.unit, &field.unit) {
                    (None, None) => true,
                    (Some(a), Some(b)) => a.metadata.identifier == b.metadata.identifier && a.family == b.family && a.symbol == b.symbol,
                    _ => false,
                };
                if previous.field.value_type != field.field.value_type || !compatible_unit {
                    let mut error = Failure::new(
                        Reason::IncompatibleSchema,
                        request.operation,
                        "Captured versions have incompatible types or units; select one version explicitly",
                    );
                    error.field = Some(field.field.id.clone());
                    error.available_versions = available;
                    return Err(error);
                }
                previous.field.nullable |= field.field.nullable;
                *count += 1;
            } else {
                config.bound(merged.len() + 1, config.limits().fields, "fields", request.operation)?;
                field_order.push(field.field.id.clone());
                merged.insert(field.field.id.clone(), (field, 1));
            }
        }
    }
    let fields = field_order
        .into_iter()
        .map(|id| {
            let (mut field, count) = merged.remove(&id).expect("captured field order");
            field.field.nullable |= count < version_count;
            field.field
        })
        .collect();
    let schema = Schema {
        identity: id.to_owned(),
        versions: version_ids.into_iter().collect(),
        fields,
    };
    config.bound(harmos::MemorySize::memory_size(&schema), config.limits().schema_bytes as usize, "schema_bytes", request.operation)?;
    Ok(schema)
}

/* === Tests === */
