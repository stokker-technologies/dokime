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
use crate::{Error, Result, client::Context};
use dokime_protocol::client::PluginState;
use dokime_protocol::v1;
use harmos_protocol::v1::Catalog;
use prost::Message;
use prost_reflect::{DescriptorPool, FieldDescriptor, Kind};
use std::collections::BTreeMap;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Inventory {
    pub suites: Vec<(String, v1::Suite)>,
    pub records: Vec<v1::RecordSchema>,
    pub streams: Vec<v1::TelemetrySchema>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Inventory {
    pub async fn read(context: &Context) -> Result<Self> {
        context.host.refresh_plugin_states().await?;
        let plugins = context.host.plugins()?;
        let mut catalogs = Vec::new();
        for plugin in plugins.iter().filter(|plugin| plugin.state == PluginState::Ready as i32) {
            catalogs.push(
                plugin
                    .description
                    .as_ref()
                    .and_then(|description| description.catalog.clone())
                    .ok_or_else(|| Error::Invalid("Browse Catalog: Reconnect the Plugin with Its Catalog".into()))?,
            );
        }
        Self::from_catalogs(catalogs, context.host.suites()?)
    }

    pub fn captured(context: &Context, parent: &str) -> Result<Self> {
        let snapshot = context.host.store().metadata(parent)?;
        let catalog = Catalog::decode(snapshot.execution.catalog.as_slice())?;
        Self::from_catalogs(vec![catalog], Vec::new())
    }

    fn from_catalogs(catalogs: Vec<Catalog>, suites: Vec<(String, v1::Suite)>) -> Result<Self> {
        let mut records = BTreeMap::new();
        let mut streams = BTreeMap::new();
        for catalog in catalogs {
            let types = catalog.types.clone().ok_or_else(|| Error::Invalid("Browse Catalog: Reconnect with Message Descriptors".into()))?;
            let pool = DescriptorPool::from_file_descriptor_set(types).map_err(|error| Error::Invalid(error.to_string()))?;
            for section in &catalog.sections {
                if section.r#type != "dokime.v1.RecordContracts" {
                    continue;
                }
                for record in v1::RecordContracts::decode(section.payload.as_slice())?.records {
                    let id = record.metadata.as_ref().map(|metadata| metadata.identifier.clone()).unwrap_or_default();
                    let descriptor = pool
                        .get_message_by_name(&record.message)
                        .ok_or_else(|| Error::Invalid(format!("Browse Record {id}: Rebuild Its Message Descriptors")))?;
                    let fields = descriptor
                        .fields()
                        .map(|field| v1::RecordField {
                            metadata: Some(metadata(&field)),
                            value_type: value_type(&field) as i32,
                            required: !field.supports_presence(),
                            nullable: field.supports_presence(),
                            ..Default::default()
                        })
                        .collect();
                    let schema = v1::RecordSchema {
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
                    .map(|field| v1::TelemetryColumn {
                        metadata: Some(metadata(&field)),
                        value_type: value_type(&field) as i32,
                        required: !field.supports_presence(),
                        nullable: field.supports_presence(),
                        ..Default::default()
                    })
                    .collect();
                let schema = v1::TelemetrySchema {
                    stream: stream.id.clone(),
                    message_name: stream.row.clone(),
                    schema_version: 1,
                    columns,
                    metadata: Some(v1::Metadata {
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

fn metadata(field: &FieldDescriptor) -> v1::Metadata {
    v1::Metadata {
        identifier: field.name().into(),
        label: field.name().into(),
        ..Default::default()
    }
}
fn value_type(field: &FieldDescriptor) -> v1::ValueType {
    if field.is_list() || field.is_map() {
        return v1::ValueType::List;
    }
    match field.kind() {
        Kind::Bool => v1::ValueType::Boolean,
        Kind::Int32 | Kind::Int64 | Kind::Sint32 | Kind::Sint64 | Kind::Sfixed32 | Kind::Sfixed64 | Kind::Enum(_) => v1::ValueType::Integer,
        Kind::Uint32 | Kind::Uint64 | Kind::Fixed32 | Kind::Fixed64 => v1::ValueType::UnsignedInteger,
        Kind::Double | Kind::Float => v1::ValueType::Number,
        Kind::String => v1::ValueType::Text,
        Kind::Bytes => v1::ValueType::Bytes,
        Kind::Message(message) if message.full_name() == "google.protobuf.Timestamp" => v1::ValueType::Datetime,
        Kind::Message(_) => v1::ValueType::Bytes,
    }
}

/* === Tests === */
