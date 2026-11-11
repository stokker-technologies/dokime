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
