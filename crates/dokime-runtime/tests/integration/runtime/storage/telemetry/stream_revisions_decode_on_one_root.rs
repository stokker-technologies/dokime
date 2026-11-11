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

// Third Party Imports

// Local Imports
use super::*;

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

#[tokio::test]
async fn stream_revisions_decode_on_one_root() {
    let root = tempfile::tempdir().unwrap();
    let backend = Arc::new(dokime_runtime::storage::Backend::primary(root.path().to_owned()).await.unwrap());
    let local = State::new(backend.local_state());
    let state = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        harmos::storage::StorageBackend::history(backend.as_ref()).clone(),
        dokime_runtime::config::storage_state_defaults(),
    ));
    let storage = Manager::new(StorageBackends { primary: backend, state }, dokime_runtime::config::storage_defaults()).unwrap();
    let runtime = Runtime::<TestApp>::launch((), StorageConfig { storage: Some(storage.clone()) }).await.unwrap();
    let old = dokime_runtime::telemetry::TelemetryContract::declared("revision.reading", "Reading").typed::<CaptureReading>(1);
    let new = dokime_runtime::telemetry::TelemetryContract::declared("revision.reading", "Reading").typed::<Reading>(1);
    let mut captured_rows = Vec::new();
    for (index, (contract, payload)) in [
        (old, harmos::Message::encode(&CaptureReading { value: 7.0 })),
        (new, harmos::Message::encode(&Reading { value: "changed shape".into() })),
    ]
    .into_iter()
    .enumerate()
    {
        let codec = Arc::new(telemetry::codec(&contract, &Default::default()).unwrap());
        local.register_telemetry_contract(contract.clone(), codec.clone()).await.unwrap();
        storage.write_schema(codec.schema().clone(), codec.graph().clone()).await.unwrap();
        let mut sample = dokime_runtime::telemetry::TelemetrySample::new(
            dokime_runtime::model::identity::new_id(),
            contract.stream.clone(),
            index as u64,
            dokime_domain::model::time::Timestamp::from_datetime(time::OffsetDateTime::UNIX_EPOCH),
            "revision.fixture",
        );
        sample.scope = Some(dokime_domain::model::execution::RecordScope {
            test_execution_id: Some(dokime_domain::model::identity::TestExecutionId::from_seed("revision.execution")),
            ..Default::default()
        });
        sample.schema_version = contract.schema_version;
        sample.values = decode_payload(&codec, &payload).unwrap();
        sample.values.insert("timestamp".into(), sample.timestamp.clone().into());
        sample.storage_payload = Some(payload);
        let entry = TelemetryEntry {
            epoch: dokime_domain::model::identity::SessionId::from_seed("revisions"),
            ordinal: index as u64,
            sample,
        };
        let payload = codec.encode(&entry, usize::MAX).unwrap();
        captured_rows.push((codec.schema().clone(), payload.clone()));
        storage
            .write_stream(vec![StreamInput {
                stream: Identifier::new("revision.reading").unwrap(),
                sequence: index as u64,
                record: RecordInput {
                    schema: codec.schema().clone(),
                    scope: Some(
                        Identifier::new(entry.sample.scope.as_ref().unwrap().test_execution_id.as_ref().unwrap().as_str()).unwrap(),
                    ),
                    item: Identifier::new(entry.sample.sample_id.as_str()).unwrap(),
                    source: Some(Identifier::new(entry.epoch.as_str()).unwrap()),
                    source_time: Some(std::time::UNIX_EPOCH),
                    payload,
                },
            }])
            .await
            .unwrap();
        storage.flush().await.unwrap();
    }
    // Report Materialization Resolves Persisted Definitions Before Decoding Captured Frames.
    let codecs = saved_codecs(&storage, &storage.cursor_saved(), None, &Default::default(), |_, _| true).await.unwrap();
    for (schema, payload) in captured_rows {
        let codec = codecs.iter().find(|codec| codec.schema() == &schema).unwrap();
        codec.decode(&payload, usize::MAX).unwrap();
    }
    drop(codecs);
    let page = collect_saved(&storage, &storage.cursor_saved(), &Default::default(), &Default::default(), &local).await.unwrap();
    assert_eq!(page.len(), 2);
    assert_eq!(page[0].values["value"], dokime_domain::model::values::Value::Number(7.0));
    assert_eq!(page[1].values["value"], dokime_domain::model::values::Value::Text("changed shape".into()));
    drop(page);
    runtime.shutdown().await.unwrap();
}
