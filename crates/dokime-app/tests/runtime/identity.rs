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
use std::{path::Path, sync::Arc, time::Duration};

// Third Party Imports
use dokime::core::{DokimeNodeIdentity, Identifier, Metadata};
use tokio::sync::Barrier;

// Local Imports
use dokime_app::{Runtime, RuntimeConfig};

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
    RuntimeConfig::new(
        root,
        Metadata::new(
            Identifier::new("fixture.identity").unwrap(),
            "Identity",
            None,
        ),
    )
    .with_application_root(application)
}

/* === Tests === */

pub(crate) async fn concurrent_creators_identity() {
    let root = tempfile::tempdir().unwrap();
    let application = root.path().join("application");
    let barrier = Arc::new(Barrier::new(8));
    let tasks = (0..8)
        .map(|index| {
            let config = config(&root.path().join(format!("runtime-{index}")), &application);
            let barrier = Arc::clone(&barrier);
            tokio::spawn(async move {
                barrier.wait().await;
                let mut runtime = Runtime::start(config).await.unwrap();
                let identity = runtime.handle().identity().node;
                runtime.shutdown().await.unwrap();
                identity
            })
        })
        .collect::<Vec<_>>();

    let identities = tokio::time::timeout(
        Duration::from_secs(15),
        futures_util::future::join_all(tasks),
    )
    .await
    .unwrap()
    .into_iter()
    .map(Result::unwrap)
    .collect::<Vec<_>>();
    assert!(identities.iter().all(|node| node == &identities[0]));
}

pub(crate) async fn existing_identity_republishing() {
    let root = tempfile::tempdir().unwrap();
    let node = DokimeNodeIdentity::transient(None);
    let bytes = serde_json::to_vec(&serde_json::json!({ "identity": node })).unwrap();
    let path = root.path().join("settings.json");
    std::fs::write(&path, &bytes).unwrap();

    let mut runtime = Runtime::start(config(&root.path().join("data"), root.path()))
        .await
        .unwrap();
    assert_eq!(runtime.handle().identity().node, node);
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    runtime.shutdown().await.unwrap();
}

pub(crate) async fn corruption_refuses_identity() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("settings.json");
    std::fs::write(&path, b"broken").unwrap();
    assert!(
        Runtime::start(config(&root.path().join("data"), root.path()))
            .await
            .is_err()
    );
    assert_eq!(std::fs::read(&path).unwrap(), b"broken");
}
