/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Contract-Declared Arrow Telemetry Values

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::HashMap, sync::Arc};

// Third Party Imports
use arrow_schema::{DataType, Field, Schema, SchemaRef, TimeUnit};
use harmos::schema::{
    Cardinality,
    graph::{Kind, Schema as Graph},
};
use sha2::{Digest, Sha256};

// Local Imports
use crate::storage::{Error, Result};
use dokime_domain::model::{telemetry::TelemetryContract, values::ValueType};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn schema(contract: &TelemetryContract, execution: Option<&str>) -> Result<SchemaRef> {
    let graph = contract.storage_schema.as_ref().ok_or(Error::Invalid("telemetry contract has no storage graph"))?;
    let mut fields = fields(graph, graph.root)?;
    for column in &contract.columns {
        let name = column.metadata.identifier.as_str();
        let existing = fields.iter().position(|field| field.name() == name);
        let data_type = match column.value_type {
            ValueType::Datetime => DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())),
            ValueType::Object if existing.is_none() => DataType::Utf8,
            _ => match existing {
                Some(index) => fields[index].data_type().clone(),
                None => return Err(Error::Invalid("telemetry column is absent from its graph")),
            },
        };
        let mut metadata = HashMap::from([
            ("stream_id".into(), contract.stream.to_string()),
            ("description".into(), column.metadata.description.clone().unwrap_or_default()),
        ]);
        metadata.insert("unit".into(), serde_json::to_string(&column.unit)?);
        let field = Arc::new(Field::new(name, data_type, !column.required || column.nullable).with_metadata(metadata));
        match existing {
            Some(index) => fields[index] = field,
            None => fields.push(field),
        }
    }
    // Metadata Has Its Own Namespace, So Arbitrary Authored Signal Names Cannot Collide.
    let storage = vec![
        Field::new("position", DataType::UInt64, false),
        Field::new("epoch", DataType::Utf8, false),
        Field::new("ordinal", DataType::UInt64, false),
        Field::new("sample_id", DataType::Utf8, false),
        Field::new("sequence", DataType::UInt64, false),
        Field::new("timestamp", DataType::Utf8, false),
        Field::new("received_at", DataType::Utf8, false),
        Field::new("source", DataType::Utf8, false),
        Field::new("scope", DataType::Utf8, true),
        Field::new("summary", DataType::Utf8, false),
        Field::new("metadata", DataType::Utf8, false),
    ];
    let contract_bytes = serde_json::to_vec(contract)?;
    let digest = Sha256::digest(&contract_bytes);
    let hash: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    let metadata = HashMap::from([
        ("execution_id".into(), execution.unwrap_or_default().to_owned()),
        ("dokime_version".into(), env!("CARGO_PKG_VERSION").to_owned()),
        ("contract_hash".into(), hash),
        ("contract".into(), String::from_utf8(contract_bytes).map_err(|_| Error::Invalid("contract is not UTF-8"))?),
    ]);
    let fields =
        vec![Field::new("_storage", DataType::Struct(storage.into()), false), Field::new("values", DataType::Struct(fields.into()), false)];
    Ok(Arc::new(Schema::new_with_metadata(fields, metadata)))
}

/* === Private Functions === */

fn fields(graph: &Graph, root: usize) -> Result<Vec<Arc<Field>>> {
    let definition = graph.definitions.get(root).ok_or(Error::Invalid("telemetry graph definition is absent"))?;
    definition
        .iter()
        .map(|field| {
            let data_type = match field.kind {
                Kind::Bool => DataType::Boolean,
                Kind::I32 | Kind::I64 => DataType::Int64,
                Kind::U32 | Kind::U64 => DataType::UInt64,
                Kind::F32 | Kind::F64 => DataType::Float64,
                Kind::String => DataType::Utf8,
                Kind::Bytes => DataType::List(Arc::new(Field::new("item", DataType::UInt64, false))),
                Kind::Message { definition } => DataType::Struct(fields(graph, definition)?.into()),
            };
            let data_type = if field.cardinality == Cardinality::Repeated {
                DataType::List(Arc::new(Field::new("item", data_type, false)))
            } else {
                data_type
            };
            let metadata = HashMap::from([("unit".into(), "null".into()), ("description".into(), String::new())]);
            Ok(Arc::new(
                Field::new(field.name.as_ref(), data_type, matches!(field.cardinality, Cardinality::Optional | Cardinality::Variant))
                    .with_metadata(metadata),
            ))
        })
        .collect()
}

/* === Tests === */
