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
use dokime::core::CommandId;

// Local Imports
use crate::support::prelude::*;
use dokime_app::domains::evidence::notes::{CreateNoteRequest, NotePublication};
use dokime_app::execution::notes::NoteOwner;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
async fn fixture(root: &std::path::Path) -> (Runtime, CreateNoteRequest) {
    let metadata = Metadata::new(Identifier::new("notes.fixture").unwrap(), "Notes", None);
    let owner = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        Runtime::start(RuntimeConfig::new(root, metadata.clone())),
    )
    .await
    .expect("runtime boot stalled")
    .unwrap();
    let handle = owner.handle();
    let cycle_id = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        handle.start_cycle(
            CycleDefinition {
                suite: TestSuite {
                    metadata: metadata.clone(),
                    tags: vec![],
                    parameter_groups: vec![],
                    test_case_templates: vec![],
                },
                metadata,
                cases: vec![],
                continue_on_failure: false,
            },
            CommandContext::default(),
        ),
    )
    .await
    .expect("cycle creation stalled")
    .unwrap();
    let request = CreateNoteRequest {
        epoch: handle.process_epoch(),
        mutation_id: CommandId::new(),
        owner: NoteOwner::Cycle { cycle_id },
        actor: "operator".into(),
        message: "inspected".into(),
    };
    (owner, request)
}

/* === Tests === */

pub(crate) async fn empty_cycle_datasets() {
    let root = tempdir().unwrap();
    let (mut owner, request) = fixture(root.path()).await;
    let runtime = owner.handle();
    let committed = runtime.create_current_note(request.clone()).await.unwrap();
    assert_eq!(committed.publication, NotePublication::NoExistingDatasets);
    let retry = runtime.create_current_note(request).await.unwrap();
    assert_eq!(retry.admission, committed.admission);
    let snapshot = runtime.watch_state().latest();
    assert_eq!(snapshot.state().note_admissions.len(), 1);
    assert_eq!(snapshot.state().iter_notes().count(), 1);
    owner.shutdown().await.unwrap();
}
