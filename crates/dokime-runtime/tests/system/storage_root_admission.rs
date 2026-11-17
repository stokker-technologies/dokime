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
use dokime_protocol::model::{Import, Origin, origin::Kind};
use dokime_runtime::Error;
use dokime_runtime::Host;
use dokime_storage::{Execution, Store};
use rusqlite::Connection;
use std::error;
use std::fs::OpenOptions;

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
fn incompatible_content_roots_are_refused_without_resetting_evidence() -> Result<(), Box<dyn error::Error>> {
    super::support::run(Box::pin(async {
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        let execution = Execution {
            identifier: "evidence".into(),
            cycle: "cycle".into(),
            origin: Some(Origin {
                kind: Some(Kind::Import(Import::default())),
            }),
            ..Default::default()
        };
        let identifier = execution.identifier.clone();
        store.create(&execution)?;
        let database = Connection::open(root.path().join("execution.sqlite3"))?;
        database.execute_batch("CREATE TABLE content_objects(id TEXT PRIMARY KEY); INSERT INTO content_objects VALUES('retired-file')")?;
        drop(database);
        drop(store);
        let refused = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await;
        assert!(matches!(refused, Err(Error::Storage(dokime_storage::Error::Incompatible(_)))));
        let database = Connection::open(root.path().join("execution.sqlite3"))?;
        assert_eq!(database.query_row("SELECT state FROM executions WHERE id=?1", [identifier], |row| row.get::<_, i32>(0))?, 0);
        assert_eq!(database.query_row("SELECT COUNT(*) FROM content_objects", [], |row| row.get::<_, i64>(0))?, 1);
        Ok(())
    }))
}

#[test]
fn hosts_refuse_occupied_roots_before_opening_evidence() -> Result<(), Box<dyn error::Error>> {
    super::support::run(Box::pin(async {
        // Refuse An External Lease Before Creating The Evidence Database
        let root = super::support::tempdir()?;
        let lease = OpenOptions::new().create(true).truncate(false).read(true).write(true).open(root.path().join("local.lock"))?;
        lease.try_lock()?;
        let refused = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await;
        assert!(matches!(refused, Err(Error::OccupiedRoot(path)) if path == root.path()));
        assert!(!root.path().join("execution.sqlite3").exists());
        drop(lease);

        // Keep The Lease While A Running Host Owns The Root
        let host = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
        let retained = host.clone();
        assert!(
            matches!(Host::open(root.path(), super::support::settings(), super::support::service_settings()).await, Err(Error::OccupiedRoot(path)) if path == root.path())
        );

        // Release The Root At Shutdown Although A Handle Remains, And Refuse New Work Through It
        host.shutdown().await?;
        let reopened = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
        assert!(reopened.executions()?.is_empty());
        assert!(retained.add_remote("unavailable", "http://127.0.0.1:1").await.is_err());
        assert!(retained.start_cycle(Vec::new()).await.is_err());
        reopened.shutdown().await?;

        // Release The Root With Its Last Owner When A Host Is Dropped Without Shutdown
        drop((host, retained, reopened));
        let abandoned = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
        let second = abandoned.clone();
        drop(abandoned);
        assert!(matches!(
            Host::open(root.path(), super::support::settings(), super::support::service_settings()).await,
            Err(Error::OccupiedRoot(_))
        ));
        drop(second);
        let host = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
        host.shutdown().await?;
        Ok(())
    }))
}

#[test]
fn superseded_plan_tables_are_refused_without_resetting_the_root() -> Result<(), Box<dyn error::Error>> {
    super::support::run(Box::pin(async {
        for table in ["cases", "registrations"] {
            // Preserve the Incompatible Journal Alongside Existing Execution Evidence.
            let root = tempfile::tempdir()?;
            let store = Store::open(root.path())?;
            store.create(&Execution {
                identifier: "retained-execution".into(),
                cycle: "cycle".into(),
                origin: Some(Origin {
                    kind: Some(Kind::Import(Import::default())),
                }),
                ..Default::default()
            })?;
            drop(store);
            let database = Connection::open(root.path().join("execution.sqlite3"))?;
            database.execute_batch(&format!("CREATE TABLE {table}(payload BLOB); INSERT INTO {table} VALUES(X'010203')"))?;
            drop(database);

            // Admission Refuses the Old Shape Without Migrating or Deleting It.
            let refused = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await;
            assert!(matches!(refused, Err(Error::Storage(dokime_storage::Error::Incompatible(_)))));
            let database = Connection::open(root.path().join("execution.sqlite3"))?;
            let execution: String = database.query_row("SELECT id FROM executions", [], |row| row.get(0))?;
            assert_eq!(execution, "retained-execution");
            let payload: Vec<u8> = database.query_row(&format!("SELECT payload FROM {table}"), [], |row| row.get(0))?;
            assert_eq!(payload, vec![1, 2, 3]);
        }
        Ok(())
    }))
}
