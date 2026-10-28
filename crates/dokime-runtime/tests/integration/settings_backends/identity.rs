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
use std::{path::Path, sync::Arc};

// Third Party Imports
use dokime_runtime::{Identifier, Metadata};
use tokio::sync::Barrier;

// Local Imports
use dokime_runtime::{Runtime, RuntimeConfig};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn config(root: &Path, application: &Path) -> RuntimeConfig {
    crate::service_support::budgets::runtime_config(root, Metadata::new(Identifier::new("fixture.identity").unwrap(), "Identity", None))
        .with_application_root(application)
}

/* === Tests === */

pub(crate) async fn concurrent_creators_identity() {
    let root = tempfile::tempdir().unwrap();
    let application = root.path().join("application");
    let barrier = Arc::new(Barrier::new(8));
    let database = dokime_runtime::storage::sqlite::State::open(&root.path().join("data")).await.unwrap();
    let tasks = (0..8)
        .map(|_| {
            let database = database.clone();
            let application = application.clone();
            let barrier = Arc::clone(&barrier);
            tokio::spawn(async move {
                barrier.wait().await;
                let settings = dokime_runtime::config::Settings::open(application, database).await.unwrap();
                settings.read(|metadata| metadata.identity.clone()).await.unwrap()
            })
        })
        .collect::<Vec<_>>();

    let identities = futures_util::future::join_all(tasks).await.into_iter().map(Result::unwrap).collect::<Vec<_>>();
    assert!(identities.iter().all(|node| node == &identities[0]));
    database.close().await.unwrap();
}

pub(crate) async fn existing_identity_republishing() {
    let root = tempfile::tempdir().unwrap();
    let configuration = config(&root.path().join("data"), root.path());
    let mut first = Runtime::start(configuration.clone()).await.unwrap();
    let node = first.handle().identity().node;
    first.shutdown().await.unwrap();
    let mut reopened = Runtime::start(configuration).await.unwrap();
    assert_eq!(reopened.handle().identity().node, node);
    assert!(!root.path().join("settings.json").exists());
    reopened.shutdown().await.unwrap();
}

pub(crate) async fn legacy_files_ignored() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("settings.json");
    std::fs::write(&path, b"broken").unwrap();
    let config_path = root.path().join("config.toml");
    std::fs::write(&config_path, b"legacy").unwrap();
    let mut runtime = Runtime::start(config(&root.path().join("data"), root.path())).await.unwrap();
    runtime.shutdown().await.unwrap();
    assert_eq!(std::fs::read(&config_path).unwrap(), b"legacy");
    assert_eq!(std::fs::read(&path).unwrap(), b"broken");
}
