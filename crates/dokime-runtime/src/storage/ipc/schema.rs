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
    let mut fields = fields(graph, graph.root, &mut Vec::new())?;
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
        if column.value_type == ValueType::Object && data_type == DataType::Utf8 {
            metadata.insert("encoding".into(), "json".into());
        }
        let field = Arc::new(Field::new(name, data_type, !column.required || column.nullable).with_metadata(metadata));
        match existing {
            Some(index) => fields[index] = field,
            None => fields.push(field),
        }
    }
    // Metadata Has Its Own Namespace, So Arbitrary Authored Signal Names Cannot Collide.
    let storage = vec![
        Field::new("position", DataType::UInt64, false),
        Field::new("at", DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())), false),
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
        Field::new("native_metadata", DataType::Utf8, false),
        Field::new("original_strings", DataType::Utf8, false),
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
    let fields = fields.into_iter().map(|field| annotate(&field, contract.stream.as_str())).collect::<Vec<_>>();
    Ok(Arc::new(Schema::new_with_metadata(fields, metadata)))
}

/* === Private Functions === */

fn annotate(field: &Field, stream: &str) -> Field {
    let kind = match field.data_type() {
        DataType::Struct(fields) => {
            DataType::Struct(fields.iter().map(|field| Arc::new(annotate(field, stream))).collect::<Vec<_>>().into())
        }
        DataType::List(child) => DataType::List(Arc::new(annotate(child, stream))),
        kind => kind.clone(),
    };
    let mut metadata = field.metadata().clone();
    if !metadata.contains_key("unit") {
        metadata.insert("unit", "null");
    }
    if !metadata.contains_key("description") {
        metadata.insert("description", "");
    }
    metadata.insert("stream_id", stream.to_owned());
    field.clone().with_data_type(kind).with_metadata(metadata)
}

fn fields(graph: &Graph, root: usize, active: &mut Vec<usize>) -> Result<Vec<Arc<Field>>> {
    if active.contains(&root) {
        return Err(Error::Invalid("recursive telemetry schemas cannot be represented by Arrow"));
    }
    let definition = graph.definitions.get(root).ok_or(Error::Invalid("telemetry graph definition is absent"))?;
    active.push(root);
    let result = definition
        .iter()
        .map(|field| {
            let data_type = match field.kind {
                Kind::Bool => DataType::Boolean,
                Kind::I32 | Kind::I64 => DataType::Int64,
                Kind::U32 | Kind::U64 => DataType::UInt64,
                Kind::F32 | Kind::F64 => DataType::Float64,
                Kind::String => DataType::Utf8,
                Kind::Bytes => DataType::List(Arc::new(Field::new("item", DataType::UInt64, false))),
                Kind::Message { definition } if graph.definitions.get(definition).is_some_and(Vec::is_empty) => DataType::Utf8,
                Kind::Message { definition } => DataType::Struct(fields(graph, definition, active)?.into()),
            };
            let data_type = if field.cardinality == Cardinality::Repeated {
                let mut item = Field::new("item", data_type, false);
                if matches!(field.kind,Kind::Message { definition } if graph.definitions.get(definition).is_some_and(Vec::is_empty)) {
                    item = item.with_metadata(HashMap::from([("encoding".into(), "json".into())]));
                }
                DataType::List(Arc::new(item))
            } else {
                data_type
            };
            let mut metadata = HashMap::from([("unit".into(), "null".into()), ("description".into(), String::new())]);
            if matches!(field.kind,Kind::Message { definition } if graph.definitions.get(definition).is_some_and(Vec::is_empty)) {
                metadata.insert("encoding".into(), "json".into());
            }
            Ok(Arc::new(
                Field::new(field.name.as_ref(), data_type, matches!(field.cardinality, Cardinality::Optional | Cardinality::Variant))
                    .with_metadata(metadata),
            ))
        })
        .collect();
    active.pop();
    result
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime_domain::model::{
        identity::{SessionId, TelemetrySampleId},
        telemetry::{TelemetryColumn, TelemetryRole},
        time::Timestamp,
        values::Value,
    };
    use harmos::Message as _;

    #[harmos::message]
    struct Nested {
        label: String,
        count: u64,
    }

    #[harmos::message]
    struct Empty {}

    #[harmos::message]
    struct Reading {
        signed: i64,
        unsigned: u64,
        number: f64,
        flag: bool,
        text: String,
        items: Vec<Nested>,
        object: Nested,
        empty: Empty,
        empty_items: Vec<Empty>,
        integers: Vec<i64>,
        optional_object: Option<Nested>,
        optional: Option<i64>,
        timestamp: String,
        clock: String,
        opaque: String,
    }

    #[test]
    fn every_value_type_round_trips_with_metadata_and_exact_native_envelopes() {
        let row = Reading {
            signed: i64::MIN,
            unsigned: u64::MAX,
            number: 1.25,
            flag: true,
            text: "世界".into(),
            items: vec![Nested { label: "item".into(), count: u64::MAX }],
            object: Nested { label: "object".into(), count: 9 },
            empty: Empty {},
            empty_items: vec![Empty {}],
            integers: vec![i64::MIN, i64::MAX],
            optional_object: None,
            optional: None,
            timestamp: "1970-01-01T00:00:00.000001+00:00".into(),
            clock: "2026-10-30T00:00:00.123456+00:00".into(),
            opaque: r#"{ "value": [1, true] }"#.into(),
        };
        let contract = TelemetryContract::declared("roundtrip.fixture", "Round Trip")
            .typed::<Reading>()
            .column(TelemetryColumn::declared("clock", "Clock", ValueType::Datetime, TelemetryRole::Value))
            .column(TelemetryColumn::declared("opaque", "Opaque", ValueType::Object, TelemetryRole::Value));
        let codec = crate::storage::telemetry::codec(&contract, &Default::default()).unwrap();
        let timestamp = Timestamp::new("1970-01-01T00:00:00.000001Z");
        let mut sample = crate::telemetry::TelemetrySample::new(
            TelemetrySampleId::from_seed("roundtrip.sample"),
            contract.stream.clone(),
            1,
            timestamp,
            "fixture",
        )
        .typed(&row);
        sample.values.extend(codec.decode_payload(&row.encode()).unwrap());
        sample.values.insert("timestamp".into(), Value::Datetime(sample.timestamp.to_datetime().unwrap()));
        sample.summary.insert("unsigned".into(), Value::Unsigned(u64::MAX));
        let original = crate::storage::telemetry::TelemetryEntry {
            epoch: SessionId::from_seed("roundtrip.session"),
            ordinal: 1,
            sample,
        };
        let encoded = codec.encode(&original, usize::MAX).unwrap();
        let mut admitted = codec.decode_arrow(&encoded, usize::MAX).unwrap();
        let schema = schema(&contract, Some("execution")).unwrap();
        crate::storage::ipc::normalize_values(schema.field(1), &mut admitted.entry.sample.values).unwrap();
        fn metadata(field: &Field) {
            for key in ["unit", "description", "stream_id"] {
                assert!(field.metadata().contains_key(key));
            }
            match field.data_type() {
                DataType::Struct(fields) => {
                    for field in fields {
                        metadata(field);
                    }
                }
                DataType::List(item) => metadata(item),
                _ => {}
            }
        }
        for field in schema.fields() {
            metadata(field);
        }
        assert_eq!(schema.metadata().get("contract_hash").unwrap().len(), 64);
        let batch = crate::storage::ipc::record_batch(schema.clone(), &[(9, admitted)]).unwrap();
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("roundtrip.arrow");
        let mut writer = crate::storage::ipc::Writer::create(&path, schema.clone()).unwrap();
        let index = writer.append(&batch).unwrap();
        writer.finish().unwrap();
        let batch = crate::storage::ipc::read_batch(&path, schema, &index).unwrap();
        let (position, restored) = crate::storage::ipc::telemetry_entry(&batch, 0, &contract).unwrap();
        let mut tail = original.sample.values.clone();
        crate::storage::ipc::normalize_values(batch.schema().field(1), &mut tail).unwrap();
        assert_eq!(restored.sample.values, tail);
        assert_eq!(position, 9);
        for field in [
            "signed",
            "unsigned",
            "number",
            "flag",
            "text",
            "items",
            "object",
            "empty",
            "empty_items",
            "integers",
            "optional_object",
            "optional",
        ] {
            assert_eq!(restored.sample.values.get(field), original.sample.values.get(field), "{field}");
        }
        assert!(matches!(restored.sample.values.get("clock"), Some(Value::Datetime(_))));
        assert!(matches!(restored.sample.values.get("opaque"), Some(Value::Object(_))));
        assert_eq!(restored.sample.summary, original.sample.summary);
        let encoding = crate::storage::ipc::native_encoding(&batch, 0).unwrap();
        assert_eq!(codec.encode_arrow(&restored, encoding.0, encoding.1).unwrap(), encoded);
    }
}
