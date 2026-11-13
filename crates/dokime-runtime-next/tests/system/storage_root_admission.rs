/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Exclusive Evidence Root Admission

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_runtime_next::{Error, Host};

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
async fn hosts_refuse_occupied_roots_before_opening_evidence() -> Result<(), Box<dyn std::error::Error>> {
    // Refuse An External Lease Before Creating The Evidence Database
    let root = tempfile::tempdir()?;
    let lease = std::fs::OpenOptions::new().create(true).truncate(false).read(true).write(true).open(root.path().join("local.lock"))?;
    lease.try_lock()?;
    let refused = Host::open(root.path(), super::support::settings());
    assert!(matches!(refused, Err(Error::OccupiedRoot(path)) if path == root.path()));
    assert!(!root.path().join("execution.sqlite3").exists());
    drop(lease);

    // Keep The Lease While A Running Host Owns The Root
    let host = Host::open(root.path(), super::support::settings())?;
    let retained = host.clone();
    assert!(matches!(Host::open(root.path(), super::support::settings()), Err(Error::OccupiedRoot(path)) if path == root.path()));

    // Release The Root At Shutdown Although A Handle Remains, And Refuse New Work Through It
    host.shutdown().await?;
    let reopened = Host::open(root.path(), super::support::settings())?;
    assert!(reopened.executions()?.is_empty());
    assert!(retained.add_remote("http://127.0.0.1:1").await.is_err());
    assert!(retained.start_cycle(Vec::new()).await.is_err());
    reopened.shutdown().await?;

    // Release The Root With Its Last Owner When A Host Is Dropped Without Shutdown
    drop((host, retained, reopened));
    let abandoned = Host::open(root.path(), super::support::settings())?;
    let second = abandoned.clone();
    drop(abandoned);
    assert!(matches!(Host::open(root.path(), super::support::settings()), Err(Error::OccupiedRoot(_))));
    drop(second);
    let host = Host::open(root.path(), super::support::settings())?;
    host.shutdown().await?;
    Ok(())
}
