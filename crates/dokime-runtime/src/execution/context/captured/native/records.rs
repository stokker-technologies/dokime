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
use super::telemetry::{validation_mode, validation_mode_code, value_type, value_type_code};
use crate::records::{RecordContract, RecordDisplay, RecordField, RecordIconKey, RecordSeverity, RecordTone};
use crate::storage::objects::presentation as native;
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::SuiteId;
use dokime_domain::model::identity::TemplateId;
use dokime_domain::model::values::MeasurementUnit;
use dokime_domain::model::values::Value;
use dokime_domain::model::values::ValueFormat;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
type Declarations = BTreeMap<Identifier, BTreeMap<u32, RecordContract>>;

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub(crate) struct Contracts {
    types: Vec<Type>,
}
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Type {
    key: String,
    versions: Vec<Contract>,
}
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Contract {
    record_type: String,
    schema_version: u32,
    metadata: Option<native::Metadata>,
    template_id: Option<String>,
    owner: Option<String>,
    default_severity: u32,
    icon: u32,
    tone: u32,
    fields: Vec<Field>,
    validation_mode: Option<u32>,
    allow_extra_fields: bool,
}
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Field {
    metadata: Option<native::Metadata>,
    value_type: u32,
    required: bool,
    nullable: bool,
    unit: Option<native::MeasurementUnit>,
    choices: Vec<native::Value>,
    default: Option<native::Value>,
    format: Option<native::ValueFormat>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Contracts {}
impl TryFrom<&Declarations> for Contracts {
    type Error = Error;
    fn try_from(values: &Declarations) -> Result<Self> {
        let mut types = slots(values.len())?;
        for (key, versions) in values {
            let mut contracts = slots(versions.len())?;
            for contract in versions.values() {
                contracts.push(contract.try_into()?);
            }
            types.push(Type { key: text(key.as_str())?, versions: contracts });
        }
        Ok(Self { types })
    }
}
impl TryFrom<Contracts> for Declarations {
    type Error = Error;
    fn try_from(value: Contracts) -> Result<Self> {
        let mut types = BTreeMap::new();
        for kind in value.types {
            let key = Identifier::new(kind.key).map_err(crate::storage::Error::from)?;
            let mut versions = BTreeMap::new();
            for contract in kind.versions {
                let contract: RecordContract = contract.try_into()?;
                versions.insert(contract.schema_version, contract);
            }
            types.insert(key, versions);
        }
        Ok(types)
    }
}
impl Contract {}
impl TryFrom<&RecordContract> for Contract {
    type Error = Error;
    fn try_from(value: &RecordContract) -> Result<Self> {
        let mut fields = slots(value.fields.len())?;
        for field in &value.fields {
            fields.push(field.try_into()?);
        }
        Ok(Self {
            record_type: text(value.record_type.as_str())?,
            schema_version: value.schema_version,
            metadata: Some((&value.metadata).try_into()?),
            template_id: value.template_id.as_ref().map(|id| text(id.as_str())).transpose()?,
            owner: value.owner.as_ref().map(|id| text(id.as_str())).transpose()?,
            default_severity: severity_code(&value.default_severity),
            icon: icon_code(&value.display.icon),
            tone: tone_code(&value.display.tone),
            fields,
            validation_mode: value.validation_mode.as_ref().map(validation_mode_code),
            allow_extra_fields: value.allow_extra_fields,
        })
    }
}
impl TryFrom<Contract> for RecordContract {
    type Error = Error;
    fn try_from(value: Contract) -> Result<Self> {
        let mut fields = slots(value.fields.len())?;
        for field in value.fields {
            fields.push(field.try_into()?);
        }
        Ok(Self {
            record_type: Identifier::new(value.record_type).map_err(crate::storage::Error::from)?,
            schema_version: value.schema_version,
            metadata: value.metadata.ok_or_else(|| invalid("record declaration metadata is missing"))?.try_into()?,
            template_id: value.template_id.map(TemplateId::from_uuid).transpose().map_err(crate::storage::Error::from)?,
            owner: value.owner.map(SuiteId::from_uuid).transpose().map_err(crate::storage::Error::from)?,
            default_severity: severity(value.default_severity)?,
            display: RecordDisplay { icon: icon(value.icon)?, tone: tone(value.tone)? },
            fields,
            validation_mode: value.validation_mode.map(validation_mode).transpose()?,
            allow_extra_fields: value.allow_extra_fields,
        })
    }
}
impl Field {}
impl TryFrom<&RecordField> for Field {
    type Error = Error;
    fn try_from(value: &RecordField) -> Result<Self> {
        let mut choices = slots(value.choices.len())?;
        for choice in &value.choices {
            choices.push(choice.try_into()?);
        }
        Ok(Self {
            metadata: Some((&value.metadata).try_into()?),
            value_type: value_type_code(&value.value_type),
            required: value.required,
            nullable: value.nullable,
            unit: value.unit.as_ref().map(native::MeasurementUnit::try_from).transpose()?,
            choices,
            default: value.default.as_ref().map(native::Value::try_from).transpose()?,
            format: value.format.as_ref().map(native::ValueFormat::try_from).transpose()?,
        })
    }
}
impl TryFrom<Field> for RecordField {
    type Error = Error;
    fn try_from(value: Field) -> Result<Self> {
        let mut choices = slots(value.choices.len())?;
        for choice in value.choices {
            choices.push(choice.try_into()?);
        }
        Ok(Self {
            metadata: value.metadata.ok_or_else(|| invalid("record field metadata is missing"))?.try_into()?,
            value_type: value_type(value.value_type)?,
            required: value.required,
            nullable: value.nullable,
            unit: value.unit.map(MeasurementUnit::try_from).transpose()?,
            choices,
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
fn slots<T>(count: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|_| invalid("record declaration allocation failed"))?;
    if values.capacity() > count {
        return Err(invalid("record declaration slots exceed admission"));
    }
    Ok(values)
}
fn text(source: &str) -> Result<String> {
    let mut value = String::new();
    value.try_reserve_exact(source.len()).map_err(|_| invalid("record declaration text allocation failed"))?;
    if value.capacity() > source.len() {
        return Err(invalid("record declaration text exceeds admission"));
    }
    value.push_str(source);
    Ok(value)
}
fn severity_code(value: &RecordSeverity) -> u32 {
    match value {
        RecordSeverity::Debug => 1,
        RecordSeverity::Info => 2,
        RecordSeverity::Warning => 3,
        RecordSeverity::Error => 4,
    }
}
fn severity(value: u32) -> Result<RecordSeverity> {
    match value {
        1 => Ok(RecordSeverity::Debug),
        2 => Ok(RecordSeverity::Info),
        3 => Ok(RecordSeverity::Warning),
        4 => Ok(RecordSeverity::Error),
        _ => Err(invalid("unknown captured record severity")),
    }
}
fn tone_code(value: &RecordTone) -> u32 {
    match value {
        RecordTone::Info => 1,
        RecordTone::Success => 2,
        RecordTone::Warning => 3,
        RecordTone::Danger => 4,
        RecordTone::Accent => 5,
        RecordTone::Muted => 6,
        RecordTone::Neutral => 7,
    }
}
fn tone(value: u32) -> Result<RecordTone> {
    match value {
        1 => Ok(RecordTone::Info),
        2 => Ok(RecordTone::Success),
        3 => Ok(RecordTone::Warning),
        4 => Ok(RecordTone::Danger),
        5 => Ok(RecordTone::Accent),
        6 => Ok(RecordTone::Muted),
        7 => Ok(RecordTone::Neutral),
        _ => Err(invalid("unknown captured record tone")),
    }
}
fn icon_code(value: &RecordIconKey) -> u32 {
    match value {
        RecordIconKey::Activity => 1,
        RecordIconKey::AlertTriangle => 2,
        RecordIconKey::Archive => 3,
        RecordIconKey::Bell => 4,
        RecordIconKey::CheckCircle => 5,
        RecordIconKey::CircleDashed => 6,
        RecordIconKey::ClipboardCheck => 7,
        RecordIconKey::Cpu => 8,
        RecordIconKey::FileText => 9,
        RecordIconKey::Flag => 10,
        RecordIconKey::Gauge => 11,
        RecordIconKey::HeartPulse => 12,
        RecordIconKey::HelpCircle => 13,
        RecordIconKey::MessageSquareText => 14,
        RecordIconKey::Package => 15,
        RecordIconKey::Plug => 16,
        RecordIconKey::RadioTower => 17,
        RecordIconKey::RotateCcw => 18,
        RecordIconKey::ShieldAlert => 19,
        RecordIconKey::Target => 20,
        RecordIconKey::TimerReset => 21,
        RecordIconKey::Wrench => 22,
        RecordIconKey::Zap => 23,
        RecordIconKey::Clock => 24,
    }
}
fn icon(value: u32) -> Result<RecordIconKey> {
    match value {
        1 => Ok(RecordIconKey::Activity),
        2 => Ok(RecordIconKey::AlertTriangle),
        3 => Ok(RecordIconKey::Archive),
        4 => Ok(RecordIconKey::Bell),
        5 => Ok(RecordIconKey::CheckCircle),
        6 => Ok(RecordIconKey::CircleDashed),
        7 => Ok(RecordIconKey::ClipboardCheck),
        8 => Ok(RecordIconKey::Cpu),
        9 => Ok(RecordIconKey::FileText),
        10 => Ok(RecordIconKey::Flag),
        11 => Ok(RecordIconKey::Gauge),
        12 => Ok(RecordIconKey::HeartPulse),
        13 => Ok(RecordIconKey::HelpCircle),
        14 => Ok(RecordIconKey::MessageSquareText),
        15 => Ok(RecordIconKey::Package),
        16 => Ok(RecordIconKey::Plug),
        17 => Ok(RecordIconKey::RadioTower),
        18 => Ok(RecordIconKey::RotateCcw),
        19 => Ok(RecordIconKey::ShieldAlert),
        20 => Ok(RecordIconKey::Target),
        21 => Ok(RecordIconKey::TimerReset),
        22 => Ok(RecordIconKey::Wrench),
        23 => Ok(RecordIconKey::Zap),
        24 => Ok(RecordIconKey::Clock),
        _ => Err(invalid("unknown captured record icon")),
    }
}

/* === Tests === */
