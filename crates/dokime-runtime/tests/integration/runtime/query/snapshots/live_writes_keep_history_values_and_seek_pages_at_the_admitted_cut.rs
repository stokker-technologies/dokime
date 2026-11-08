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
