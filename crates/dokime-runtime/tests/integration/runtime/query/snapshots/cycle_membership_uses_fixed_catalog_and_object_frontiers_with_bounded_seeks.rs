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
