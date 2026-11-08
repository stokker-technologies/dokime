/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod live_writes_keep_history_values_and_seek_pages_at_the_admitted_cut;

mod authenticated_cut_resumes_after_restart_and_rejects_deletion_or_generation_change;

mod cycle_membership_uses_fixed_catalog_and_object_frontiers_with_bounded_seeks;

mod runtime_snapshot_pages_bind_selection_and_allow_size_changes_during_writes;

mod captured_contracts_are_immutable_bounded_and_available_after_restart;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_domain::model::{
    execution::wire::ExecutionStatus,
    history::wire::ExecutionAvailability,
    identity::{Identifier, wire::Timestamp},
    query::{error::Reason, page::ExecutionSummary, selection::ExecutionSelection, snapshot::SnapshotRef},
};

use dokime_runtime::runtime::query::snapshots::{
    Cut, Scope,
    testing::{Key, Read, State, Tokens},
};

use rusqlite::params;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn expires() -> Timestamp {
    Timestamp { seconds: 100, nanos: 0 }
}

async fn store(root: &std::path::Path) -> State {
    State::open(root).await.unwrap()
}

async fn seed(state: &State, id: &str, revision: i64, position: u64, status: ExecutionStatus) {
    let id = id.to_owned();
    state.call(move |connection| {
        let transaction=connection.transaction()?;
        let summary=ExecutionSummary {
            execution_id:id.clone(),cycle_id:Some("cycle".into()),case_id:"case".into(),template_id:Some("query.fixture".into()),
            label:Some(id.clone()),status,started_at:Some(Timestamp{seconds:10,nanos:0}),finished_at:None,result:None,
            availability:ExecutionAvailability::Uncheckpointed,checkpoint_available:false,archived:false,
        };
        let position=format!("{position:016x}");
        transaction.execute("INSERT INTO storage_data_entries(position,kind,identity,schema_id,item,body,digest) VALUES(?1,'object',?1,'dokime.object.execution',?2,x'00',zeroblob(32))",params![position,id])?;
        let time=((10_000_000_000i128)^i128::MIN).to_be_bytes();
        transaction.execute("INSERT INTO query_history VALUES('execution',?1,?2,?3,'cycle','query.fixture',?4,0,0,?5,?6)",params![id,revision,position,serde_json::to_string(&status).unwrap(),time.as_slice(),serde_json::to_vec(&summary).unwrap()])?;
        transaction.execute("UPDATE storage_data_head SET saved=?1",[position])?;
        transaction.execute("UPDATE query_history_head SET revision=?1",[revision])?;
        transaction.commit()
    }).await.unwrap();
}

fn selection() -> ExecutionSelection {
    ExecutionSelection {
        template_ids: vec![],
        cycle_ids: vec![],
        statuses: vec![],
        started_at: None,
        archive: None,
    }
}

fn read(cut: &Cut, after: Option<Key>, rows: u32) -> Read {
    Read {
        revision: cut.catalog,
        cut: Some(cut.clone()),
        after,
        rows,
        bytes: 8192,
    }
}

fn signed(cut: &Cut) -> Cut {
    cut.clone()
}

/* === Tests === */
