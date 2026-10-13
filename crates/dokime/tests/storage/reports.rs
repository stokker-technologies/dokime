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
    storage::{FileMetadata, ReadLimits, StorageBackends},
};

// Local Imports
use dokime::{
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

pub(crate) async fn report_publication_reopens() {
    let root = tempfile::tempdir().unwrap();
    let primary = Arc::new(
        Backend::primary(root.path().to_path_buf(), Default::default())
            .await
            .unwrap(),
    );
    let recovery = Arc::new(primary.recovery(Default::default()));
    let storage = Manager::new(
        StorageBackends {
            primary,
            recovery,
            replicas: vec![],
        },
        Default::default(),
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
    let cycle = dokime::execution::TestCycle {
        saved: None,
        test_cycle_id: owner.clone(),
        session_id: None,
        metadata: dokime::core::Metadata::new(
            Identifier::new("report.cycle").unwrap(),
            "Report Cycle",
            None,
        ),
        lifecycle: dokime::execution::CycleStatus::Completed,
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

    let primary = Arc::new(
        Backend::primary(root.path().to_path_buf(), Default::default())
            .await
            .unwrap(),
    );
    let recovery = Arc::new(primary.recovery(Default::default()));
    let storage = Manager::new(
        StorageBackends {
            primary,
            recovery,
            replicas: vec![],
        },
        Default::default(),
    )
    .unwrap();
    let runtime = Runtime::<Reader>::builder((), ())
        .runtime_with_storage(&storage)
        .await
        .unwrap();
    for report_id in [None, Some(file.metadata.id.clone())] {
        let mut query = reports::query(reports::Owner::Cycle(&owner), report_id);
        query.options.page = Some(ReadLimits {
            rows: 1.try_into().unwrap(),
            bytes: (1024 * 1024).try_into().unwrap(),
        });
        let mut history = storage.query(query).await.unwrap();
        let page = history.next_page().await.unwrap().unwrap();
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].position, receipt);
        let publication = reports::Publication::from_record(&page[0]).unwrap();
        assert_eq!(publication.file().unwrap(), &file);
        assert_eq!(publication.cycle().unwrap().restore().unwrap(), cycle);
        assert_eq!(page[0].item, file.metadata.id);
        drop(page);
        assert!(history.next_page().await.unwrap().is_none());
    }
    let mut absent = storage
        .query(reports::query(reports::Owner::Cycle(&other), None))
        .await
        .unwrap();
    assert!(absent.next_page().await.unwrap().is_none());
    drop(absent);
    runtime.stop().await.unwrap();
}
