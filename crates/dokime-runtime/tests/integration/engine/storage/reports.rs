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
use std::sync::Arc;

// Third Party Imports
use harmos::{
    Application, Runtime,
    state::Never,
    storage::{FileMetadata, StorageBackends},
};

// Local Imports
use {
    dokime_runtime::Identifier, dokime_runtime::TestCycleId, dokime_runtime::Timestamp, dokime_runtime::storage::Backend,
    dokime_runtime::storage::Manager, dokime_runtime::storage::reports,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Reader;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Application for Reader {
    type Lanes = dokime_runtime::app::Lanes;
    type State = ();
    type Transaction = Never;
    type Record = Never;
    type Streams = ();
    type Resources = ();
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn report_publication_reopens() {
    let root = tempfile::tempdir().unwrap();
    let primary = Arc::new(Backend::primary(root.path().to_path_buf()).await.unwrap());
    let recovery = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        harmos::storage::StorageBackend::history(primary.as_ref()).clone(),
        dokime_runtime::config::storage_state_defaults(),
    ));
    let storage = Manager::new(StorageBackends { primary, state: recovery }, super::sqlite::storage::storage_settings()).unwrap();
    let runtime = Runtime::<Reader>::builder((), ()).runtime_with_storage(&storage).await.unwrap();
    let owner = TestCycleId::from_seed("report-owner");
    let other = TestCycleId::from_seed("another-owner");
    let evidence = storage.cursor_saved();
    let mut file = storage
        .create_file(FileMetadata {
            id: Identifier::new("report.revision-one").unwrap(),
            scope: Some(Identifier::new(owner.as_str()).unwrap()),
            media_type: "text/html".into(),
            expected_length: None,
            expected_sha256: None,
        })
        .await
        .unwrap();
    file.write_all(b"<html>verified report</html>".to_vec()).await.unwrap();
    let file = file.finish().await.unwrap();
    let saved = storage.flush_until(&file.position).await.unwrap();
    let at = Timestamp::new("2026-10-09T09:00:00Z");
    let cycle = dokime_domain::model::execution::result::TestCycle {
        saved: None,
        test_cycle_id: owner.clone(),
        session_id: None,
        metadata: dokime_runtime::Metadata::new(Identifier::new("report.cycle").unwrap(), "Report Cycle", None),
        lifecycle: dokime_domain::model::execution::result::CycleStatus::Completed,
        phase: dokime_domain::model::execution::result::CyclePhase::Sealing,
        created_at: at.clone(),
        started_at: None,
        updated_at: at.clone(),
        execution_ids: Vec::new(),
        records: Default::default(),
        continue_automatically: false,
        result: None,
    };
    let input = reports::Input {
        owner: reports::Owner::Cycle(&owner),
        cycle: Some(&cycle),
        execution: None,
        generated_at: &at,
        final_report: true,
        evidence_through: &evidence,
        dependencies_saved: &saved,
        executions: &[],
        inputs: &[],
        file: &file,
    };
    let mut publication = reports::Prepared::new(&storage, &input, &Default::default()).unwrap();
    let receipt = publication.finish().await.unwrap();
    assert_eq!(publication.finish().await.unwrap(), receipt);
    assert_eq!(storage.status().accepted, receipt);
    drop(publication);
    runtime.stop().await.unwrap();

    let primary = Arc::new(Backend::primary(root.path().to_path_buf()).await.unwrap());
    let recovery = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        harmos::storage::StorageBackend::history(primary.as_ref()).clone(),
        dokime_runtime::config::storage_state_defaults(),
    ));
    let storage = Manager::new(StorageBackends { primary, state: recovery }, super::sqlite::storage::storage_settings()).unwrap();
    let runtime = Runtime::<Reader>::builder((), ()).runtime_with_storage(&storage).await.unwrap();
    let published = |owner, report_id| {
        let storage = storage.clone();
        async move {
            storage
                .read_records::<reports::Publication>(harmos::storage::RecordRead {
                    at: storage.cursor_saved(),
                    selection: reports::selection(owner, report_id),
                    after: 0,
                })
                .await
                .unwrap()
                .collect()
                .await
                .unwrap()
        }
    };
    for report_id in [None, Some(file.metadata.id.clone())] {
        let page = published(reports::Owner::Cycle(&owner), report_id).await;
        assert_eq!(page.iter().map(|batch| batch.len()).sum::<usize>(), 1);
        let record = page.iter().flat_map(|batch| batch.iter()).next().unwrap();
        assert_eq!(record.position, receipt);
        assert_eq!(record.item, file.metadata.id);
        let publication = page
            .try_map(|batches| {
                let batch = batches.into_iter().find(|batch| !batch.is_empty()).unwrap();
                batch.try_map(|mut records| reports::Publication::from_record(records.pop().unwrap()))
            })
            .unwrap();
        assert_eq!(publication.file().unwrap(), &file);
        assert_eq!(publication.cycle().unwrap().restore().unwrap(), cycle);
    }
    assert!(published(reports::Owner::Cycle(&other), None).await.iter().all(|batch| batch.is_empty()));
    runtime.stop().await.unwrap();
}
