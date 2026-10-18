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
use dokime_runtime::storage::sqlite::*;

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
async fn source_order_restart() {
    let root = tempfile::tempdir().unwrap();
    let state = State::open(root.path()).await.unwrap();
    state.register_source("fabric", "microsoft.fabric").await.unwrap();
    state.reorder_sources(vec!["fabric".into(), "local".into()]).await.unwrap();
    let order = state.sources().await.unwrap();
    assert_eq!(order[0].identifier, "fabric");
    assert_eq!(order[1].priority, 1);
    assert!(state.reorder_sources(vec!["local".into(), "local".into()]).await.is_err());
    assert!(state.reorder_sources(vec!["local".into()]).await.is_err());
    assert_eq!(state.sources().await.unwrap(), order);
    assert!(state.register_source("fabric", "another.adapter").await.is_err());
    assert!(state.set_source_enabled("local", false).await.is_err());
    drop(state);
    let reopened = State::open(root.path()).await.unwrap();
    assert_eq!(reopened.sources().await.unwrap(), order);
    reopened.reorder_sources(vec!["local".into(), "fabric".into()]).await.unwrap();
    assert_eq!(reopened.sources().await.unwrap()[0].identifier, "local");
}
