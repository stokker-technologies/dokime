/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::model::Suite;
use harmos_protocol::host::PluginState;
use harmos_protocol::model::Declarations;
use prost::Message;
use prost_reflect::{DescriptorPool, FieldDescriptor, Kind};
use std::collections::BTreeMap;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Inventory {
    pub suites: Vec<(String, dokime_protocol::model::Suite)>,
    pub records: Vec<dokime_protocol::host::RecordSchema>,
    pub streams: Vec<dokime_protocol::host::TelemetrySchema>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Inventory {
    pub async fn read(context: &Context) -> Result<Self> {
        context.host.refresh_plugin_states().await?;
        let plugins = context.host.plugins().await?;
        let mut catalogs = Vec::new();
        for plugin in plugins.iter().filter(|plugin| plugin.state == PluginState::Ready as i32) {
            catalogs.push(
                plugin
                    .description
                    .as_ref()
                    .and_then(|description| description.declarations.clone())
                    .ok_or_else(|| Error::Invalid("Browse Declarations: Reconnect the Plugin with Its Declarations".into()))?,
            );
        }
        Self::from_catalogs(catalogs, context.host.suites()?)
    }

    pub fn captured(context: &Context, parent: &str) -> Result<Self> {
        let snapshot = context.host.store().metadata(parent)?;
        let catalog = Declarations::decode(snapshot.execution.catalog.as_slice())?;
        Self::from_catalogs(vec![catalog], Vec::new())
    }

    fn from_catalogs(catalogs: Vec<Declarations>, suites: Vec<(String, Suite)>) -> Result<Self> {
        let mut records = BTreeMap::new();
        let mut streams = BTreeMap::new();
        for catalog in catalogs {
            let types =
                catalog.types.clone().ok_or_else(|| Error::Invalid("Browse Declarations: Reconnect with Message Descriptors".into()))?;
            let pool = DescriptorPool::from_file_descriptor_set(types).map_err(|error| Error::Invalid(error.to_string()))?;
            for section in &catalog.sections {
                if section.r#type != "dokime.model.v1.RecordContracts" {
                    continue;
                }
                for record in dokime_protocol::model::RecordContracts::decode(section.payload.as_slice())?.records {
                    let id = record.metadata.as_ref().map(|metadata| metadata.identifier.clone()).unwrap_or_default();
                    let descriptor = pool
                        .get_message_by_name(&record.message)
                        .ok_or_else(|| Error::Invalid(format!("Browse Record {id}: Rebuild Its Message Descriptors")))?;
                    let fields = descriptor
                        .fields()
                        .map(|field| dokime_protocol::host::RecordField {
                            metadata: Some(metadata(&field)),
                            value_type: value_type(&field) as i32,
                            required: !field.supports_presence(),
                            nullable: field.supports_presence(),
                            ..Default::default()
                        })
                        .collect();
                    let schema = dokime_protocol::host::RecordSchema {
                        record_type: id.clone(),
                        message_name: record.message,
                        schema_version: 1,
                        metadata: record.metadata,
                        fields,
                        ..Default::default()
                    };
                    if records.insert(id, schema).is_some() {
                        return Err(Error::Invalid("Browse Records: Give Admitted Record Contracts Unique Identifiers".into()));
                    }
                }
            }
            for stream in &catalog.streams {
                let descriptor = pool
                    .get_message_by_name(&stream.row)
                    .ok_or_else(|| Error::Invalid(format!("Browse Stream {}: Rebuild Its Row Descriptors", stream.id)))?;
                let columns = descriptor
                    .fields()
                    .map(|field| dokime_protocol::host::TelemetryColumn {
                        metadata: Some(metadata(&field)),
                        value_type: value_type(&field) as i32,
                        required: !field.supports_presence(),
                        nullable: field.supports_presence(),
                        ..Default::default()
                    })
                    .collect();
                let schema = dokime_protocol::host::TelemetrySchema {
                    stream: stream.id.clone(),
                    message_name: stream.row.clone(),
                    schema_version: 1,
                    columns,
                    metadata: Some(dokime_protocol::model::Metadata {
                        identifier: stream.id.clone(),
                        label: stream.id.clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                };
                if streams.insert(stream.id.clone(), schema).is_some() {
                    return Err(Error::Invalid("Browse Streams: Give Admitted Streams Unique Identifiers".into()));
                }
            }
        }
        Ok(Self {
            suites,
            records: records.into_values().collect(),
            streams: streams.into_values().collect(),
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn metadata(field: &FieldDescriptor) -> dokime_protocol::model::Metadata {
    dokime_protocol::model::Metadata {
        identifier: field.name().into(),
        label: field.name().into(),
        ..Default::default()
    }
}
fn value_type(field: &FieldDescriptor) -> dokime_protocol::model::ValueType {
    if field.is_list() || field.is_map() {
        return dokime_protocol::model::ValueType::List;
    }
    match field.kind() {
        Kind::Bool => dokime_protocol::model::ValueType::Boolean,
        Kind::Int32 | Kind::Int64 | Kind::Sint32 | Kind::Sint64 | Kind::Sfixed32 | Kind::Sfixed64 | Kind::Enum(_) => {
            dokime_protocol::model::ValueType::Integer
        }
        Kind::Uint32 | Kind::Uint64 | Kind::Fixed32 | Kind::Fixed64 => dokime_protocol::model::ValueType::UnsignedInteger,
        Kind::Double | Kind::Float => dokime_protocol::model::ValueType::Number,
        Kind::String => dokime_protocol::model::ValueType::Text,
        Kind::Bytes => dokime_protocol::model::ValueType::Bytes,
        Kind::Message(message) if message.full_name() == "google.protobuf.Timestamp" => dokime_protocol::model::ValueType::Datetime,
        Kind::Message(_) => dokime_protocol::model::ValueType::Bytes,
    }
}

/* === Tests === */
