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
async fn runtime_snapshot_pages_bind_selection_and_allow_size_changes_during_writes() {
    use dokime_domain::model::{
        identity::Metadata,
        query::{
            page::PageRequest,
            selection::{CycleScope, CycleSelection, RecordScope},
        },
    };
    use dokime_runtime::{Runtime, config::RuntimeConfig, transactions::CycleDefinition};
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
    let contract = dokime_runtime::records::builtin_record_contracts().into_iter().next().unwrap();
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
