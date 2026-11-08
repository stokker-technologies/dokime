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
    let resumed = tokens.decode(&token, "dokime.query.list_records").unwrap();
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
    assert!(matches!(error, dokime_runtime::storage::Error::QuerySnapshotUnavailable));
    state.close().await.unwrap();
}
