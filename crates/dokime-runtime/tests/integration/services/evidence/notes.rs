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
use dokime_runtime::CommandId;

// Local Imports
use crate::service_support::prelude::*;
use dokime_runtime::domains::evidence::notes::{CreateNote, NotePublication};
use dokime_runtime::execution::notes::NoteOwner;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
async fn fixture(root: &std::path::Path) -> (Runtime, CreateNote) {
    let metadata = Metadata::new(Identifier::new("notes.fixture").unwrap(), "Notes", None);
    let owner = Runtime::start(crate::service_support::budgets::runtime_config(
        root,
        metadata.clone(),
    ))
    .await
    .unwrap();
    let handle = owner.handle();
    let cycle_id = handle
        .create_cycle(
            CycleDefinition {
                metadata,
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let request = CreateNote {
        epoch: handle.process_epoch(),
        mutation_id: dokime_runtime::model::identity::new_id::<CommandId>(),
        owner: NoteOwner::Cycle { cycle_id },
        actor: "operator".into(),
        message: "inspected".into(),
    };
    (owner, request)
}

/* === Tests === */

#[tokio::test]
async fn empty_cycle_datasets() {
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
