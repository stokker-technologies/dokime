/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Save Round Trip And Refusal Boundaries

use std::{net::Ipv4Addr, path::PathBuf, sync::Arc};

use super::fixture::{
    Gate, Listener, Service,
    delivery::{DownloadConfig, bytes},
};
use dokime_gui::connection::{ConnectionError, ConnectionSession};
use dokime_gui::testing::{ArtifactSource, DeliveryFailure, DeliveryStage, move_verified_file, save_verified};

#[tokio::test]
async fn delivery() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("evidence.bin");
    let source = || ArtifactSource::Artifact { artifact_id: "fixture-artifact".into() };
    let service = Service::new("native-save");
    let download = service.download.clone();
    let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
    let session = ConnectionSession::default();
    let connected = session.connect(server.local()).await.unwrap();
    let generation = connected.generation;
    let save = |picked: Option<(PathBuf, bool)>| {
        save_verified(session.clone(), generation.clone(), source(), "evidence.bin".into(), async move { Ok(picked) })
    };

    // A Cancelled Picker Performs No Content Request And Creates No Stage.
    let before = download.lock().unwrap().calls;
    assert!(save(None).await.unwrap().is_none());
    assert_eq!(download.lock().unwrap().calls, before);
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);

    // Verified Frames Replace Only The Consented Destination.
    std::fs::write(&target, b"original").unwrap();
    let delivered = save(Some((target.clone(), true))).await.unwrap().unwrap();
    assert_eq!(PathBuf::from(delivered.path), target);
    assert_eq!(std::fs::read(&target).unwrap(), bytes());
    assert_eq!(download.lock().unwrap().calls, before + 1);
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);

    // A Destination Appearing After Selection Must Not Be Replaced Without Consent.
    let error = save(Some((target.clone(), false))).await.unwrap_err();
    assert_eq!(error.stage, DeliveryStage::Rename);
    assert_eq!(std::fs::read(&target).unwrap(), bytes());
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);

    // Digest Failure Cannot Publish Or Retain A Partial Stage.
    download.lock().unwrap().corrupt = true;
    let error = save(Some((target.clone(), true))).await.unwrap_err();
    assert_eq!(error.stage, DeliveryStage::Stream);
    assert_eq!(std::fs::read(&target).unwrap(), bytes());
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
    download.lock().unwrap().corrupt = false;

    // Disconnect During The Real Stream Before Its Completion Frame.
    let gate = Arc::new(Gate::new());
    download.lock().unwrap().gate = Some(gate.clone());
    let pending = tokio::spawn(save(Some((target.clone(), true))));
    gate.entered.notified().await;
    assert_eq!(std::fs::read(&target).unwrap(), bytes());
    session.disconnect().unwrap();
    gate.release.add_permits(1);
    let error = pending.await.unwrap().unwrap_err();
    assert!(matches!(error.failure, DeliveryFailure::Rpc(dokime_gui::Error::Session(ConnectionError::Superseded))));
    assert_eq!(std::fs::read(&target).unwrap(), bytes());
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);

    // Final Publication Rechecks Ownership Even After The Caller Has Verified Content.
    let staged = root.path().join(".stale.tmp");
    std::fs::write(&staged, bytes()).unwrap();
    assert!(move_verified_file(session.clone(), generation, staged.clone(), target.clone(), true).await.is_err());
    assert!(!staged.exists());
    assert_eq!(std::fs::read(&target).unwrap(), bytes());
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);

    // The New Generation Can Save A Fresh File After Every Refusal.
    *download.lock().unwrap() = DownloadConfig::default();
    let generation = session.connect(server.local()).await.unwrap().generation;
    let fresh = root.path().join("fresh.bin");
    let picked = fresh.clone();
    let result = save_verified(session.clone(), generation, source(), "fresh.bin".into(), async move { Ok(Some((picked, false))) })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(PathBuf::from(result.path), fresh);
    assert_eq!(std::fs::read(&fresh).unwrap(), bytes());
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 2);
    session.disconnect().unwrap();
}
