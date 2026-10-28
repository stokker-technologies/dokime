/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;

use crate::storage::objects::presentation as native;
use crate::telemetry::contract::{TelemetryColumn, TelemetryContract, TelemetryRole};
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::SuiteId;
use dokime_domain::model::identity::TemplateId;
use dokime_domain::model::time::DurationMillis;
use dokime_domain::model::validation::ValidationMode;
use dokime_domain::model::values::MeasurementUnit;
use dokime_domain::model::values::Value;
use dokime_domain::model::values::ValueFormat;
use dokime_domain::model::values::ValueType;

type Declarations = BTreeMap<Identifier, BTreeMap<u32, TelemetryContract>>;

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

impl TryFrom<&Declarations> for Contracts {
    type Error = Error;
    fn try_from(values: &Declarations) -> Result<Self> {
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
        let mut streams = BTreeMap::new();
        for stream in value.streams {
            let key = Identifier::new(stream.key).map_err(crate::storage::Error::from)?;
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

impl TryFrom<&TelemetryContract> for Contract {
    type Error = Error;
    fn try_from(value: &TelemetryContract) -> Result<Self> {
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
        let mut columns = slots(value.columns.len())?;
        for column in value.columns {
            columns.push(column.try_into()?);
        }
        Ok(Self {
            stream: Identifier::new(value.stream).map_err(crate::storage::Error::from)?,
            schema_version: value.schema_version,
            metadata: value.metadata.ok_or_else(|| invalid("telemetry contract metadata is missing"))?.try_into()?,
            template_id: value.template_id.map(TemplateId::from_uuid).transpose().map_err(crate::storage::Error::from)?,
            owner: value.owner.map(SuiteId::from_uuid).transpose().map_err(crate::storage::Error::from)?,
            columns,
            storage_schema: value.storage_schema,
            default_x: value.default_x,
            interval: value.interval.map(DurationMillis),
            allow_extra_fields: value.allow_extra_fields,
            scalar: value.scalar,
            validation_mode: value.validation_mode.map(validation_mode).transpose()?,
            foreign: value.foreign,
        })
    }
}

impl TryFrom<&TelemetryColumn> for Column {
    type Error = Error;
    fn try_from(value: &TelemetryColumn) -> Result<Self> {
        Ok(Self {
            metadata: Some((&value.metadata).try_into()?),
            value_type: value_type_code(&value.value_type),
            role: role_code(&value.role),
            unit: value.unit.as_ref().map(native::MeasurementUnit::try_from).transpose()?,
            display_unit: value.display_unit.clone(),
            required: value.required,
            nullable: value.nullable,
            default: value.default.as_ref().map(native::Value::try_from).transpose()?,
            format: value.format.as_ref().map(native::ValueFormat::try_from).transpose()?,
        })
    }
}

impl TryFrom<Column> for TelemetryColumn {
    type Error = Error;
    fn try_from(value: Column) -> Result<Self> {
        Ok(Self {
            metadata: value.metadata.ok_or_else(|| invalid("telemetry column metadata is missing"))?.try_into()?,
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

fn invalid(message: &str) -> Error {
    Error::InvalidReportInputs(message.into())
}

fn slots<T>(count: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|_| invalid("telemetry declaration allocation failed"))?;
    Ok(values)
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
