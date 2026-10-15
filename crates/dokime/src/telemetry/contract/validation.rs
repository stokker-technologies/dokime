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
use crate::core::{ValidationMode, Value, value_matches_type};
use crate::error::TelemetryFailure;
use crate::telemetry::{
    TelemetrySample,
    contract::{TelemetryColumn, TelemetryContract},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn validate_telemetry_sample(
    contract: Option<&TelemetryContract>,
    sample: &TelemetrySample,
    default_mode: ValidationMode,
) -> crate::Result<BTreeMap<String, Value>> {
    let Some(contract) = contract else {
        if default_mode == ValidationMode::Prototype {
            return Ok(sample.values.clone());
        }
        return Err(crate::Error::Telemetry(TelemetryFailure::UnknownStream {
            stream: sample.stream.to_string(),
        }));
    };

    validate_normalized_telemetry_sample(&contract.normalized(), sample, default_mode)
}

// Validate against a contract already passed through `normalized`, so a batch
// prepares its contract once instead of once per row.
pub fn validate_normalized_telemetry_sample(
    contract: &TelemetryContract,
    sample: &TelemetrySample,
    default_mode: ValidationMode,
) -> crate::Result<BTreeMap<String, Value>> {
    if contract.stream != sample.stream {
        return Err(crate::Error::Telemetry(TelemetryFailure::UnknownStream {
            stream: sample.stream.to_string(),
        }));
    }

    if contract.schema_version != sample.schema_version {
        return Err(crate::Error::Telemetry(
            TelemetryFailure::SchemaVersionMismatch {
                stream: sample.stream.to_string(),
                expected: contract.schema_version,
                actual: sample.schema_version,
            },
        ));
    }

    if contract.validation_mode.unwrap_or(default_mode) == ValidationMode::Prototype {
        return Ok(sample.values.clone());
    }

    validate_columns(
        &sample.stream.to_string(),
        &sample.values,
        &contract.columns,
        contract.allow_extra_fields,
    )
}

/* === Private Functions === */
fn validate_columns(
    stream: &str,
    payload: &BTreeMap<String, Value>,
    columns: &[TelemetryColumn],
    allow_extra_fields: bool,
) -> crate::Result<BTreeMap<String, Value>> {
    let mut resolved = payload.clone();
    let declared: BTreeMap<&str, &TelemetryColumn> = columns
        .iter()
        .map(|column| (column.metadata.identifier.as_str(), column))
        .collect();

    for (name, column) in &declared {
        let Some(value) = resolved.get(*name) else {
            if let Some(default) = &column.default {
                resolved.insert((*name).to_owned(), default.clone());
                continue;
            }
            if column.required {
                return Err(crate::Error::Telemetry(TelemetryFailure::MissingColumn {
                    stream: stream.to_owned(),
                    column: (*name).to_owned(),
                }));
            }
            continue;
        };

        if matches!(value, Value::Null) {
            if column.nullable || !column.required {
                continue;
            }
            return Err(crate::Error::Telemetry(TelemetryFailure::InvalidColumn {
                stream: stream.to_owned(),
                column: (*name).to_owned(),
            }));
        }

        if !value_matches_type(value, &column.value_type) {
            return Err(crate::Error::Telemetry(TelemetryFailure::InvalidColumn {
                stream: stream.to_owned(),
                column: (*name).to_owned(),
            }));
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
            return Err(crate::Error::Telemetry(
                TelemetryFailure::UnexpectedColumns {
                    stream: stream.to_owned(),
                    columns: extra,
                },
            ));
        }
    }

    Ok(resolved)
}

/* === Tests === */
