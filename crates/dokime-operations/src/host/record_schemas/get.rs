/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Catalog Record Schemas Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::host::ClientSettings;

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Error, Result, host::Context};

use dokime_protocol::host::RecordField;
use dokime_protocol::host::RecordSchema;
use dokime_protocol::host::RecordSchemasGetRequest;
use dokime_protocol::host::RecordSchemasGetResponse;
use dokime_protocol::model::Metadata;
use dokime_protocol::model::RecordContracts;
use dokime_protocol::model::Severity;
use dokime_protocol::model::ValueType;
use harmos_protocol::model::Declarations;
use prost_reflect::DescriptorPool;
use prost_reflect::Kind;
use prost_reflect::MessageDescriptor;
use prost_types::FileDescriptorSet;
use std::collections::BTreeMap;

use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.RecordSchemas/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<RecordSchemasGetRequest, RecordSchemasGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: RecordSchemasGetRequest) -> Result<RecordSchemasGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let key = format!("record-schema/{}/{}/{}", request.parent, request.name, request.schema_version);
        let (value, cut) = crate::host::views::read(context, &key, request.at.as_ref(), async {
            if request.parent.is_empty() {
                let inventory = crate::host::catalog_inventory::Inventory::read(context).await?;
                if let Some(value) = inventory.records.into_iter().find(|record| record.record_type == request.name) {
                    return Ok(value);
                }
            }

            if request.schema_version != 0 && request.schema_version != 1 {
                return Err(Error::Invalid("Get Captured Schema: Select Captured Schema Version 1".into()));
            }
            if request.name == "dokime.log" {
                let pool = DescriptorPool::from_file_descriptor_set(FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?)
                    .map_err(|error| Error::Invalid(error.to_string()))?;
                let descriptor = pool
                    .get_message_by_name("dokime.host.v1.evidence.LogsAppendRequest")
                    .ok_or_else(|| Error::Invalid("Get Captured Log Schema: Rebuild the Host Protocol".into()))?;
                return Ok(RecordSchema {
                    record_type: request.name,
                    schema_version: 1,
                    metadata: Some(Metadata {
                        identifier: "dokime.log".into(),
                        label: "Execution Log".into(),
                        ..Default::default()
                    }),
                    fields: fields(descriptor),
                    default_severity: Severity::Info as i32,
                    ..Default::default()
                });
            }
            let mut found = None;
            let mut frozen_shape = None;
            for id in context.host.store().identifiers()?.into_iter().filter(|id| request.parent.is_empty() || id == &request.parent) {
                let cut = context.host.store().metadata(&id)?;
                let catalog = Declarations::decode(cut.execution.catalog.as_slice())?;
                for section in &catalog.sections {
                    if section.r#type != "dokime.model.v1.RecordContracts" {
                        continue;
                    }
                    for record in RecordContracts::decode(section.payload.as_slice())?.records {
                        if record.metadata.as_ref().is_none_or(|metadata| metadata.identifier != request.name) {
                            continue;
                        }
                        let pool = DescriptorPool::from_file_descriptor_set(
                            catalog
                                .types
                                .clone()
                                .ok_or_else(|| Error::Invalid("Get Captured Schema: Restore Its Frozen Descriptors".into()))?,
                        )
                        .map_err(|error| Error::Invalid(error.to_string()))?;
                        let descriptor = pool
                            .get_message_by_name(&record.message)
                            .ok_or_else(|| Error::Invalid("Get Captured Schema: Restore Its Frozen Record Descriptor".into()))?;
                        let shape = shape(&descriptor);
                        if frozen_shape.as_ref().is_some_and(|existing| existing != &shape) {
                            return Err(Error::Invalid(
                                "Get Captured Schema: Inspect Each Execution's Frozen Catalog to Resolve Conflicting Captured Wire Shapes"
                                    .into(),
                            ));
                        }
                        frozen_shape = Some(shape);
                        let fields = fields(descriptor);
                        let candidate = RecordSchema {
                            record_type: request.name.clone(),
                            schema_version: 1,
                            metadata: record.metadata,
                            fields,
                            ..Default::default()
                        };
                        if found.as_ref().is_some_and(|existing| existing != &candidate) {
                            return Err(Error::Invalid(
                                "Get Captured Schema: Inspect the Execution's Frozen Catalog to Resolve Conflicting Captured Types".into(),
                            ));
                        }
                        found = Some(candidate);
                    }
                }
            }
            found.ok_or_else(|| Error::Invalid(format!("Get Captured Schema: Select a Captured Record Type {}", request.name)))
        })
        .await?;
        Ok(RecordSchemasGetResponse {
            value: Some(value),
            resolved_at: Some(cut),
        })
    })
    .await
}

/* === Private Functions === */
fn shape(descriptor: &MessageDescriptor) -> Vec<u8> {
    let mut pending = vec![descriptor.clone()];
    let mut messages = BTreeMap::new();
    let mut enums = BTreeMap::new();
    while let Some(message) = pending.pop() {
        if messages.contains_key(message.full_name()) {
            continue;
        }
        messages.insert(message.full_name().to_owned(), message.descriptor_proto().encode_to_vec());
        for field in message.fields() {
            match field.kind() {
                Kind::Message(message) => pending.push(message),
                Kind::Enum(enumeration) => {
                    enums.insert(enumeration.full_name().to_owned(), enumeration.enum_descriptor_proto().encode_to_vec());
                }
                _ => (),
            }
        }
    }
    let mut shape = Vec::new();
    for (name, bytes) in messages.into_iter().chain(enums) {
        shape.extend((name.len() as u64).to_le_bytes());
        shape.extend(name.as_bytes());
        shape.extend((bytes.len() as u64).to_le_bytes());
        shape.extend(bytes);
    }
    shape
}

fn fields(descriptor: MessageDescriptor) -> Vec<RecordField> {
    descriptor
        .fields()
        .map(|field| {
            let value_type = if field.is_list() || field.is_map() {
                ValueType::List
            } else {
                match field.kind() {
                    Kind::Bool => ValueType::Boolean,
                    Kind::Double | Kind::Float => ValueType::Number,
                    Kind::Uint32 | Kind::Fixed32 | Kind::Uint64 | Kind::Fixed64 => ValueType::UnsignedInteger,
                    Kind::Int32 | Kind::Sint32 | Kind::Sfixed32 | Kind::Int64 | Kind::Sint64 | Kind::Sfixed64 | Kind::Enum(_) => {
                        ValueType::Integer
                    }
                    Kind::String => ValueType::Text,
                    Kind::Message(message) if message.full_name() == "google.protobuf.Timestamp" => ValueType::Datetime,
                    _ => ValueType::Bytes,
                }
            };
            RecordField {
                metadata: Some(Metadata {
                    identifier: field.name().into(),
                    label: field.name().into(),
                    ..Default::default()
                }),
                value_type: value_type as i32,
                required: field.is_required(),
                nullable: field.supports_presence(),
                ..Default::default()
            }
        })
        .collect()
}

/* === Tests === */
