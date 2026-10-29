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
use std::collections::BTreeMap;

// Third Party Imports

// Local Imports
use crate::error::TelemetryFailure;
use crate::telemetry::{
    TelemetrySample,
    contract::{TelemetryColumn, TelemetryContract},
};
use dokime_domain::model::validation::ValidationMode;
use dokime_domain::model::values::Value;
use dokime_domain::model::values::value_matches_type;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn validate_telemetry_sample(
    contract: Option<&TelemetryContract>, sample: &TelemetrySample, default_mode: ValidationMode,
) -> crate::Result<BTreeMap<String, Value>> {
    check_telemetry_sample(contract, sample, default_mode)?;
    Ok(resolved_values(contract, sample, default_mode))
}

pub fn validate_normalized_telemetry_sample(
    contract: &TelemetryContract, sample: &TelemetrySample, default_mode: ValidationMode,
) -> crate::Result<BTreeMap<String, Value>> {
    check_normalized_telemetry_sample(contract, sample, default_mode)?;
    Ok(resolved_values(Some(contract), sample, default_mode))
}

pub fn check_telemetry_sample(
    contract: Option<&TelemetryContract>, sample: &TelemetrySample, default_mode: ValidationMode,
) -> crate::Result<()> {
    let Some(contract) = contract else {
        if default_mode == ValidationMode::Prototype {
            return Ok(());
        }
        return Err(crate::Error::Telemetry(TelemetryFailure::UnknownStream {
            stream: sample.stream.as_str().to_owned(),
        }));
    };
    check_sample(contract, sample, default_mode, true)
}

pub fn check_normalized_telemetry_sample(
    contract: &TelemetryContract, sample: &TelemetrySample, default_mode: ValidationMode,
) -> crate::Result<()> {
    check_sample(contract, sample, default_mode, false)
}

/// The Borrowed Rule Index And Every Possible Direct Validation Diagnostic.
/// This Excludes Caller Formatting And Native Typed-Payload Validation Workspaces.
pub fn telemetry_sample_check_memory(contract: Option<&TelemetryContract>, sample: &TelemetrySample) -> usize {
    let count = contract.map_or(0, |contract| contract.columns.len().saturating_add(1));
    let mut bytes = harmos::btree_map_memory::<&str, Rule<'_>>(count);
    let mut longest = "timestamp".len();
    if let Some(contract) = contract {
        for column in &contract.columns {
            longest = longest.max(column.metadata.identifier.as_str().len());
        }
    }
    bytes = bytes.saturating_add(sample.stream.as_str().len()).saturating_add(longest);
    bytes = bytes.saturating_add(sample.values.len().saturating_mul(std::mem::size_of::<String>()));
    for name in sample.values.keys() {
        bytes = bytes.saturating_add(name.len());
    }
    bytes
}

/* === Private Functions === */

enum Rule<'a> {
    Column(&'a TelemetryColumn),
    Timestamp,
}

impl Rule<'_> {
    fn default(&self) -> Option<&Value> {
        match self {
            Self::Column(column) => column.default.as_ref(),
            Self::Timestamp => None,
        }
    }
    fn required(&self) -> bool {
        match self {
            Self::Column(column) => column.required,
            Self::Timestamp => true,
        }
    }
    fn nullable(&self) -> bool {
        match self {
            Self::Column(column) => column.nullable,
            Self::Timestamp => false,
        }
    }
    fn value_type(&self) -> &dokime_domain::model::values::ValueType {
        match self {
            Self::Column(column) => &column.value_type,
            Self::Timestamp => &dokime_domain::model::values::ValueType::Datetime,
        }
    }
}

fn check_sample(
    contract: &TelemetryContract, sample: &TelemetrySample, default_mode: ValidationMode, implicit_timestamp: bool,
) -> crate::Result<()> {
    let stream = sample.stream.as_str();
    if contract.stream != sample.stream {
        return Err(crate::Error::Telemetry(TelemetryFailure::UnknownStream { stream: stream.to_owned() }));
    }
    if contract.schema_version != sample.schema_version {
        return Err(crate::Error::Telemetry(TelemetryFailure::SchemaVersionMismatch {
            stream: stream.to_owned(),
            expected: contract.schema_version,
            actual: sample.schema_version,
        }));
    }
    if contract.validation_mode.unwrap_or(default_mode) == ValidationMode::Prototype {
        return Ok(());
    }

    // A Borrowed Index Preserves Lexical Diagnostic Order And Last-Declaration Wins.
    let mut declared = BTreeMap::new();
    for column in &contract.columns {
        declared.insert(column.metadata.identifier.as_str(), Rule::Column(column));
    }
    if implicit_timestamp {
        declared.entry("timestamp").or_insert(Rule::Timestamp);
    }
    for (name, rule) in &declared {
        let Some(value) = sample.values.get(*name) else {
            if rule.default().is_some() || !rule.required() {
                continue;
            }
            return Err(crate::Error::Telemetry(TelemetryFailure::MissingColumn {
                stream: stream.to_owned(),
                column: (*name).to_owned(),
            }));
        };
        if matches!(value, Value::Null) {
            if rule.nullable() || !rule.required() {
                continue;
            }
        } else if value_matches_type(value, rule.value_type()) {
            continue;
        }
        return Err(crate::Error::Telemetry(TelemetryFailure::InvalidColumn {
            stream: stream.to_owned(),
            column: (*name).to_owned(),
        }));
    }
    if !contract.allow_extra_fields {
        let count = sample.values.keys().filter(|name| !declared.contains_key(name.as_str())).count();
        if count != 0 {
            let mut extra = Vec::with_capacity(count);
            for name in sample.values.keys().filter(|name| !declared.contains_key(name.as_str())) {
                extra.push(name.to_owned());
            }
            return Err(crate::Error::Telemetry(TelemetryFailure::UnexpectedColumns {
                stream: stream.to_owned(),
                columns: extra,
            }));
        }
    }
    Ok(())
}

fn resolved_values(
    contract: Option<&TelemetryContract>, sample: &TelemetrySample, default_mode: ValidationMode,
) -> BTreeMap<String, Value> {
    let mut values = sample.values.clone();
    let Some(contract) = contract else { return values };
    if contract.validation_mode.unwrap_or(default_mode) == ValidationMode::Prototype {
        return values;
    }
    for (name, column) in contract.column_map() {
        if !values.contains_key(name)
            && let Some(default) = &column.default
        {
            values.insert(name.to_owned(), default.clone());
        }
    }
    values
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::contract::TelemetryRole;
    use dokime_domain::model::values::ValueType;

    #[test]
    fn borrowed_check_preserves_defaults_and_sorted_extra_diagnostics() {
        let mut contract = TelemetryContract::declared("fixture.validation", "Fixture").strict();
        let mut column = TelemetryColumn::declared("reading", "Reading", ValueType::Number, TelemetryRole::Value);
        column.default = Some(Value::Number(3.0));
        contract.columns.push(column);
        let at = dokime_domain::model::time::Timestamp::from_datetime(time::OffsetDateTime::UNIX_EPOCH);
        let mut sample = TelemetrySample::new(crate::model::identity::new_id(), contract.stream.clone(), 0, at, "fixture");
        sample.values.insert("timestamp".into(), Value::Datetime(time::OffsetDateTime::UNIX_EPOCH));
        check_normalized_telemetry_sample(&contract, &sample, ValidationMode::Strict).unwrap();
        assert!(!sample.values.contains_key("reading"));
        let resolved = validate_normalized_telemetry_sample(&contract, &sample, ValidationMode::Strict).unwrap();
        assert_eq!(resolved.get("reading"), Some(&Value::Number(3.0)));

        sample.values.insert("z".into(), Value::Bool(true));
        sample.values.insert("a".into(), Value::Bool(true));
        let error = check_normalized_telemetry_sample(&contract, &sample, ValidationMode::Strict).unwrap_err();
        let crate::Error::Telemetry(TelemetryFailure::UnexpectedColumns { columns, .. }) = error else { panic!("unexpected error kind") };
        assert_eq!(columns, ["a", "z"]);
    }
}
