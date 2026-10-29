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
use model::execution::notes::NoteOwner;

// Local Imports
use dokime_domain::model::identity::CommandId;
use crate::service_support::prelude::*;
use dokime_domain::model;
use dokime_runtime::domains::evidence::notes::{CreateNote, NotePublication};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn empty_cycle_datasets(owner: &Runtime) {
    let runtime = owner.handle();
    let metadata = Metadata::new(Identifier::new("notes.fixture").unwrap(), "Notes", None);
    let cycle = runtime.create_cycle(CycleDefinition { metadata, continue_automatically: false }, CommandContext::default()).await.unwrap();
    let request = CreateNote {
        epoch: runtime.process_epoch(),
        mutation_id: dokime_runtime::model::identity::new_id::<CommandId>(),
        owner: NoteOwner::Cycle { cycle_id: cycle },
        actor: "operator".into(),
        message: "inspected".into(),
    };
    let cycle_id = request.owner.cycle_id().to_string();
    let before = runtime
        .storage()
        .read_object::<dokime_runtime::storage::objects::cycle::Cycle>(&cycle_id, harmos::storage::ObjectVersion::Latest)
        .await
        .unwrap()
        .unwrap();
    let committed = runtime.create_current_note(request.clone()).await.unwrap();
    assert_eq!(committed.publication, NotePublication::NoExistingDatasets);
    let retry = runtime.create_current_note(request).await.unwrap();
    assert_eq!(retry.admission, committed.admission);
    let snapshot = runtime.watch_state().latest();
    assert_eq!(snapshot.state().note_admissions.len(), 1);
    assert_eq!(snapshot.state().iter_notes().count(), 1);
    let after = runtime
        .storage()
        .read_object::<dokime_runtime::storage::objects::cycle::Cycle>(&cycle_id, harmos::storage::ObjectVersion::Latest)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(after.version, before.version, "notes and retries do not rewrite an unrelated cycle projection");
}

/* === Private Functions === */

/* === Tests === */
