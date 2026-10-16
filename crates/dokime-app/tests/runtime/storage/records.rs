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
use crate::support::prelude::*;
use dokime_app::domains::evidence::RecordSelection;

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

pub(crate) async fn managed_records_spool() {
    let root = tempdir().unwrap();
    let settings = config(root.path());
    let mut runtime =
        Runtime::with_execution_fixtures_without_builtins(settings.clone(), std::iter::empty())
            .await
            .unwrap();
    runtime.shutdown().await.unwrap();
    let persistent = root_files(root.path());

    let mut runtime =
        Runtime::with_execution_fixtures_without_builtins(settings.clone(), std::iter::empty())
            .await
            .unwrap();
    let handle = runtime.handle();
    handle
        .add_operator_note(OperatorNoteRequest {
            scope: Some(RecordScope::default()),
            author: Some("operator".into()),
            message: "A durable observation".into(),
        })
        .await
        .unwrap();
    let selection = RecordSelection {
        record_types: vec![Identifier::new("operator.note_added").unwrap()],
        ..RecordSelection::default()
    };
    let records = handle.selected_records(selection.clone()).await.unwrap();
    assert_eq!(records.len(), 1);
    drop(handle);
    runtime.shutdown().await.unwrap();
    assert_eq!(
        root_files(root.path()),
        persistent,
        "record capture creates no separate process spool files"
    );

    let mut runtime =
        Runtime::with_execution_fixtures_without_builtins(settings, std::iter::empty())
            .await
            .unwrap();
    let recovered = runtime.handle().selected_records(selection).await.unwrap();
    assert_eq!(
        recovered, records,
        "global record visibility survives managed recovery"
    );
    runtime.shutdown().await.unwrap();
    assert_eq!(root_files(root.path()), persistent);
}

#[tokio::test]
async fn managed_records_kill() {
    if let Some(root) = std::env::var_os("DOKIME_RECORD_POSITION_KILL_ROOT") {
        record_position_writer(PathBuf::from(root)).await;
        return;
    }
    let root = tempdir().unwrap();
    let marker = root.path().join("saved-records.json");
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "storage::records::managed_records_kill",
            "--nocapture",
        ])
        .env("DOKIME_RECORD_POSITION_KILL_ROOT", root.path())
        .spawn()
        .unwrap();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    while !marker.exists() && tokio::time::Instant::now() < deadline {
        assert!(
            child.try_wait().unwrap().is_none(),
            "writer exited before saving records"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(marker.exists(), "writer did not save records");
    let old: Vec<dokime_app::records::RecordEntry> =
        serde_json::from_slice(&std::fs::read(marker).unwrap()).unwrap();
    let (mut owner, runtime) = started(config(root.path())).await;
    for index in 0..32 {
        publish_position_record(&runtime, format!("New Process {index}")).await;
    }
    let current = runtime
        .selected_records(position_selection())
        .await
        .unwrap();
    assert_eq!(
        current.len(),
        old.len() + 32,
        "physical paging must not skip a new visible record"
    );
    let new: Vec<_> = current
        .iter()
        .filter(|record| old.iter().all(|prior| prior.record_id != record.record_id))
        .collect();
    assert_eq!(new.len(), 32);
    assert!(old.iter().all(|prior| {
        current
            .iter()
            .any(|record| record.record_id == prior.record_id)
    }));
    assert!(
        new.iter()
            .all(|record| old.iter().all(|prior| prior.sequence != record.sequence)),
        "managed recovery preserves saved logical positions"
    );
    drop(runtime);
    owner.shutdown().await.unwrap();

    let (mut owner, runtime) = started(config(root.path())).await;
    assert_eq!(
        runtime
            .selected_records(position_selection())
            .await
            .unwrap(),
        current
    );
    drop(runtime);
    owner.shutdown().await.unwrap();

    use harmos::storage::StorageBackend;
    let backend = dokime::storage::Backend::primary(config(root.path()).storage_root)
        .await
        .unwrap();
    let saved = backend.cursor_saved().await.unwrap();
    let mut count = 0;
    let mut identities = std::collections::BTreeSet::new();
    let mut positions = std::collections::BTreeSet::new();
    for row in
        crate::support::backend::records(&backend, &saved, dokime::storage::records::schema()).await
    {
        let record = dokime::storage::records::RecordRow::decode_entry(&row.payload).unwrap();
        if record.source != "runtime"
            || record.record_type.as_str() != "calibration.field.completed"
        {
            continue;
        }
        assert_eq!(row.item.as_str(), record.record_id.as_str());
        count += 1;
        identities.insert(record.record_id.clone());
        positions.insert(record.sequence);
    }
    assert_eq!(
        count, 64,
        "both process histories remain in primary storage"
    );
    assert_eq!(identities.len(), 64);
    assert_eq!(
        positions.len(),
        64,
        "managed journal positions survive the kill"
    );
    backend.close().await.unwrap();
}

async fn record_position_writer(root: PathBuf) {
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let mut settings = config(&root);
    let (_owner, runtime) = started_with_fixture(settings, executor.clone()).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let cycle = start_cycle(&runtime, suite, false).await;
    for index in 0..32 {
        if index == 16 {
            // One record-free transaction makes both logical-position parities
            // available without relying on the number of boot transactions.
            runtime
                .update_cycle_policy(cycle.clone(), true, CommandContext::default())
                .await
                .unwrap();
        }
        publish_position_record(&runtime, format!("Old Process {index}")).await;
    }
    let old = runtime
        .selected_records(position_selection())
        .await
        .unwrap();
    assert_eq!(old.len(), 32);
    runtime.storage().flush().await.unwrap();
    runtime
        .run_cycle_cases(cycle, CommandContext::default())
        .await;
    executor.wait_until_entered().await;
    // The Shared Manager Saves Facts And Logical Recovery Before The Kill.
    let temporary = root.join("saved-records.tmp");
    std::fs::write(&temporary, serde_json::to_vec(&old).unwrap()).unwrap();
    std::fs::rename(temporary, root.join("saved-records.json")).unwrap();
    std::future::pending::<()>().await;
}

async fn publish_position_record(runtime: &RuntimeHandle, message: String) {
    use dokime_app::records::{
        ProductCalibrationDetails, ProductCalibrationKind, ProductCalibrationPhase,
    };
    let mut draft = ProductCalibrationDetails::new(
        "bench",
        ProductCalibrationKind::Field,
        ProductCalibrationPhase::Completed,
    )
    .into_draft("fixture.record_positions");
    draft.message = Some(message);
    runtime.publish_product_record(draft).await.unwrap();
}

fn position_selection() -> RecordSelection {
    RecordSelection {
        record_types: vec![Identifier::new("calibration.field.completed").unwrap()],
        ..Default::default()
    }
}

fn root_files(root: &Path) -> Vec<std::ffi::OsString> {
    let mut files = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    files.sort();

    files
}
