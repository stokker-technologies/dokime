/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Desktop Host Connection Journeys

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{num::NonZeroUsize, sync::Arc, time::Duration};

// Third Party Imports

// Local Imports
use super::*;
use dokime_runtime_next::{ControlledClock, PlacementSettings};
use dokime_sdk::client::v1::{self as library, access};
use dokime_sdk::v1;
use tokio_stream::StreamExt;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn settings() -> ConnectionSettings {
    ConnectionSettings {
        host: HostSettings {
            clock: Arc::new(ControlledClock::new(0)),
            placement: PlacementSettings {
                connect_timeout: Duration::from_secs(30),
                reconnect_delay: Duration::from_millis(25),
                stop_grace: Duration::from_secs(30),
                stream_window: NonZeroUsize::new(8).unwrap(),
                message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
            },
        },
        services: services::ClientSettings {
            read_timeout: Duration::from_secs(30),
            mutation_timeout: Duration::from_secs(30),
            watch_timeout: Duration::from_secs(3600),
            max_cases: 1000,
            page_items: 1000,
            message_bytes: 4 * 1024 * 1024,
            stream_buffer: 8,
        },
        client: ClientSettings {
            retained_bytes: NonZeroUsize::new(64 * 1024 * 1024).unwrap(),
            message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
            input_items: NonZeroUsize::new(8).unwrap(),
            input_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        },
        connect_timeout: Duration::from_secs(30),
    }
}

/* === Tests === */

#[tokio::test]
async fn embedded_readiness_attachment_and_restart() -> Result<(), Box<dyn std::error::Error>> {
    // Start the Embedded Host and Read Its Ready SDK Identity
    let root = tempfile::tempdir()?;
    let mut owner = Connection::new(settings())?;
    let session = owner.start(root.path()).await?;
    assert!(owner.is_embedded());
    let identity = session.identity().status.as_ref().unwrap().identity.as_ref().unwrap().host_id.clone();
    let epoch = session.identity().resolved_at.as_ref().unwrap().history.as_ref().unwrap().epoch.clone();
    assert!(session.identity().status.as_ref().unwrap().available);

    // An Attached Desktop Observes the Same Host and Does Not Own Its Lifetime
    let mut attached = Connection::new(settings())?;
    let second = attached.attach_local(session.endpoint()).await?;
    assert!(!attached.is_embedded());
    assert_eq!(second.identity().status, session.identity().status);
    attached.close().await?;
    assert!(second.cancellation().is_cancelled());
    let request = session.request(library::StatusGetRequest {})?;
    session.execute(session.client().status.get(request)).await?;

    // Closing Invalidates Every Lease Before Joining the Host
    owner.close().await?;
    assert!(session.cancellation().is_cancelled());
    assert!(matches!(owner.session(), Err(Error::Disconnected)));
    assert!(matches!(session.execute(async { Ok(()) }).await, Err(Error::Superseded)));

    // Restart Retains the Store Identity and Creates a New Watch History
    let reopened = owner.start(root.path()).await?;
    assert_eq!(reopened.identity().status.as_ref().unwrap().identity.as_ref().unwrap().host_id, identity);
    assert_ne!(reopened.identity().resolved_at.as_ref().unwrap().history.as_ref().unwrap().epoch, epoch);
    owner.close().await?;
    Ok(())
}

#[tokio::test]
async fn incompatible_storage_is_preserved_and_retry_uses_an_explicit_fresh_root() -> Result<(), Box<dyn std::error::Error>> {
    // A Foreign Root Must Stay Byte-for-Byte Available for Backup
    let existing = tempfile::tempdir()?;
    let original = b"operator evidence retained for backup";
    let evidence = existing.path().join("existing-evidence.bin");
    std::fs::write(&evidence, original)?;
    let mut owner = Connection::new(settings())?;
    let error = owner.start(existing.path()).await.err().ok_or("Start Host: Expected Incompatible Storage Refusal")?;
    assert!(matches!(error, Error::Host(dokime_runtime_next::Error::Storage(_))));
    assert_eq!(std::fs::read(&evidence)?, original);
    assert_eq!(std::fs::read_dir(existing.path())?.count(), 1);
    assert!(!owner.is_embedded());

    // The Operator Can Select Another Root Without Touching the Original
    let fresh = tempfile::tempdir()?;
    owner.start(fresh.path()).await?;
    assert_eq!(std::fs::read(&evidence)?, original);
    owner.close().await?;
    Ok(())
}

#[tokio::test]
async fn sdk_calls_and_watches_share_the_owned_connection() -> Result<(), Box<dyn std::error::Error>> {
    // Observe Current State Through the SDK Watch Transport
    let root = tempfile::tempdir()?;
    let mut owner = Connection::new(settings())?;
    let session = owner.start(root.path()).await?;
    let request = v1::ObservationWatchRuntimeRequest::default().encode_to_vec();
    let mut watch = session.watch_raw("/dokime.v1.Observation/WatchRuntime", &request).await?;
    let opening = watch.next().await.ok_or("Watch Runtime: Missing Initial Frame")??;
    let opening = v1::ObservationWatchRuntimeOutput::decode(opening.as_slice())?;
    let Some(v1::observation_watch_runtime_output::Value::Item(opening)) = opening.value else {
        return Err("Watch Runtime: Expected Initial State".into());
    };
    assert!(opening.resynchronized);

    // A Mutation and Its Read Use Canonical Generated Messages on the Same SDK Root
    let request = v1::ExecutionCreateCycleRequest {
        metadata: Some(v1::Metadata {
            label: "Desktop Native Journey".into(),
            ..Default::default()
        }),
        ..Default::default()
    }
    .encode_to_vec();
    let response = session.call_raw("/dokime.v1.Execution/CreateCycle", &request).await?;
    let cycle = v1::CreateCycleReply::decode(response.as_slice())?;
    assert!(!cycle.cycle_id.is_empty());
    loop {
        let frame = watch.next().await.ok_or("Watch Runtime: Closed Before Cycle Admission")??;
        let frame = v1::ObservationWatchRuntimeOutput::decode(frame.as_slice())?;
        let Some(v1::observation_watch_runtime_output::Value::Item(frame)) = frame.value else {
            return Err("Watch Runtime: Finished Before Cycle Admission".into());
        };
        if frame.cycle_count == 1 {
            break;
        }
    }
    let request = v1::ExecutionGetCycleRequest { cycle_id: cycle.cycle_id.clone() }.encode_to_vec();
    let response = session.call_raw("/dokime.v1.Execution/GetCycle", &request).await?;
    assert_eq!(v1::CycleSummary::decode(response.as_slice())?.cycle_id, cycle.cycle_id);

    // Unknown Routes Fail Explicitly and Shutdown Joins the Idle Watch
    let error = session.call_raw("/dokime.v1.Unknown/Get", &[]).await.unwrap_err();
    assert!(matches!(error, Error::Rpc(status) if status.code() == tonic::Code::Unimplemented));
    owner.close().await?;
    while let Some(frame) = watch.next().await {
        frame?;
    }
    assert!(session.watches.lock().await.is_empty());
    Ok(())
}

#[tokio::test]
async fn pinned_pairing_identity_and_certificate_rotation() -> Result<(), Box<dyn std::error::Error>> {
    // Enable an Isolated Authenticated Listener Through the SDK Root
    let root = tempfile::tempdir()?;
    let mut owner = Connection::new(settings())?;
    let local = owner.start(root.path()).await?;
    let request = local.request(access::ListenerGetRequest {})?;
    let current = local.execute(local.client().access.listener.get(request)).await?.into_inner();
    let mut value = current.value.ok_or("Get Listener: Missing Settings")?;
    value.enabled = true;
    value.endpoint = "127.0.0.1:0".into();
    value.certificate_names = vec!["localhost".into(), "127.0.0.1".into()];
    let request = local.request(access::ListenerSetRequest {
        value: Some(value),
        expected_revision: current.revision,
    })?;
    local.execute(local.client().access.listener.set(request)).await?;

    // Export and Reopen the Pairing File Without Exposing Its Credential in Debug
    let request = local.request(access::PairingsCreateRequest { label: "Desktop Story".into() })?;
    let created = local.execute(local.client().access.pairings.create(request)).await?.into_inner();
    let identity = local.identity().status.as_ref().unwrap().identity.as_ref().unwrap().host_id.clone();
    let pairing = Pairing {
        version: 1,
        backend_identifier: identity.clone(),
        label: "Desktop Story".into(),
        endpoints: vec![local.endpoint().replacen("http://", "https://", 1), created.endpoint],
        certificate_pem: created.pinned_certificate_pem,
        access_key: created.credential,
    };
    assert!(!format!("{pairing:?}").contains(&pairing.access_key));
    let path = root.path().join("desktop.pair");
    std::fs::write(&path, serde_json::to_vec(&pairing)?)?;
    let mut remote = Connection::new(settings())?;
    let connected = remote.attach_remote(Pairing::read(&path)?).await?;
    assert_eq!(connected.identity().status.as_ref().unwrap().identity.as_ref().unwrap().host_id, identity);

    // A Correct Certificate Cannot Override the Expected Host Identity
    let mut changed_identity = Pairing::read(&path)?;
    changed_identity.backend_identifier = "another-host".into();
    assert!(matches!(remote.attach_remote(changed_identity).await, Err(Error::IdentityChanged)));
    assert!(connected.cancellation().is_cancelled());

    // Rotation Refuses the Old Pin; Fresh Pairing Reconnects Without Restarting the Host
    let request = local.request(access::CertificatesCreateRequest {})?;
    local.execute(local.client().access.certificates.create(request)).await?;
    assert!(remote.attach_remote(Pairing::read(&path)?).await.is_err());
    let request = local.request(access::PairingsCreateRequest { label: "Rotated Desktop Story".into() })?;
    let rotated = local.execute(local.client().access.pairings.create(request)).await?.into_inner();
    assert_ne!(rotated.pinned_certificate_pem, pairing.certificate_pem);
    let replacement = Pairing {
        version: 1,
        backend_identifier: identity,
        label: "Rotated Desktop Story".into(),
        endpoints: vec![rotated.endpoint],
        certificate_pem: rotated.pinned_certificate_pem,
        access_key: rotated.credential,
    };
    remote.attach_remote(replacement).await?;
    remote.close().await?;
    let request = local.request(library::StatusGetRequest {})?;
    local.execute(local.client().status.get(request)).await?;
    owner.close().await?;
    Ok(())
}
