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
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use crate::model::identity::Identifier;
use crate::model::identity::Metadata;
use crate::model::identity::SuiteId;
use crate::model::identity::TemplateId;
use crate::model::validation::ValidationMode;
use crate::model::values::MeasurementUnit;
use crate::model::values::Value;
use crate::model::values::ValueFormat;
use crate::model::values::ValueType;

/* === Constants === */

const TIMESTAMP: &str = "timestamp";
const TIMESTAMP_LABEL: &str = "Timestamp";
const TIMESTAMP_DESCRIPTION: &str = "The instant this row was measured.";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetryContract {
    pub stream: Identifier,
    pub schema_version: u32,
    pub metadata: Metadata,
    pub template_id: Option<TemplateId>,
    pub owner: Option<SuiteId>,
    pub columns: Vec<TelemetryColumn>,
    pub storage_schema: Option<harmos_domain::schema::graph::Schema>,
    pub default_x: Option<String>,
    pub interval: Option<crate::model::time::DurationMillis>,
    pub allow_extra_fields: bool,
    pub scalar: bool,
    pub validation_mode: Option<ValidationMode>,
    pub foreign: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetryColumn {
    pub metadata: Metadata,
    pub value_type: ValueType,
    pub role: TelemetryRole,
    pub unit: Option<MeasurementUnit>,
    pub display_unit: Option<String>,
    pub required: bool,
    pub nullable: bool,
    pub default: Option<Value>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<ValueFormat>,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TelemetryRole {
    Value,
    Counter,
    State,
    Distance,
    Latency,
    Throughput,
}

/* === Traits === */

/* === Implementations === */

impl harmos_domain::MemorySize for TelemetryContract {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.storage_schema), std::mem::size_of_val(&self.storage_schema)),
            (harmos_domain::MemorySize::memory_size(&self.stream), std::mem::size_of_val(&self.stream)),
            (harmos_domain::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos_domain::MemorySize::memory_size(&self.template_id), std::mem::size_of_val(&self.template_id)),
            (harmos_domain::MemorySize::memory_size(&self.owner), std::mem::size_of_val(&self.owner)),
            (harmos_domain::MemorySize::memory_size(&self.columns), std::mem::size_of_val(&self.columns)),
            (harmos_domain::MemorySize::memory_size(&self.default_x), std::mem::size_of_val(&self.default_x)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for TelemetryColumn {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos_domain::MemorySize::memory_size(&self.unit), std::mem::size_of_val(&self.unit)),
            (harmos_domain::MemorySize::memory_size(&self.display_unit), std::mem::size_of_val(&self.display_unit)),
            (harmos_domain::MemorySize::memory_size(&self.default), std::mem::size_of_val(&self.default)),
            (harmos_domain::MemorySize::memory_size(&self.format), std::mem::size_of_val(&self.format)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl<'de> Deserialize<'de> for TelemetryRole {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        match raw.as_str() {
            "time" | "value" => Ok(Self::Value),
            "counter" => Ok(Self::Counter),
            "state" => Ok(Self::State),
            "distance" => Ok(Self::Distance),
            "latency" => Ok(Self::Latency),
            "throughput" => Ok(Self::Throughput),
            other => Err(serde::de::Error::unknown_variant(other, &["value", "counter", "state", "distance", "latency", "throughput"])),
        }
    }
}

impl TelemetryContract {
    pub fn declared(stream: impl Into<String>, label: impl Into<String>) -> Self {
        let stream = Identifier::declared(stream, "telemetry stream");

        Self {
            metadata: Metadata::new(stream.clone(), label, None),
            stream,
            schema_version: 1,
            template_id: None,
            owner: None,
            columns: Vec::new(),
            storage_schema: None,
            default_x: None,
            interval: None,
            allow_extra_fields: true,
            scalar: false,
            validation_mode: None,
            foreign: false,
        }
        .normalized()
    }

    pub fn with_storage_schema(mut self, schema: harmos_domain::schema::graph::Schema) -> Self {
        self.storage_schema = Some(schema);
        self
    }

    pub fn typed<T: harmos_domain::Message + harmos_domain::schema::Described>(self) -> Self {
        let graph =
            harmos_domain::schema::graph::Schema::from_declaration(T::DEFINITION, &harmos_domain::config::SchemaSettings::default())
                .expect("a declared telemetry row has a valid finite schema");
        self.with_storage_schema(graph)
    }

    pub fn described(mut self, description: impl Into<String>) -> Self {
        self.metadata.description = Some(description.into());
        self
    }

    pub fn for_template(mut self, template_id: TemplateId) -> Self {
        self.template_id = Some(template_id);
        self
    }

    pub fn column(mut self, column: TelemetryColumn) -> Self {
        self.columns.push(column);
        self
    }

    pub fn columns(mut self, columns: impl IntoIterator<Item = TelemetryColumn>) -> Self {
        self.columns.extend(columns);
        self
    }

    pub fn every(mut self, interval_ms: u64) -> Self {
        self.interval = Some(crate::model::time::DurationMillis(interval_ms));
        self
    }

    pub fn strict(mut self) -> Self {
        self.validation_mode = Some(ValidationMode::Strict);
        self.allow_extra_fields = false;
        self
    }

    pub fn foreign(mut self) -> Self {
        self.foreign = true;
        self
    }

    pub fn normalized(&self) -> Self {
        let missing_timestamp = !self.columns.iter().any(|column| column.metadata.identifier.as_str() == TIMESTAMP);
        let count = self.columns.len().checked_add(usize::from(missing_timestamp)).expect("normalized telemetry columns overflow");
        let mut columns = Vec::with_capacity(count);
        if missing_timestamp {
            columns.push(TelemetryColumn::new(
                Metadata::new(
                    Identifier::new(TIMESTAMP).expect("timestamp is a valid identifier"),
                    TIMESTAMP_LABEL,
                    Some(TIMESTAMP_DESCRIPTION.to_owned()),
                ),
                ValueType::Datetime,
                TelemetryRole::Value,
            ));
        }

        columns.extend(self.columns.iter().cloned());
        Self {
            stream: self.stream.clone(),
            schema_version: self.schema_version,
            metadata: self.metadata.clone(),
            template_id: self.template_id.clone(),
            owner: self.owner.clone(),
            columns,
            storage_schema: self.storage_schema.clone(),
            default_x: self.default_x.clone().or_else(|| Some(TIMESTAMP.to_owned())),
            interval: self.interval,
            allow_extra_fields: self.allow_extra_fields,
            scalar: self.scalar,
            validation_mode: self.validation_mode,
            foreign: self.foreign,
        }
    }

    pub fn stream_schema(&self) -> Self {
        let mut schema = self.normalized();
        schema.template_id = None;
        schema
    }

    pub fn column_map(&self) -> BTreeMap<&str, &TelemetryColumn> {
        self.columns.iter().map(|column| (column.metadata.identifier.as_str(), column)).collect()
    }
}

impl TelemetryColumn {
    pub fn declared(identifier: impl Into<String>, label: impl Into<String>, value_type: ValueType, role: TelemetryRole) -> Self {
        let identifier = Identifier::declared(identifier, "telemetry column");

        Self::new(Metadata::new(identifier, label, None), value_type, role)
    }

    pub fn new(metadata: Metadata, value_type: ValueType, role: TelemetryRole) -> Self {
        Self {
            metadata,
            value_type,
            role,
            unit: None,
            display_unit: None,
            required: true,
            nullable: false,
            default: None,
            format: None,
        }
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    pub fn nullable(mut self) -> Self {
        self.nullable = true;
        self.required = false;
        self
    }

    pub fn with_unit(mut self, unit: MeasurementUnit) -> Self {
        self.unit = Some(unit);
        self
    }

    pub fn with_format(mut self, format: ValueFormat) -> Self {
        self.format = Some(format);
        self
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_reserves_timestamp_and_preserves_authored_columns() {
        let mut contract = TelemetryContract::declared("fixture.normalization", "Fixture");
        contract.columns.clear();
        contract.default_x = None;
        contract.columns.push(TelemetryColumn::declared("reading", "Reading", ValueType::Number, TelemetryRole::Value));
        let authored = contract.columns.clone();
        let normalized = contract.normalized();
        assert_eq!(normalized.columns.len(), authored.len() + 1);
        assert_eq!(normalized.columns[0].metadata.identifier.as_str(), TIMESTAMP);
        assert_eq!(&normalized.columns[1..], authored.as_slice());
        assert_eq!(normalized.default_x.as_deref(), Some(TIMESTAMP));
        assert_eq!(contract.columns, authored);
        assert_eq!(normalized.normalized(), normalized);
    }
}
