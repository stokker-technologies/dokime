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
async fn later_refusal_preserves_the_accepted_prefix_without_replay() {
    let root = tempfile::tempdir().unwrap();
    let backend = Arc::new(dokime_runtime::storage::Backend::primary(root.path().to_owned()).await.unwrap());
    let local = State::new(backend.local_state());
    let mut settings = dokime_runtime::config::storage_defaults();
    settings.request.max_pending_write_bytes = NonZeroUsize::new(128 * 1024).unwrap();
    let state = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        harmos::storage::StorageBackend::history(backend.as_ref()).clone(),
        dokime_runtime::config::storage_state_defaults(),
    ));
    let storage = Manager::new(StorageBackends { primary: backend, state }, settings).unwrap();
    let runtime = Runtime::<TestApp>::launch((), StorageConfig { storage: Some(storage.clone()) }).await.unwrap();
    let contract = dokime_runtime::telemetry::TelemetryContract::declared("partial.fixture", "Partial").typed::<Reading>(1);
    let codec = Arc::new(telemetry::codec(&contract, &Default::default()).unwrap());
    local.register_telemetry_contract(contract.clone(), codec.clone()).await.unwrap();
    storage.write_schema(codec.schema().clone(), Arc::clone(codec.graph())).await.unwrap();
    let execution = dokime_domain::model::identity::TestExecutionId::from_seed("partial.scope");
    let mut rows = Vec::with_capacity(2);
    let epoch = dokime_domain::model::identity::SessionId::from_seed("partial.epoch");
    for (index, value) in ["first".to_owned(), "x".repeat(256 * 1024)].into_iter().enumerate() {
        let row = Reading { value };
        let mut sample = dokime_runtime::telemetry::TelemetrySample::new(
            dokime_runtime::model::identity::new_id(),
            contract.stream.clone(),
            index as u64,
            dokime_domain::model::time::Timestamp::from_datetime(time::OffsetDateTime::UNIX_EPOCH),
            "partial.fixture",
        )
        .typed(&row);
        sample.scope = Some(dokime_domain::model::execution::RecordScope {
            test_execution_id: Some(execution.clone()),
            ..Default::default()
        });
        rows.push(PreparedSample {
            sample: Arc::new(sample),
            lease: None,
            contract: 0,
            ordinal: index as u64,
        });
    }
    let rows = stage_rows(&epoch, &rows, &[codec]).unwrap();
    let before = storage.cursor_accepted();
    let admission = Admission { storage: storage.clone(), local, rows };
    let result = admission.commit(true).await.unwrap();
    assert_eq!(&*result.accepted, &[true, false]);
    assert!(result.failure.is_some());
    assert_eq!(storage.cursor_accepted().sequence, before.sequence + 1);
    drop(result);
    storage.flush().await.unwrap();
    runtime.shutdown().await.unwrap();
}
