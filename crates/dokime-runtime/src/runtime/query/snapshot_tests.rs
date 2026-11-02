/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Snapshot Cut, Seek, Authentication And Restart Proofs

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use rusqlite::params;
use serde_json::json;

// Local Imports
use super::*;
use crate::storage::sqlite::{
    State,
    history::{Key, Read},
};
use dokime_domain::model::{
    execution::wire::ExecutionStatus,
    history::wire::ExecutionAvailability,
    query::{page::ExecutionSummary, selection::ExecutionSelection},
};

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
    let state = State::open(root).await.unwrap();
    state.data_open(None).await.unwrap();
    state
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

fn signed(cut: &Cut) -> Token {
    Token::Snapshot(cut.clone())
}

/* === Tests === */

#[tokio::test]
async fn live_writes_keep_history_values_and_seek_pages_at_the_admitted_cut() {
    let root = tempfile::tempdir().unwrap();
    let state = store(root.path()).await;
    seed(&state, "a", 1, 1, ExecutionStatus::Running).await;
    seed(&state, "b", 2, 2, ExecutionStatus::Failed).await;
    let cut = state
        .snapshot_capture(Scope::History, None, None, expires(), b"{\"records\":[],\"telemetry\":[]}".to_vec(), None)
        .await
        .unwrap()
        .unwrap();
    let execution_cut = state
        .snapshot_capture(Scope::Execution("a".into()), None, None, expires(), b"{\"records\":[],\"telemetry\":[]}".to_vec(), None)
        .await
        .unwrap()
        .unwrap();
    let first = state.history_executions(selection(), read(&cut, None, 1)).await.unwrap();
    seed(&state, "a", 3, 3, ExecutionStatus::Passed).await;
    seed(&state, "c", 4, 4, ExecutionStatus::Failed).await;
    state
        .call(|connection| {
            connection.execute("DELETE FROM storage_data_entries WHERE position='0000000000000001'", [])?;
            connection.execute("UPDATE storage_data_head SET retention_generation='0000000000000001'", [])
        })
        .await
        .unwrap();
    assert!(state.snapshot_available(execution_cut).await.unwrap());
    let page = state.history_executions(selection(), read(&cut, first.next, 3)).await.unwrap();
    assert_eq!(page.rows.iter().map(|row| row.execution_id.as_str()).collect::<Vec<_>>(), ["b"]);
    let repeat = state.history_executions(selection(), read(&cut, None, 3)).await.unwrap();
    assert!(repeat.rows[0].status == ExecutionStatus::Running);
    let fresh = state
        .snapshot_capture(Scope::History, None, None, expires(), b"{\"records\":[],\"telemetry\":[]}".to_vec(), None)
        .await
        .unwrap()
        .unwrap();
    let fresh = state.history_executions(selection(), read(&fresh, None, 3)).await.unwrap();
    assert_eq!(fresh.rows.len(), 3);
    assert!(fresh.rows[0].status == ExecutionStatus::Passed);
    // A Flush Can Save Beyond The Requested Frontier; Metadata Still Uses The Original Cut.
    let overshot = state
        .snapshot_capture(Scope::History, Some(cut.at.clone()), None, expires(), b"{\"records\":[],\"telemetry\":[]}".to_vec(), None)
        .await
        .unwrap()
        .unwrap();
    let original = state.history_executions(selection(), read(&overshot, None, 3)).await.unwrap();
    assert_eq!(original.rows.len(), 2);
    assert!(original.rows[0].status == ExecutionStatus::Running);
    state.close().await.unwrap();
}

#[tokio::test]
async fn authenticated_cut_resumes_after_restart_and_rejects_deletion_or_generation_change() {
    let root = tempfile::tempdir().unwrap();
    let state = store(root.path()).await;
    seed(&state, "a", 1, 1, ExecutionStatus::Running).await;
    let cut = state
        .snapshot_capture(Scope::Execution("a".into()), None, None, expires(), b"{\"records\":[],\"telemetry\":[]}".to_vec(), None)
        .await
        .unwrap()
        .unwrap();
    let tokens = Tokens {
        key: state.snapshot_key().await.unwrap(),
        maximum: 4096,
    };
    let token = tokens.encode(&signed(&cut), "dokime.query.list_records").unwrap();
    state.close().await.unwrap();
    drop(state);
    let state = store(root.path()).await;
    let tokens = Tokens {
        key: state.snapshot_key().await.unwrap(),
        maximum: 4096,
    };
    let Token::Snapshot(resumed) = tokens.decode(&token, "dokime.query.list_records").unwrap() else { panic!("wrong token kind") };
    assert!(resumed == cut);
    assert!(state.snapshot_available(resumed.clone()).await.unwrap());
    state.call(|connection| connection.execute("UPDATE storage_data_head SET retention_generation='0000000000000001'", [])).await.unwrap();
    assert!(state.snapshot_available(resumed.clone()).await.unwrap());
    state.call(|connection| connection.execute("UPDATE query_token_key SET generation='0000000000000001'", [])).await.unwrap();
    assert!(!state.snapshot_available(resumed.clone()).await.unwrap());
    let resumed = state
        .snapshot_capture(Scope::Execution("a".into()), None, None, expires(), b"{\"records\":[],\"telemetry\":[]}".to_vec(), None)
        .await
        .unwrap()
        .unwrap();
    let mut foreign = resumed.clone();
    foreign.at.history.epoch = Identifier::new("other.epoch").unwrap();
    assert!(!state.snapshot_available(foreign).await.unwrap());
    state.call(|connection| connection.execute("DELETE FROM storage_data_entries WHERE position='0000000000000001'", [])).await.unwrap();
    assert!(!state.snapshot_available(resumed.clone()).await.unwrap());
    let error = state.history_executions(selection(), read(&resumed, None, 1)).await.err().unwrap();
    assert!(matches!(error, crate::storage::Error::QuerySnapshotUnavailable));
    state.close().await.unwrap();
}

#[tokio::test]
async fn cycle_membership_uses_fixed_catalog_and_object_frontiers_with_bounded_seeks() {
    let root = tempfile::tempdir().unwrap();
    let state = store(root.path()).await;
    seed(&state, "a", 1, 1, ExecutionStatus::Running).await;
    seed(&state, "b", 2, 2, ExecutionStatus::Pending).await;
    seed(&state, "cycle", 3, 3, ExecutionStatus::Pending).await;
    state
        .call(|connection| {
            connection.execute("UPDATE storage_data_entries SET schema_id='dokime.object.cycle' WHERE item='cycle'", [])?;
            connection.execute("UPDATE query_history SET kind='cycle' WHERE id='cycle'", [])?;
            connection.execute("INSERT INTO query_members VALUES('cycle','0000000000000003','a')", [])?;
            Ok(())
        })
        .await
        .unwrap();
    let cut = state
        .snapshot_capture(Scope::Cycle("cycle".into()), None, None, expires(), b"{\"records\":[],\"telemetry\":[]}".to_vec(), None)
        .await
        .unwrap()
        .unwrap();
    seed(&state, "c", 4, 4, ExecutionStatus::Pending).await;
    seed(&state, "cycle", 5, 5, ExecutionStatus::Pending).await;
    state
        .call(|connection| {
            connection.execute("UPDATE storage_data_entries SET schema_id='dokime.object.cycle' WHERE item='cycle'", [])?;
            connection.execute("UPDATE query_history SET kind='cycle' WHERE id='cycle'", [])?;
            connection.execute("INSERT INTO query_members VALUES('cycle','0000000000000005','a'),('cycle','0000000000000005','b')", [])?;
            Ok(())
        })
        .await
        .unwrap();
    assert_eq!(state.snapshot_members(cut.clone(), String::new(), 1).await.unwrap().unwrap(), ["a"]);
    assert!(state.snapshot_members(cut, "a".into(), 1).await.unwrap().unwrap().is_empty());
    let fresh = state
        .snapshot_capture(Scope::Cycle("cycle".into()), None, None, expires(), b"{\"records\":[],\"telemetry\":[]}".to_vec(), None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(state.snapshot_members(fresh, "a".into(), 1).await.unwrap().unwrap(), ["b"]);
    state.close().await.unwrap();
}

#[tokio::test]
async fn tokens_are_bounded_authenticated_expiring_and_scoped() {
    let root = tempfile::tempdir().unwrap();
    let state = store(root.path()).await;
    let cut = state
        .snapshot_capture(Scope::History, None, None, expires(), b"{\"records\":[],\"telemetry\":[]}".to_vec(), None)
        .await
        .unwrap()
        .unwrap();
    let tokens = Tokens {
        key: state.snapshot_key().await.unwrap(),
        maximum: 4096,
    };
    let token = tokens.encode(&signed(&cut), "dokime.query.list_records").unwrap();
    let mut changed = token.as_bytes().to_vec();
    changed[3] = if changed[3] == b'A' { b'B' } else { b'A' };
    assert!(tokens.decode(std::str::from_utf8(&changed).unwrap(), "dokime.query.list_records").is_err());
    let foreign = Tokens { key: [0; 32], maximum: 4096 };
    assert!(foreign.decode(&token, "dokime.query.list_records").is_err());
    assert!(
        validate_cut(&cut, &Scope::Execution("a".into()), Timestamp { seconds: 99, nanos: 999_999_999 }, "dokime.query.list_records")
            .is_ok()
    );
    assert!(validate_cut(&cut, &Scope::History, expires(), "dokime.query.list_records").unwrap_err().reason == Reason::SnapshotExpired);
    let mut scoped = cut.clone();
    scoped.scope = Scope::Execution("a".into());
    assert!(validate_cut(&scoped, &Scope::Execution("b".into()), Timestamp::default(), "dokime.query.list_records").is_err());
    scoped.source = "external".into();
    assert!(
        validate_cut(&scoped, &scoped.scope, Timestamp::default(), "dokime.query.list_records").unwrap_err().reason
            == Reason::UnsupportedSource
    );
    let tiny = Tokens { key: tokens.key, maximum: 32 };
    assert!(tiny.encode(&signed(&cut), "dokime.query.list_records").is_err());
    state.close().await.unwrap();
}

#[test]
fn seek_binding_ignores_page_sizes_and_conjunction_order_but_preserves_projection_order() {
    let first = json!({"scope":"a","where":[{"field":"value","operator":"gte","values":[1]},{"field":"status","operator":"eq","values":["failed"]}],"fields":["cpu.load","memory.used"],"page":{"rows":1},"at":null});
    let mut reordered = first.clone();
    reordered["where"].as_array_mut().unwrap().reverse();
    reordered["page"] = json!({"rows":12,"bytes":8192,"after":"opaque"});
    let operation = "dokime.query.list_samples";
    let binding = tokens::binding(&first, 8192, operation).unwrap();
    assert_eq!(binding, tokens::binding(&reordered, 8192, operation).unwrap());
    reordered["fields"].as_array_mut().unwrap().reverse();
    assert_ne!(binding, tokens::binding(&reordered, 8192, operation).unwrap());
    assert_ne!(binding, tokens::binding(&first, 8192, "dokime.query.list_records").unwrap());
}

#[tokio::test]
async fn runtime_snapshot_pages_bind_selection_and_allow_size_changes_during_writes() {
    use crate::{Runtime, config::RuntimeConfig, transactions::CycleDefinition};
    use dokime_domain::model::{
        identity::Metadata,
        query::{
            page::PageRequest,
            selection::{CycleScope, CycleSelection, RecordScope},
        },
    };
    let root = tempfile::tempdir().unwrap();
    let metadata = Metadata::new(Identifier::new("query.fixture").unwrap(), "Query", None);
    let config = RuntimeConfig::new(root.path(), metadata.clone()).with_credential_store(keyring_core::mock::Store::new().unwrap());
    let mut runtime = Runtime::start(config).await.unwrap();
    let handle = runtime.handle();
    let first = handle
        .create_cycle(
            CycleDefinition {
                metadata: metadata.clone(),
                continue_automatically: false,
            },
            Default::default(),
        )
        .await
        .unwrap();
    let second = handle
        .create_cycle(
            CycleDefinition {
                metadata: metadata.clone(),
                continue_automatically: false,
            },
            Default::default(),
        )
        .await
        .unwrap();
    let captured = handle.query_snapshot(Scope::Cycle(first.to_string()), None, None, [0; 32], "dokime.query.list_records").await.unwrap();
    assert!(handle.query_cycle_members(&captured, "", 1, "dokime.query.list_records").await.unwrap().is_empty());
    let contracts = handle.query_snapshot_contracts(&captured, "dokime.query.list_records").await.unwrap();
    let contracts: serde_json::Value = serde_json::from_slice(&contracts).unwrap();
    assert!(contracts["records"].is_array());
    let contract = crate::records::builtin_record_contracts().into_iter().next().unwrap();
    let records = handle
        .query_list_records(
            dokime_operations::query::ListRecords {
                scope: RecordScope::Cycle(CycleScope {
                    cycle_id: first.to_string(),
                    execution_id: None,
                }),
                record_type: contract.record_type.to_string(),
                versions: vec![],
                r#where: vec![],
                between: None,
                page: PageRequest { rows: None, bytes: None, after: None },
                at: Some(SnapshotRef::Token(captured.snapshot.token)),
            },
            65536,
        )
        .await
        .unwrap();
    assert!(records.complete);
    let mut request = dokime_operations::query::ListCycles {
        selection: CycleSelection {
            cycle_ids: vec![],
            phases: vec![],
            statuses: vec![],
            created_at: None,
            archive: None,
        },
        page: PageRequest { rows: Some(1), bytes: None, after: None },
        at: None,
    };
    let page = handle.query_list_cycles(request.clone(), 65536).await.unwrap();
    assert_eq!(page.rows[0].cycle_id, first.to_string());
    assert!(!page.complete);
    let snapshot = page.snapshot.unwrap();
    handle.create_cycle(CycleDefinition { metadata, continue_automatically: false }, Default::default()).await.unwrap();
    request.at = Some(SnapshotRef::Token(snapshot.token));
    request.page.after = page.next;
    request.page.rows = Some(3);
    let page = handle.query_list_cycles(request.clone(), 65536).await.unwrap();
    assert_eq!(page.rows.len(), 1);
    assert_eq!(page.rows[0].cycle_id, second.to_string());
    assert!(page.complete);
    let repeated = handle.query_list_cycles(request.clone(), 65536).await.unwrap();
    assert!(page == repeated);
    request.selection.cycle_ids = vec![first.to_string()];
    assert!(handle.query_list_cycles(request.clone(), 65536).await.err().unwrap().reason == Reason::InvalidContinuation);
    request.page.after = None;
    request.page.bytes = Some(32);
    assert!(handle.query_list_cycles(request, 65536).await.err().unwrap().reason == Reason::ResourceLimit);
    runtime.shutdown().await.unwrap();
}

#[tokio::test]
async fn captured_contracts_are_immutable_bounded_and_available_after_restart() {
    let root = tempfile::tempdir().unwrap();
    let state = store(root.path()).await;
    let body = b"{\"records\":[],\"telemetry\":[]}".to_vec();
    let cut = state.snapshot_capture(Scope::History, None, None, expires(), body.clone(), None).await.unwrap().unwrap();
    let newer = state
        .snapshot_capture(
            Scope::History,
            None,
            None,
            expires(),
            b"{\"records\":[],\"telemetry\":[{\"stream\":\"query.updated\"}]}".to_vec(),
            None,
        )
        .await
        .unwrap()
        .unwrap();
    assert_ne!(cut.contracts, newer.contracts);
    assert_eq!(state.snapshot_contracts(cut.clone(), 4096).await.unwrap(), body);
    assert!(state.snapshot_contracts(cut.clone(), 8).await.is_err());
    state.close().await.unwrap();
    drop(state);
    let state = store(root.path()).await;
    assert_eq!(state.snapshot_contracts(cut.clone(), 4096).await.unwrap(), body);
    let identity = cut.contracts.clone();
    state.call(move |connection| connection.execute("DELETE FROM query_contract_cuts WHERE identity=?1", [identity])).await.unwrap();
    assert!(!state.snapshot_available(cut).await.unwrap());
    state.close().await.unwrap();
}
