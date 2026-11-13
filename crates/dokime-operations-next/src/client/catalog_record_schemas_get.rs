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

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;
use prost::Message;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.v1.catalog.RecordSchemas/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::catalog::RecordSchemasGetRequest, v1::catalog::RecordSchemasGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::catalog::RecordSchemasGetRequest) -> Result<v1::catalog::RecordSchemasGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let key = format!("record-schema/{}/{}/{}", request.parent, request.name, request.schema_version);
        let (value, cut) = super::views::read(context, &key, request.at.as_ref(), async {
            if request.parent.is_empty() {
                let inventory = super::catalog_inventory::Inventory::read(context).await?;
                if let Some(value) = inventory.records.into_iter().find(|record| record.record_type == request.name) {
                    return Ok(value);
                }
            }

            if request.schema_version != 0 && request.schema_version != 1 {
                return Err(Error::Invalid("Get Captured Schema: Select Captured Schema Version 1".into()));
            }
            if request.name == "dokime.log" {
                let pool = prost_reflect::DescriptorPool::from_file_descriptor_set(prost_types::FileDescriptorSet::decode(
                    dokime_protocol::FILE_DESCRIPTOR_SET,
                )?)
                .map_err(|error| Error::Invalid(error.to_string()))?;
                let descriptor = pool
                    .get_message_by_name("dokime.context.v1.evidence.LogsAppendRequest")
                    .ok_or_else(|| Error::Invalid("Get Captured Log Schema: Rebuild the Host Protocol".into()))?;
                return Ok(v1::RecordSchema {
                    record_type: request.name,
                    schema_version: 1,
                    metadata: Some(v1::Metadata {
                        identifier: "dokime.log".into(),
                        label: "Execution Log".into(),
                        ..Default::default()
                    }),
                    fields: fields(descriptor),
                    default_severity: v1::Severity::Info as i32,
                    ..Default::default()
                });
            }
            let mut found = None;
            let mut frozen_shape = None;
            for id in context.host.store().identifiers()?.into_iter().filter(|id| request.parent.is_empty() || id == &request.parent) {
                let cut = context.host.store().metadata(&id)?;
                let catalog = harmos_protocol::v1::Catalog::decode(cut.execution.catalog.as_slice())?;
                for section in &catalog.sections {
                    if section.r#type != "dokime.v1.RecordContracts" {
                        continue;
                    }
                    for record in v1::RecordContracts::decode(section.payload.as_slice())?.records {
                        if record.metadata.as_ref().is_none_or(|metadata| metadata.identifier != request.name) {
                            continue;
                        }
                        let pool = prost_reflect::DescriptorPool::from_file_descriptor_set(
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
                        let candidate = v1::RecordSchema {
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
        Ok(v1::catalog::RecordSchemasGetResponse {
            value: Some(value),
            resolved_at: Some(cut),
        })
    })
    .await
}

/* === Private Functions === */
fn shape(descriptor: &prost_reflect::MessageDescriptor) -> Vec<u8> {
    let mut pending = vec![descriptor.clone()];
    let mut messages = std::collections::BTreeMap::new();
    let mut enums = std::collections::BTreeMap::new();
    while let Some(message) = pending.pop() {
        if messages.contains_key(message.full_name()) {
            continue;
        }
        messages.insert(message.full_name().to_owned(), message.descriptor_proto().encode_to_vec());
        for field in message.fields() {
            match field.kind() {
                prost_reflect::Kind::Message(message) => pending.push(message),
                prost_reflect::Kind::Enum(enumeration) => {
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

fn fields(descriptor: prost_reflect::MessageDescriptor) -> Vec<v1::RecordField> {
    descriptor
        .fields()
        .map(|field| {
            let value_type = if field.is_list() || field.is_map() {
                v1::ValueType::List
            } else {
                match field.kind() {
                    prost_reflect::Kind::Bool => v1::ValueType::Boolean,
                    prost_reflect::Kind::Double | prost_reflect::Kind::Float => v1::ValueType::Number,
                    prost_reflect::Kind::Uint32
                    | prost_reflect::Kind::Fixed32
                    | prost_reflect::Kind::Uint64
                    | prost_reflect::Kind::Fixed64 => v1::ValueType::UnsignedInteger,
                    prost_reflect::Kind::Int32
                    | prost_reflect::Kind::Sint32
                    | prost_reflect::Kind::Sfixed32
                    | prost_reflect::Kind::Int64
                    | prost_reflect::Kind::Sint64
                    | prost_reflect::Kind::Sfixed64
                    | prost_reflect::Kind::Enum(_) => v1::ValueType::Integer,
                    prost_reflect::Kind::String => v1::ValueType::Text,
                    prost_reflect::Kind::Message(message) if message.full_name() == "google.protobuf.Timestamp" => v1::ValueType::Datetime,
                    _ => v1::ValueType::Bytes,
                }
            };
            v1::RecordField {
                metadata: Some(v1::Metadata {
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
