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
use std::collections::{BTreeMap, BTreeSet};
// Third Party Imports

// Local Imports
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::validation::ValidationMode;
use dokime_domain::model::values::Value;
use dokime_domain::model::values::value_matches_type;
use crate::records::{
    Record, RecordEntry,
    contract::{RecordContract, RecordField},
    error::{RecordFailure, Result},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn validate_record_details(
    contract: Option<&RecordContract>,
    record: &RecordEntry,
    default_mode: ValidationMode,
) -> Result<BTreeMap<String, Value>> {
    validate_payload(
        contract,
        &record.record_type,
        record.schema_version,
        &record.details,
        default_mode,
    )
}

pub fn validate_record(
    contract: Option<&RecordContract>,
    record: &Record,
    default_mode: ValidationMode,
) -> Result<BTreeMap<String, Value>> {
    validate_payload(
        contract,
        &record.record_type,
        record.schema_version,
        &record.details,
        default_mode,
    )
}

/* === Private Functions === */
fn validate_payload(
    contract: Option<&RecordContract>,
    record_type: &Identifier,
    schema_version: u32,
    details: &BTreeMap<String, Value>,
    default_mode: ValidationMode,
) -> Result<BTreeMap<String, Value>> {
    let Some(contract) = contract else {
        if default_mode == ValidationMode::Prototype {
            return Ok(details.clone());
        }
        return Err(RecordFailure::UnknownRecordType {
            record_type: record_type.to_string(),
        }
        .into());
    };

    if contract.record_type != *record_type {
        return Err(RecordFailure::UnknownRecordType {
            record_type: record_type.to_string(),
        }
        .into());
    }

    if contract.schema_version != schema_version {
        return Err(RecordFailure::SchemaVersionMismatch {
            record_type: record_type.to_string(),
            expected: contract.schema_version,
            actual: schema_version,
        }
        .into());
    }

    if contract.validation_mode.unwrap_or(default_mode) == ValidationMode::Prototype {
        return Ok(details.clone());
    }

    validate_fields(
        &record_type.to_string(),
        details,
        &contract.fields,
        contract.allow_extra_fields,
    )
}

fn validate_fields(
    record_type: &str,
    payload: &BTreeMap<String, Value>,
    fields: &[RecordField],
    allow_extra_fields: bool,
) -> Result<BTreeMap<String, Value>> {
    let mut resolved = payload.clone();
    let declared: BTreeMap<&str, &RecordField> = fields
        .iter()
        .map(|field| (field.metadata.identifier.as_str(), field))
        .collect();

    for (name, field) in &declared {
        let Some(value) = resolved.get(*name) else {
            if let Some(default) = &field.default {
                resolved.insert((*name).to_owned(), default.clone());
                continue;
            }
            if field.required {
                return Err(RecordFailure::MissingField {
                    record_type: record_type.to_owned(),
                    field: (*name).to_owned(),
                }
                .into());
            }
            continue;
        };

        if matches!(value, Value::Null) {
            if field.nullable || !field.required {
                continue;
            }
            return Err(RecordFailure::InvalidField {
                record_type: record_type.to_owned(),
                field: (*name).to_owned(),
            }
            .into());
        }

        if !value_matches_type(value, &field.value_type) {
            return Err(RecordFailure::InvalidField {
                record_type: record_type.to_owned(),
                field: (*name).to_owned(),
            }
            .into());
        }

        if !field.choices.is_empty() && !field.choices.contains(value) {
            return Err(RecordFailure::InvalidChoice {
                record_type: record_type.to_owned(),
                field: (*name).to_owned(),
            }
            .into());
        }
    }

    if !allow_extra_fields {
        let declared_names: BTreeSet<&str> = declared.keys().copied().collect();
        let extra: Vec<String> = resolved
            .keys()
            .filter(|name| !declared_names.contains(name.as_str()))
            .cloned()
            .collect();

        if !extra.is_empty() {
            return Err(RecordFailure::UnexpectedFields {
                record_type: record_type.to_owned(),
                fields: extra,
            }
            .into());
        }
    }

    Ok(resolved)
}

/* === Tests === */
