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
use serde::{Serialize, Serializer, ser::SerializeMap};

// Local Imports
use super::ReportValue;
use crate::telemetry::{TelemetryContract, TelemetrySample};
use dokime_domain::model::values::{ValueFormat, ValueType};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct SampleValues<'a>(pub(super) &'a TelemetrySample);

#[derive(Serialize)]
pub(super) struct Spec<'a> {
    stream: &'a str,
    label: &'a str,
    columns: Vec<Column<'a>>,
    default_x: Option<&'a str>,
    allow_extra_fields: bool,
}

#[derive(Serialize)]
struct Column<'a> {
    name: &'a str,
    label: &'a str,
    value_type: &'static str,
    role: &'a crate::telemetry::TelemetryRole,
    unit: Option<&'a str>,
    display_unit: Option<&'a str>,
    required: bool,
    format: Option<&'a ValueFormat>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<'a> Spec<'a> {
    pub(super) fn new(contract: &'a TelemetryContract) -> Self {
        Self {
            stream: contract.stream.as_str(),
            label: &contract.metadata.label,
            columns: contract
                .columns
                .iter()
                .map(|column| Column {
                    name: column.metadata.identifier.as_str(),
                    label: &column.metadata.label,
                    value_type: match column.value_type {
                        ValueType::Boolean => "boolean",
                        ValueType::Integer => "integer",
                        ValueType::Number => "number",
                        ValueType::Datetime => "datetime",
                        _ => "string",
                    },
                    role: &column.role,
                    unit: column.unit.as_ref().map(|unit| unit.symbol.as_str()),
                    display_unit: column.display_unit.as_deref(),
                    required: column.required,
                    format: column.format.as_ref(),
                })
                .collect(),
            default_x: contract.default_x.as_deref(),
            allow_extra_fields: contract.allow_extra_fields,
        }
    }
}

impl Serialize for SampleValues<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let has_timestamp = self.0.values.contains_key("timestamp");
        let length = self.0.values.len() + usize::from(!has_timestamp);
        let mut fields = serializer.serialize_map(Some(length))?;
        if !has_timestamp {
            fields.serialize_entry("timestamp", &self.0.timestamp)?;
        }
        for (key, value) in &self.0.values {
            fields.serialize_entry(key, &ReportValue(value))?;
        }
        fields.end()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
