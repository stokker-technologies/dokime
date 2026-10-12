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
use std::num::{NonZeroU32, NonZeroUsize};

// Third Party Imports
use harmos::{
    schema::{
        Cardinality, Field, Kind,
        reflection::{DynamicMessage, Value},
    },
    storage::*,
};
use prost::Message;

// Local Imports
use dokime::storage::sqlite::{Backend, Limits};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

pub(crate) async fn saved_descriptors_reopen() {
    let root = tempfile::tempdir().unwrap();
    let fields = vec![
        Field {
            name: "voltage".into(),
            tag: 1,
            kind: Kind::F64,
            cardinality: Cardinality::Required,
        },
        Field {
            name: "counter".into(),
            tag: 2,
            kind: Kind::U64,
            cardinality: Cardinality::Required,
        },
        Field {
            name: "enabled".into(),
            tag: 3,
            kind: Kind::Bool,
            cardinality: Cardinality::Optional,
        },
        Field {
            name: "label".into(),
            tag: 4,
            kind: Kind::String,
            cardinality: Cardinality::Required,
        },
        Field {
            name: "waveform".into(),
            tag: 5,
            kind: Kind::F32,
            cardinality: Cardinality::Repeated,
        },
    ];
    let descriptor = harmos::schema::reflection::descriptor(&fields).unwrap();
    let mut payloads = Vec::new();
    for (voltage, enabled) in [(229.5, Some(false)), (230.5, None)] {
        let mut message = DynamicMessage::new(descriptor.clone());
        message.set_field_by_name("voltage", Value::F64(voltage));
        message.set_field_by_name("counter", Value::U64(u64::MAX));
        message.set_field_by_name("label", Value::String("Charger α".into()));
        message.set_field_by_name(
            "waveform",
            Value::List(vec![Value::F32(-1.25), Value::F32(1.25)]),
        );
        if let Some(enabled) = enabled {
            message.set_field_by_name("enabled", Value::Bool(enabled));
        }
        payloads.push(message.encode_to_vec());
    }
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let history = backend.history().clone();
    let position = |sequence| StorageCursor {
        history: history.clone(),
        sequence,
    };
    let key = SchemaKey {
        payload_encoding: harmos::storage::PayloadEncoding::ProtobufV1,
        id: harmos::Identifier::new("fixture.protobuf.samples").unwrap(),
        version: NonZeroU32::MIN,
    };
    let schemas = [StorageSchema::new(
        position(1),
        key.clone(),
        fields,
        SchemaEncoding::Inline,
    )];
    let rows: Vec<_> = payloads
        .iter()
        .enumerate()
        .map(|(index, payload)| StoredRow {
            record: StoredRecord {
                position: position(index as u64 + 2),
                schema: key.clone(),
                scope: None,
                item: harmos::Identifier::new(format!("sample.{index}")).unwrap(),
                source: None,
                source_time: None,
                payload: payload.clone(),
            },
            stream: harmos::Identifier::new("voltage").unwrap(),
            sequence: index as u64,
        })
        .collect();
    let parts = [StoragePart::Schemas(&schemas), StoragePart::Stream(&rows)];
    let batch = StorageBatch::new(&parts, NonZeroUsize::new(64 * 1024).unwrap()).unwrap();
    let saved = backend.write(batch).await.unwrap();
    backend.close().await.unwrap();
    drop(backend);
    drop(schemas);
    drop(rows);
    drop(descriptor);

    // Reconstruct From The Saved Descriptor Without The Original Producer
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history)
        .await
        .unwrap();
    let snapshot = backend.read_view(&saved).await.unwrap();
    let view = ReadView::new(
        harmos::Identifier::new("protobuf.saved.view").unwrap(),
        saved.clone(),
        saved,
        snapshot.clone(),
    )
    .unwrap();
    let limits = ReadLimits {
        rows: NonZeroUsize::new(10).unwrap(),
        bytes: NonZeroUsize::new(4096).unwrap(),
    };
    let mut selection = ReadSelection::new(ReadKind::Schemas);
    selection.schema = Some(key.clone());
    let schemas = backend
        .read_schemas(&view, &selection, None, limits)
        .await
        .unwrap();
    assert_eq!(schemas.entries.len(), 1);
    assert!(schemas.next.is_none());
    let schema = &schemas.entries[0];
    assert_eq!(schema.key, key);
    assert_eq!(schema.encoding(), SchemaEncoding::Inline);
    let descriptor = harmos::schema::reflection::descriptor(&schema.fields).unwrap();
    let mut selection = ReadSelection::new(ReadKind::Stream);
    selection.schema = Some(key.clone());
    let rows = backend
        .read_stream(&view, &selection, None, limits)
        .await
        .unwrap();
    assert_eq!(rows.entries.len(), 2);
    assert!(rows.next.is_none());
    let mut voltage_sum = 0.0;
    for (index, row) in rows.entries.iter().enumerate() {
        assert_eq!(row.record.payload, payloads[index]);
        let message =
            DynamicMessage::decode(descriptor.clone(), row.record.payload.as_slice()).unwrap();
        let voltage = message.get_field_by_name("voltage").unwrap();
        let Value::F64(voltage) = voltage.as_ref() else {
            panic!("saved voltage differs from its descriptor");
        };
        voltage_sum += voltage;
        assert_eq!(
            message.get_field_by_name("counter").unwrap().as_ref(),
            &Value::U64(u64::MAX)
        );
        assert_eq!(
            message.get_field_by_name("label").unwrap().as_ref(),
            &Value::String("Charger α".into())
        );
        assert_eq!(
            message.get_field_by_name("waveform").unwrap().as_ref(),
            &Value::List(vec![Value::F32(-1.25), Value::F32(1.25)])
        );
        assert_eq!(message.has_field_by_name("enabled"), index == 0);
        assert_eq!(message.encode_to_vec(), payloads[index]);
    }
    assert_eq!(voltage_sum, 460.0);
    backend.release_view(&snapshot).await.unwrap();
    backend.close().await.unwrap();
}
