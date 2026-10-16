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
use harmos::MemorySize;

// Local Imports
use crate::{Error, Result};
use dokime::core::{
    DurationMillis, Identifier, MeasurementUnit, Metadata, SuiteId, TemplateId, ValidationMode,
    Value, ValueFormat, ValueType,
};
use dokime::storage::objects::presentation as native;
use dokime::telemetry::contract::{TelemetryColumn, TelemetryContract, TelemetryRole};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
type Declarations = BTreeMap<Identifier, BTreeMap<u32, TelemetryContract>>;

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub(crate) struct Contracts {
    streams: Vec<Stream>,
}
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Stream {
    key: String,
    versions: Vec<Contract>,
}
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Contract {
    stream: String,
    schema_version: u32,
    metadata: Option<native::Metadata>,
    template_id: Option<String>,
    owner: Option<String>,
    columns: Vec<Column>,
    storage_schema: Option<harmos::schema::graph::Schema>,
    default_x: Option<String>,
    interval: Option<u64>,
    plot_window: Option<u64>,
    allow_extra_fields: bool,
    scalar: bool,
    validation_mode: Option<u32>,
    foreign: bool,
}
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Column {
    metadata: Option<native::Metadata>,
    value_type: u32,
    role: u32,
    unit: Option<native::MeasurementUnit>,
    display_unit: Option<String>,
    required: bool,
    nullable: bool,
    default: Option<native::Value>,
    format: Option<native::ValueFormat>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Contracts {
    pub(crate) fn encoding_memory(values: &Declarations) -> Result<usize> {
        let mut bytes = std::mem::size_of::<Self>();
        for (stream, versions) in values {
            if versions.is_empty() {
                return Err(invalid(
                    "captured telemetry stream has no producer declaration",
                ));
            }
            bytes = bytes
                .saturating_add(std::mem::size_of::<Stream>())
                .saturating_add(stream.as_str().len());
            for (version, contract) in versions {
                if stream != &contract.stream
                    || *version == 0
                    || *version != contract.schema_version
                    || contract.foreign
                {
                    return Err(invalid(
                        "captured telemetry declaration is not its producer contract",
                    ));
                }
                bytes = bytes.saturating_add(Contract::encoding_memory(contract)?);
            }
        }
        Ok(bytes)
    }
    pub(crate) fn projection_memory(&self) -> Result<usize> {
        let mut bytes = harmos::btree_map_memory::<Identifier, BTreeMap<u32, TelemetryContract>>(
            self.streams.len(),
        );
        let mut previous: Option<&str> = None;
        for stream in &self.streams {
            if previous.is_some_and(|key| key >= stream.key.as_str()) || stream.versions.is_empty()
            {
                return Err(invalid("captured telemetry stream keys are not canonical"));
            }
            previous = Some(&stream.key);
            bytes = bytes.saturating_add(stream.key.capacity());
            bytes = excess(
                bytes,
                harmos::btree_map_memory::<u32, TelemetryContract>(stream.versions.len()),
                std::mem::size_of::<BTreeMap<u32, TelemetryContract>>(),
            );
            let mut previous_version = 0;
            for contract in &stream.versions {
                if contract.stream != stream.key
                    || contract.schema_version <= previous_version
                    || contract.foreign
                {
                    return Err(invalid(
                        "captured telemetry producer versions are not canonical",
                    ));
                }
                previous_version = contract.schema_version;
                bytes = excess(
                    bytes,
                    contract.projection_memory()?,
                    std::mem::size_of::<TelemetryContract>(),
                );
            }
        }
        Ok(bytes)
    }
}
impl TryFrom<&Declarations> for Contracts {
    type Error = Error;
    fn try_from(values: &Declarations) -> Result<Self> {
        if Self::encoding_memory(values)? == usize::MAX {
            return Err(invalid("telemetry encoding memory overflows"));
        }
        let mut streams = slots(values.len())?;
        for (key, versions) in values {
            let mut contracts = slots(versions.len())?;
            for contract in versions.values() {
                contracts.push(Contract::try_from(contract)?);
            }
            streams.push(Stream {
                key: key.as_str().to_owned(),
                versions: contracts,
            });
        }
        Ok(Self { streams })
    }
}
impl TryFrom<Contracts> for Declarations {
    type Error = Error;
    fn try_from(value: Contracts) -> Result<Self> {
        if value.projection_memory()? == usize::MAX {
            return Err(invalid("telemetry projection memory overflows"));
        }
        let mut streams = BTreeMap::new();
        for stream in value.streams {
            let key = Identifier::new(stream.key).map_err(dokime::storage::Error::from)?;
            let mut versions = BTreeMap::new();
            for contract in stream.versions {
                let contract: TelemetryContract = contract.try_into()?;
                versions.insert(contract.schema_version, contract);
            }
            streams.insert(key, versions);
        }
        Ok(streams)
    }
}
impl Contract {
    fn encoding_memory(value: &TelemetryContract) -> Result<usize> {
        let mut bytes = std::mem::size_of::<Self>()
            .saturating_add(value.storage_schema.memory_size())
            .saturating_add(value.stream.as_str().len())
            .saturating_add(value.template_id.as_ref().map_or(0, |id| id.as_str().len()))
            .saturating_add(value.owner.as_ref().map_or(0, |id| id.as_str().len()))
            .saturating_add(value.default_x.as_ref().map_or(0, String::len));
        bytes = excess(
            bytes,
            native::Metadata::encoding_memory(&value.metadata)?,
            std::mem::size_of::<native::Metadata>(),
        );
        for column in &value.columns {
            bytes = bytes.saturating_add(Column::encoding_memory(column)?);
        }
        Ok(bytes)
    }
    fn projection_memory(&self) -> Result<usize> {
        if let Some(mode) = self.validation_mode {
            validation_mode(mode)?;
        }
        let metadata = self
            .metadata
            .as_ref()
            .ok_or_else(|| invalid("telemetry contract metadata is missing"))?;
        let mut bytes = std::mem::size_of::<TelemetryContract>()
            .saturating_add(self.storage_schema.memory_size())
            .saturating_add(self.stream.capacity())
            .saturating_add(self.template_id.as_ref().map_or(0, String::capacity))
            .saturating_add(self.owner.as_ref().map_or(0, String::capacity))
            .saturating_add(self.default_x.as_ref().map_or(0, String::capacity));
        bytes = excess(
            bytes,
            metadata.projection_memory()?,
            std::mem::size_of::<Metadata>(),
        );
        for column in &self.columns {
            bytes = bytes.saturating_add(column.projection_memory()?);
        }
        Ok(bytes)
    }
}
impl TryFrom<&TelemetryContract> for Contract {
    type Error = Error;
    fn try_from(value: &TelemetryContract) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(invalid("telemetry contract encoding memory overflows"));
        }
        let mut columns = slots(value.columns.len())?;
        for column in &value.columns {
            columns.push(Column::try_from(column)?);
        }
        Ok(Self {
            stream: value.stream.as_str().to_owned(),
            schema_version: value.schema_version,
            metadata: Some((&value.metadata).try_into()?),
            template_id: value.template_id.as_ref().map(|id| id.as_str().to_owned()),
            owner: value.owner.as_ref().map(|id| id.as_str().to_owned()),
            columns,
            storage_schema: value.storage_schema.clone(),
            default_x: value.default_x.clone(),
            interval: value.interval.map(|duration| duration.0),
            plot_window: value.plot_window.map(|duration| duration.0),
            allow_extra_fields: value.allow_extra_fields,
            scalar: value.scalar,
            validation_mode: value.validation_mode.as_ref().map(validation_mode_code),
            foreign: value.foreign,
        })
    }
}
impl TryFrom<Contract> for TelemetryContract {
    type Error = Error;
    fn try_from(value: Contract) -> Result<Self> {
        if value.projection_memory()? == usize::MAX {
            return Err(invalid("telemetry contract projection memory overflows"));
        }
        let mut columns = slots(value.columns.len())?;
        for column in value.columns {
            columns.push(column.try_into()?);
        }
        Ok(Self {
            stream: Identifier::new(value.stream).map_err(dokime::storage::Error::from)?,
            schema_version: value.schema_version,
            metadata: value
                .metadata
                .ok_or_else(|| invalid("telemetry contract metadata is missing"))?
                .try_into()?,
            template_id: value
                .template_id
                .map(TemplateId::from_uuid)
                .transpose()
                .map_err(dokime::storage::Error::from)?,
            owner: value
                .owner
                .map(SuiteId::from_uuid)
                .transpose()
                .map_err(dokime::storage::Error::from)?,
            columns,
            storage_schema: value.storage_schema,
            default_x: value.default_x,
            interval: value.interval.map(DurationMillis),
            plot_window: value.plot_window.map(DurationMillis),
            allow_extra_fields: value.allow_extra_fields,
            scalar: value.scalar,
            validation_mode: value.validation_mode.map(validation_mode).transpose()?,
            foreign: value.foreign,
        })
    }
}
impl Column {
    fn encoding_memory(value: &TelemetryColumn) -> Result<usize> {
        let mut bytes = std::mem::size_of::<Self>()
            .saturating_add(value.display_unit.as_ref().map_or(0, String::len));
        bytes = excess(
            bytes,
            native::Metadata::encoding_memory(&value.metadata)?,
            std::mem::size_of::<native::Metadata>(),
        );
        if let Some(unit) = &value.unit {
            bytes = excess(
                bytes,
                native::MeasurementUnit::encoding_memory(unit)?,
                std::mem::size_of::<native::MeasurementUnit>(),
            );
        }
        if let Some(default) = &value.default {
            bytes = excess(
                bytes,
                native::Value::encoding_memory(default)?,
                std::mem::size_of::<native::Value>(),
            );
        }
        if let Some(format) = &value.format {
            bytes = excess(
                bytes,
                native::ValueFormat::encoding_memory(format),
                std::mem::size_of::<native::ValueFormat>(),
            );
        }
        Ok(bytes)
    }
    fn projection_memory(&self) -> Result<usize> {
        value_type(self.value_type)?;
        role(self.role)?;
        let metadata = self
            .metadata
            .as_ref()
            .ok_or_else(|| invalid("telemetry column metadata is missing"))?;
        let mut bytes = std::mem::size_of::<TelemetryColumn>()
            .saturating_add(self.display_unit.as_ref().map_or(0, String::capacity));
        bytes = excess(
            bytes,
            metadata.projection_memory()?,
            std::mem::size_of::<Metadata>(),
        );
        if let Some(unit) = &self.unit {
            bytes = excess(
                bytes,
                unit.projection_memory()?,
                std::mem::size_of::<MeasurementUnit>(),
            );
        }
        if let Some(default) = &self.default {
            bytes = bytes.saturating_add(default.projection_memory()?);
        }
        if let Some(format) = &self.format {
            bytes = excess(
                bytes,
                format.projection_memory()?,
                std::mem::size_of::<ValueFormat>(),
            );
        }
        Ok(bytes)
    }
}
impl TryFrom<&TelemetryColumn> for Column {
    type Error = Error;
    fn try_from(value: &TelemetryColumn) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(invalid("telemetry column encoding memory overflows"));
        }
        Ok(Self {
            metadata: Some((&value.metadata).try_into()?),
            value_type: value_type_code(&value.value_type),
            role: role_code(&value.role),
            unit: value
                .unit
                .as_ref()
                .map(native::MeasurementUnit::try_from)
                .transpose()?,
            display_unit: value.display_unit.clone(),
            required: value.required,
            nullable: value.nullable,
            default: value
                .default
                .as_ref()
                .map(native::Value::try_from)
                .transpose()?,
            format: value
                .format
                .as_ref()
                .map(native::ValueFormat::try_from)
                .transpose()?,
        })
    }
}
impl TryFrom<Column> for TelemetryColumn {
    type Error = Error;
    fn try_from(value: Column) -> Result<Self> {
        if value.projection_memory()? == usize::MAX {
            return Err(invalid("telemetry column projection memory overflows"));
        }
        Ok(Self {
            metadata: value
                .metadata
                .ok_or_else(|| invalid("telemetry column metadata is missing"))?
                .try_into()?,
            value_type: value_type(value.value_type)?,
            role: role(value.role)?,
            unit: value.unit.map(MeasurementUnit::try_from).transpose()?,
            display_unit: value.display_unit,
            required: value.required,
            nullable: value.nullable,
            default: value.default.map(Value::try_from).transpose()?,
            format: value.format.map(ValueFormat::try_from).transpose()?,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn invalid(message: &str) -> Error {
    Error::InvalidReportInputs(message.into())
}
fn excess(bytes: usize, owned: usize, inline: usize) -> usize {
    if owned == usize::MAX {
        usize::MAX
    } else {
        bytes.saturating_add(owned.saturating_sub(inline))
    }
}
fn slots<T>(count: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| invalid("telemetry declaration allocation failed"))?;
    if values.capacity() > count {
        return Err(invalid("telemetry declaration slots exceed admission"));
    }
    Ok(values)
}
pub(super) fn value_type_code(value: &ValueType) -> u32 {
    match value {
        ValueType::Boolean => 1,
        ValueType::Integer => 2,
        ValueType::Number => 3,
        ValueType::Text => 4,
        ValueType::Datetime => 5,
        ValueType::File => 6,
        ValueType::Object => 7,
        ValueType::Array => 8,
    }
}
pub(super) fn value_type(value: u32) -> Result<ValueType> {
    match value {
        1 => Ok(ValueType::Boolean),
        2 => Ok(ValueType::Integer),
        3 => Ok(ValueType::Number),
        4 => Ok(ValueType::Text),
        5 => Ok(ValueType::Datetime),
        6 => Ok(ValueType::File),
        7 => Ok(ValueType::Object),
        8 => Ok(ValueType::Array),
        _ => Err(invalid("unknown captured value_type")),
    }
}
fn role_code(value: &TelemetryRole) -> u32 {
    match value {
        TelemetryRole::Value => 1,
        TelemetryRole::Counter => 2,
        TelemetryRole::State => 3,
        TelemetryRole::Distance => 4,
        TelemetryRole::Latency => 5,
        TelemetryRole::Throughput => 6,
    }
}
fn role(value: u32) -> Result<TelemetryRole> {
    match value {
        1 => Ok(TelemetryRole::Value),
        2 => Ok(TelemetryRole::Counter),
        3 => Ok(TelemetryRole::State),
        4 => Ok(TelemetryRole::Distance),
        5 => Ok(TelemetryRole::Latency),
        6 => Ok(TelemetryRole::Throughput),
        _ => Err(invalid("unknown telemetry role")),
    }
}
pub(super) fn validation_mode_code(value: &ValidationMode) -> u32 {
    match value {
        ValidationMode::Strict => 1,
        ValidationMode::Prototype => 2,
    }
}
pub(super) fn validation_mode(value: u32) -> Result<ValidationMode> {
    match value {
        1 => Ok(ValidationMode::Strict),
        2 => Ok(ValidationMode::Prototype),
        _ => Err(invalid("unknown captured validation_mode")),
    }
}

/* === Tests === */
