/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Saved Reports Retain Exact Bytes and Refuse Tampering After Restart

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_protocol::v1;
use dokime_storage::{Result, Store};

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

#[test]
fn issued_positions_and_original_append_receipts_survive_restart() -> Result<()> {
    // Create Two Histories and Append One Idempotent Record.
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    for id in ["first", "second"] {
        store.create(&dokime_storage::Execution {
            identifier: id.into(),
            cycle: "cycle".into(),
            origin: Some(v1::Origin {
                kind: Some(v1::origin::Kind::Import(v1::Import::default())),
            }),
            ..Default::default()
        })?;
    }
    let evidence = dokime_storage::Evidence {
        kind: "log".into(),
        payload: vec![7],
        ..Default::default()
    };
    let original = store.append_requested("first", "append", "request", &[1], evidence.clone())?;
    let cut = store.evidence_position(None)?;
    assert_eq!(store.append_requested("first", "append", "request", &[1], evidence.clone())?, original);
    assert!(store.append_requested("first", "append", "request", &[2], evidence).is_err());
    let mut future = cut.clone();
    future.sequence += 1;
    assert!(store.evidence_position(Some(&future)).is_err());
    drop(store);

    // Reopen and Follow a Different Execution Without Reinterpreting the Cut.
    let store = Store::open(root.path())?;
    assert_eq!(store.evidence_position(Some(&cut))?, cut);
    store.append(
        "second",
        dokime_storage::Evidence {
            kind: "log".into(),
            payload: vec![9],
            ..Default::default()
        },
    )?;
    let current = store.evidence_position(None)?;
    assert!(current.sequence > cut.sequence);
    assert_eq!(store.evidence_entries(0, cut.sequence, std::num::NonZeroUsize::new(10).unwrap(), false)?.len(), 1);

    // Deleting Sealed Evidence Explicitly Invalidates a Returned Global Cut.
    store.close("first", v1::Termination::Completed, v1::ExecutionOutcome::default())?;
    store.seal("first")?;
    let seal = store.cut("first")?.seal;
    store.delete_sealed_execution("delete", "first", &seal, &v1::ExecutionsDeleteResponse::default())?;
    assert!(store.evidence_position(Some(&cut)).is_err());
    let replacement = store.evidence_position(None)?;
    assert_ne!(replacement.history, cut.history);
    Ok(())
}
