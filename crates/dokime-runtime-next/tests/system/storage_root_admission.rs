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

    // Keep The Lease While Any Host Owner Remains
    let host = Host::open(root.path(), super::support::settings())?;
    let retained = host.clone();
    assert!(matches!(Host::open(root.path(), super::support::settings()), Err(Error::OccupiedRoot(path)) if path == root.path()));
    host.shutdown().await?;
    drop(host);
    assert!(matches!(Host::open(root.path(), super::support::settings()), Err(Error::OccupiedRoot(_))));
    drop(retained);

    // Reopen The Same Evidence After All Owners Have Settled
    let host = Host::open(root.path(), super::support::settings())?;
    assert!(host.executions()?.is_empty());
    host.shutdown().await?;
    Ok(())
}
