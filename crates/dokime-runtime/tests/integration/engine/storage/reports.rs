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
use dokime_runtime::{
    core::{Identifier, TestCycleId, Timestamp},
    storage::{Backend, Manager, reports},
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
        dokime_runtime::storage::state_settings(),
    ));
    let storage = Manager::new(
        StorageBackends {
            primary,
            state: recovery,
        },
        super::sqlite::storage::storage_settings(),
    )
    .unwrap();
    let runtime = Runtime::<Reader>::builder((), ())
        .runtime_with_storage(&storage)
        .await
        .unwrap();
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
    file.write_all(b"<html>verified report</html>")
        .await
        .unwrap();
    let file = file.finish().await.unwrap();
    let saved = storage.flush_until(&file.position).await.unwrap();
    let at = Timestamp::new("2026-10-09T09:00:00Z");
    let cycle = dokime_runtime::execution::TestCycle {
        saved: None,
        test_cycle_id: owner.clone(),
        session_id: None,
        metadata: dokime_runtime::core::Metadata::new(
            Identifier::new("report.cycle").unwrap(),
            "Report Cycle",
            None,
        ),
        lifecycle: dokime_runtime::execution::CycleStatus::Completed,
        created_at: at.clone(),
        started_at: None,
        updated_at: at.clone(),
        execution_ids: Vec::new(),
        records: Default::default(),
        continue_on_failure: false,
        result: None,
    };
    let mut publication = reports::Prepared::new(
        &storage,
        &reports::Input {
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
        },
    )
    .unwrap();
    let receipt = publication.finish().await.unwrap();
    assert_eq!(publication.finish().await.unwrap(), receipt);
    assert_eq!(storage.status().accepted, receipt);
    drop(publication);
    runtime.stop().await.unwrap();

    let primary = Arc::new(Backend::primary(root.path().to_path_buf()).await.unwrap());
    let recovery = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        harmos::storage::StorageBackend::history(primary.as_ref()).clone(),
        dokime_runtime::storage::state_settings(),
    ));
    let storage = Manager::new(
        StorageBackends {
            primary,
            state: recovery,
        },
        super::sqlite::storage::storage_settings(),
    )
    .unwrap();
    let runtime = Runtime::<Reader>::builder((), ())
        .runtime_with_storage(&storage)
        .await
        .unwrap();
    let published = |owner, report_id| {
        let storage = storage.clone();
        async move {
            storage
                .read_records::<reports::Publication>(harmos::storage::RecordRead {
                    at: storage.cursor_saved(),
                    selection: reports::selection(owner, report_id),
                    after: 0,
                    filters: Vec::new(),
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
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].position, receipt);
        let publication = reports::Publication::from_record(page[0].clone()).unwrap();
        assert_eq!(publication.file().unwrap(), &file);
        assert_eq!(publication.cycle().unwrap().restore().unwrap(), cycle);
        assert_eq!(page[0].item, file.metadata.id);
    }
    assert!(
        published(reports::Owner::Cycle(&other), None)
            .await
            .is_empty()
    );
    runtime.stop().await.unwrap();
}
