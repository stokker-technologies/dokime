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
use dokime_operations::Error as OperationError;
use dokime_protocol::model::{ArtifactMetadata, ArtifactScope, Import, Origin, origin::Kind};
use dokime_runtime::Error;
use dokime_runtime::Host;
use dokime_storage::{Execution, ExecutionState, Store};
use std::error::Error as ErrorError;
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
fn incompatible_content_roots_are_refused_without_resetting_evidence() -> Result<(), Box<dyn ErrorError>> {
    super::support::run(Box::pin(async {
        let root = super::support::tempdir()?;
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
        store.begin_artifact(ArtifactMetadata {
            artifact_id: "legacy".into(),
            filename: "evidence.bin".into(),
            scope: Some(ArtifactScope {
                execution_id: Some(identifier.clone()),
                ..Default::default()
            }),
            ..Default::default()
        })?;
        drop(store);
        let refused = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await;
        assert!(matches!(refused, Err(Error::Operation(OperationError::Precondition(_)))));
        let store = Store::open(root.path())?;
        assert_eq!(store.cut(&identifier)?.state, ExecutionState::Open);
        assert!(store.has_incompatible_content()?);
        Ok(())
    }))
}

#[test]
fn hosts_refuse_occupied_roots_before_opening_evidence() -> Result<(), Box<dyn ErrorError>> {
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
        assert!(retained.add_remote("http://127.0.0.1:1").await.is_err());
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
