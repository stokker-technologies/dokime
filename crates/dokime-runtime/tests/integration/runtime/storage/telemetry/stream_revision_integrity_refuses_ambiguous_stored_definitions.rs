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
async fn stream_revision_integrity_refuses_ambiguous_stored_definitions() {
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
    let stream = dokime_domain::model::identity::Identifier::new("integrity.reading").unwrap();
    let old = dokime_runtime::telemetry::TelemetryContract::declared(stream.as_str(), "Reading").typed::<CaptureReading>(1);
    let new = dokime_runtime::telemetry::TelemetryContract::declared(stream.as_str(), "Reading").typed::<Reading>(1);
    let old = telemetry::codec(&old, &Default::default()).unwrap();
    let new =
        telemetry::TypedCodec::new(&stream, old.schema().version.get(), new.storage_schema.as_ref().unwrap(), &Default::default()).unwrap();
    storage.write_schema(old.schema().clone(), old.graph().clone()).await.unwrap();
    storage.write_schema(old.schema().clone(), old.graph().clone()).await.unwrap();
    storage.flush().await.unwrap();
    let identical = saved_codecs(&storage, &storage.cursor_saved(), None, &Default::default(), |_, _| true).await.unwrap();
    assert_eq!(identical.len(), 1);
    drop(identical);
    storage.write_schema(new.schema().clone(), new.graph().clone()).await.unwrap();
    storage.flush().await.unwrap();
    let error = saved_codecs(&storage, &storage.cursor_saved(), None, &Default::default(), |_, _| true).await.unwrap_err();
    assert!(matches!(error, Error::Storage(dokime_runtime::storage::Error::TelemetryIntegrity { stream: id }) if id == stream.as_str()));
    let error = local.telemetry_schemas(storage.cursor_saved(), Vec::new()).await.unwrap_err();
    assert!(matches!(error, dokime_runtime::storage::Error::TelemetryIntegrity { stream: id } if id == stream.as_str()));
    runtime.shutdown().await.unwrap();
}
