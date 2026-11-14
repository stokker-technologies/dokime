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
use std::{
    error::Error as StdError,
    ffi::OsString,
    num::NonZeroUsize,
    panic::{AssertUnwindSafe, resume_unwind},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

// Third Party Imports
use futures_util::FutureExt;

// Local Imports
use prost::Message;
use tokio_stream::StreamExt;
use tonic::Code;

use dokime_gui::connection::ConnectionError;
use dokime_gui::connection::host::{Connection, ConnectionSettings, Error, Pairing};
use dokime_protocol::{client, v1};
use dokime_runtime_next::{ControlledClock, HostSettings, PlacementSettings, services};
use dokime_sdk::client::{
    ClientSettings, StorageTransferSettings,
    v1::{self as library, access},
};

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
            storage: dokime_runtime_next::storage_defaults(),
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
            transfer: StorageTransferSettings { chunk_bytes: Some(64 * 1024) },
            retained_bytes: NonZeroUsize::new(64 * 1024 * 1024).unwrap(),
            message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
            input_items: NonZeroUsize::new(8).unwrap(),
            input_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        },
        connect_timeout: Duration::from_secs(30),
    }
}

/* === Tests === */

#[test]
fn embedded_readiness_attachment_and_restart() -> Result<(), Box<dyn StdError>> {
    dokime_gui::test_support::run(async {
        // Start the Embedded Host and Read Its Ready SDK Identity
        let root = dokime_gui::test_support::tempdir()?;
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
    })
}

#[test]
fn incompatible_storage_is_preserved_and_retry_uses_an_explicit_fresh_root() -> Result<(), Box<dyn StdError>> {
    dokime_gui::test_support::run(async {
        // A Foreign Root Must Stay Byte-for-Byte Available for Backup
        let existing = dokime_gui::test_support::tempdir()?;
        let original = b"operator evidence retained for backup";
        let evidence = existing.path().join("existing-evidence.bin");
        std::fs::write(&evidence, original)?;
        let mut owner = Connection::new(settings())?;
        let error = owner.start(existing.path()).await.err().ok_or("Start Host: Expected Incompatible Storage Refusal")?;
        assert!(matches!(error, Error::Host(dokime_runtime_next::Error::Storage(_))));
        assert_eq!(std::fs::read(&evidence)?, original);
        let mut entries =
            std::fs::read_dir(existing.path())?.map(|entry| entry.map(|entry| entry.file_name())).collect::<Result<Vec<_>, _>>()?;
        entries.sort();
        assert_eq!(entries, vec![OsString::from("existing-evidence.bin"), OsString::from("local.lock")]);
        assert!(!owner.is_embedded());

        // The Operator Can Select Another Root Without Touching the Original
        let fresh = dokime_gui::test_support::tempdir()?;
        owner.start(fresh.path()).await?;
        assert_eq!(std::fs::read(&evidence)?, original);
        owner.close().await?;
        Ok(())
    })
}

#[test]
fn sdk_calls_and_watches_share_the_owned_connection() -> Result<(), Box<dyn StdError>> {
    dokime_gui::test_support::run(async {
        // Observe Current State Through the SDK Watch Transport
        let root = dokime_gui::test_support::tempdir()?;
        let mut owner = Connection::new(settings())?;
        let session = owner.start(root.path()).await?;
        let active = dokime_gui::connection::ConnectionSession::default();
        let discovered = active
            .connect(dokime_gui::connection::ConnectRequest {
                target: dokime_gui::connection::ConnectionTarget::Local { endpoint: session.endpoint().to_owned() },
                expected_backend_id: None,
            })
            .await
            .map_err(|error| format!("{error:?}"))?;
        assert!(discovered.backend.unwrap().methods.iter().any(|method| method == "/harmos.v1.access.Listener/Get"));
        active.disconnect().map_err(|error| format!("{error:?}"))?;
        let request = client::backend::OverviewWatchRequest::default().encode_to_vec();
        let mut watch = session.watch_raw("/dokime.client.v1.backend.Overview/Watch", &request).await?;
        let opening = watch.next().await.ok_or("Watch Runtime: Missing Initial Frame")??;
        let opening = client::backend::OverviewWatchResponse::decode(opening.as_slice())?;
        assert!(opening.opening);

        // A Mutation and Its Read Use Canonical Generated Messages on the Same SDK Root
        let request = client::CyclesCreateRequest {
            request_id: "desktop-native-cycle".into(),
            metadata: Some(v1::Metadata {
                label: "Desktop Native Journey".into(),
                ..Default::default()
            }),
            ..Default::default()
        }
        .encode_to_vec();
        let response = session.call_raw("/dokime.client.v1.Cycles/Create", &request).await?;
        let cycle = client::CyclesCreateResponse::decode(response.as_slice())?.cycle.ok_or("Create Cycle: Missing Value")?;
        assert!(!cycle.cycle_id.is_empty());
        loop {
            let frame = watch.next().await.ok_or("Watch Runtime: Closed Before Cycle Admission")??;
            let frame = client::backend::OverviewWatchResponse::decode(frame.as_slice())?;
            if frame.overview.is_some_and(|overview| overview.cycle_summaries.len() == 1) {
                break;
            }
        }
        let request = client::CyclesGetRequest {
            name: cycle.cycle_id.clone(),
            ..Default::default()
        }
        .encode_to_vec();
        let response = session.call_raw("/dokime.client.v1.Cycles/Get", &request).await?;
        assert_eq!(client::CyclesGetResponse::decode(response.as_slice())?.cycle.unwrap().cycle_id, cycle.cycle_id);

        // Read Traceability Through Its Revisioned Resource and Retain the Scoped Cut
        let scope = v1::EvidenceScope {
            cycle_id: Some(cycle.cycle_id.clone()),
            ..Default::default()
        };
        let request = client::evidence::TraceabilityGetRequest {
            scope: Some(scope.clone()),
            ..Default::default()
        }
        .encode_to_vec();
        let response = session.call_raw("/dokime.client.v1.evidence.Traceability/Get", &request).await?;
        let response = client::evidence::TraceabilityGetResponse::decode(response.as_slice())?;
        assert!(response.snapshots.is_empty());
        assert!(response.next_page_token.is_empty());
        let cut = response.resolved_at.ok_or("Get Traceability: Missing Read Cut")?;
        assert!(cut.positions.is_empty());
        assert!(cut.revisions.is_empty());

        // Watch Traceability on the Same Native Connection Until Owned Shutdown
        let request = client::evidence::TraceabilityWatchRequest {
            scope: Some(scope),
            at: Some(cut.clone()),
            ..Default::default()
        }
        .encode_to_vec();
        let mut traceability = session.watch_raw("/dokime.client.v1.evidence.Traceability/Watch", &request).await?;
        let opening = traceability.next().await.ok_or("Watch Traceability: Missing Opening")??;
        let opening = client::evidence::TraceabilityWatchResponse::decode(opening.as_slice())?;
        assert!(opening.opening);
        assert!(opening.snapshots.is_empty());
        assert_eq!(opening.resolved_at.as_ref(), Some(&cut));

        // Unknown Routes Fail Explicitly and Shutdown Joins the Idle Watch
        let error = session.call_raw("/dokime.client.v1.Unknown/Get", &[]).await.unwrap_err();
        assert!(matches!(error, Error::Rpc(status) if status.code() == Code::Unimplemented));
        owner.close().await?;
        while let Some(frame) = watch.next().await {
            frame?;
        }
        while let Some(frame) = traceability.next().await {
            frame?;
        }
        assert!(matches!(session.execute(async { Ok(()) }).await, Err(Error::Superseded)));
        Ok(())
    })
}

#[test]
fn pinned_pairing_identity_and_certificate_rotation() -> Result<(), Box<dyn StdError>> {
    dokime_gui::test_support::run(async {
        // Enable an Isolated Authenticated Listener Through the SDK Root
        let root = dokime_gui::test_support::tempdir()?;
        let mut owner = Connection::new(settings())?;
        let local = owner.start(root.path()).await?;
        let result = AssertUnwindSafe(async {
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
            Ok::<_, Box<dyn StdError>>(())
        })
        .catch_unwind()
        .await;
        let closed = owner.close().await;
        let result = match result {
            Ok(result) => result,
            Err(panic) => resume_unwind(panic),
        };
        result?;
        closed?;
        Ok(())
    })
}

#[test]
fn desktop_owner_routes_canonical_ipc_and_replaces_generation() -> Result<(), Box<dyn StdError>> {
    dokime_gui::test_support::run(async {
        use dokime_gui::connection::ConnectionSession;
        use dokime_gui::native_session::{BackendOwnership, BackendSelection, DesktopSession, backend::Backend};

        // Start Through the Application's Actual Native Owner
        let root = dokime_gui::test_support::tempdir()?;
        let connection = ConnectionSession::default();
        let owner = DesktopSession::with_local_backend(connection.clone(), Backend::new(root.path().to_path_buf()));
        let ready = owner.select(BackendSelection::OwnedLocal).await.map_err(|error| format!("{error:?}"))?;
        assert_eq!(ready.ownership, BackendOwnership::OwnedLocal);
        let first = connection.lease(&ready.connection.generation).map_err(|error| format!("{error:?}"))?;
        let endpoint = ready.connection.endpoint.clone().ok_or("missing embedded endpoint")?;
        let generation = ready.connection.generation.parse::<u64>()?;

        // Dispatch the Same Descriptor-Fenced IPC Command Used by the WebView
        let selected = connection.clone();
        let response = tokio::task::spawn_blocking(move || {
            let app = tauri::test::mock_builder()
                .manage(selected)
                .manage(dokime_gui::rpc::Bridge::default())
                .invoke_handler(dokime_gui::rpc::handler())
                .build(tauri::test::mock_context(tauri::test::noop_assets()))
                .unwrap();
            let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();
            let mut bytes = generation.to_le_bytes().to_vec();
            bytes.extend(dokime_gui::rpc::descriptor_digest());
            bytes.extend(library::StatusGetRequest {}.encode_to_vec());
            let response = tauri::test::get_ipc_response(
                &window,
                tauri::webview::InvokeRequest {
                    cmd: "rpc".into(),
                    callback: tauri::ipc::CallbackFn(0),
                    error: tauri::ipc::CallbackFn(1),
                    url: if cfg!(target_os = "windows") { "http://tauri.localhost" } else { "tauri://localhost" }.parse().unwrap(),
                    body: tauri::ipc::InvokeBody::Raw(bytes),
                    headers: [(
                        tauri::http::HeaderName::from_static(dokime_gui::rpc::PATH_HEADER),
                        tauri::http::HeaderValue::from_static("/harmos.v1.Status/Get"),
                    )]
                    .into_iter()
                    .collect(),
                    invoke_key: tauri::test::INVOKE_KEY.into(),
                },
            )
            .map_err(|error| format!("{error:?}"))?;
            let tauri::ipc::InvokeResponseBody::Raw(response) = response else {
                return Err("IPC returned JSON instead of canonical bytes".to_owned());
            };
            assert_eq!(u64::from_le_bytes(response[..8].try_into().unwrap()), generation);
            library::StatusGetResponse::decode(&response[8..]).map_err(|error| error.to_string())
        })
        .await??;
        assert_eq!(
            response.status.as_ref().and_then(|status| status.identity.as_ref()).map(|identity| &identity.host_id),
            ready.connection.backend.as_ref().map(|backend| &backend.backend_id)
        );

        // Replacing the Selection Cancels the Old Lease Without Ending the Owned Host
        owner
            .select(BackendSelection::Attach {
                request: dokime_gui::connection::ConnectRequest {
                    target: dokime_gui::connection::ConnectionTarget::Local { endpoint },
                    expected_backend_id: None,
                },
            })
            .await
            .map_err(|error| format!("{error:?}"))?;
        assert!(matches!(first.execute(async { Ok(()) }).await, Err(ConnectionError::Superseded)));
        assert!(connection.lease(&ready.connection.generation).is_err());
        let next = connection.lease(&connection.status().generation).map_err(|error| format!("{error:?}"))?;
        next.execute(async { Ok(()) }).await.map_err(|error| format!("{error:?}"))?;
        let mut observer = Connection::new(settings())?;
        let session = observer.attach_local(&ready.connection.endpoint.unwrap()).await?;
        let suites: client::SuitesListResponse =
            session.unary("/dokime.client.v1.Suites/List", client::SuitesListRequest::default()).await?;
        assert!(suites.values.is_empty());
        observer.close().await?;
        owner.shutdown().await.map_err(|error| format!("{error:?}"))?;
        assert!(matches!(next.execute(async { Ok(()) }).await, Err(ConnectionError::Superseded)));
        assert_eq!(owner.status().ownership, BackendOwnership::None);
        Ok(())
    })
}

#[test]
fn desktop_refuses_an_occupied_root_but_can_attach_to_its_owner() -> Result<(), Box<dyn StdError>> {
    dokime_gui::test_support::run(async {
        use dokime_gui::connection::{ConnectRequest, ConnectionSession, ConnectionTarget};
        use dokime_gui::native_session::{BackendOwnership, BackendSelection, DesktopSession, DesktopSessionError, backend::Backend};
        let root = dokime_gui::test_support::tempdir()?;
        let mut first = Connection::new(settings())?;
        let session = first.start(root.path()).await?;
        let active = ConnectionSession::default();
        let second = DesktopSession::with_local_backend(active.clone(), Backend::new(root.path().to_path_buf()));
        let refused = second.select(BackendSelection::OwnedLocal).await;
        assert!(matches!(refused, Err(DesktopSessionError::BackendRunning { root: occupied, .. }) if occupied == root.path()));
        let attached = second
            .select(BackendSelection::Attach {
                request: ConnectRequest {
                    target: ConnectionTarget::Local { endpoint: session.endpoint().to_owned() },
                    expected_backend_id: None,
                },
            })
            .await
            .map_err(|error| format!("{error:?}"))?;
        assert_eq!(attached.ownership, BackendOwnership::Attached);
        second.shutdown().await.map_err(|error| format!("{error:?}"))?;
        let status: library::StatusGetResponse = session.unary("/harmos.v1.Status/Get", library::StatusGetRequest {}).await?;
        assert!(status.status.and_then(|value| value.identity).is_some());
        first.close().await?;
        let third = DesktopSession::with_local_backend(ConnectionSession::default(), Backend::new(root.path().to_path_buf()));
        third.select(BackendSelection::OwnedLocal).await.map_err(|error| format!("{error:?}"))?;
        third.shutdown().await.map_err(|error| format!("{error:?}"))?;
        Ok(())
    })
}

#[test]
fn desktop_attachment_preserves_an_incompatible_local_root() -> Result<(), Box<dyn StdError>> {
    dokime_gui::test_support::run(async {
        use dokime_gui::connection::{ConnectRequest, ConnectionSession, ConnectionTarget};
        use dokime_gui::native_session::{BackendOwnership, BackendSelection, DesktopSession, backend::Backend};
        let healthy_root = dokime_gui::test_support::tempdir()?;
        let mut healthy = Connection::new(settings())?;
        let session = healthy.start(healthy_root.path()).await?;
        let incompatible = dokime_gui::test_support::tempdir()?;
        let database = incompatible.path().join("execution.sqlite3");
        let original = b"an incompatible database preserved for recovery";
        std::fs::write(&database, original)?;
        let desktop = DesktopSession::with_local_backend(ConnectionSession::default(), Backend::new(incompatible.path().to_path_buf()));
        let attached = desktop
            .select(BackendSelection::Attach {
                request: ConnectRequest {
                    target: ConnectionTarget::Local { endpoint: session.endpoint().to_owned() },
                    expected_backend_id: None,
                },
            })
            .await
            .map_err(|error| format!("{error:?}"))?;
        assert_eq!(attached.ownership, BackendOwnership::Attached);
        assert_eq!(std::fs::read(&database)?, original);
        desktop.shutdown().await.map_err(|error| format!("{error:?}"))?;
        let status: library::StatusGetResponse = session.unary("/harmos.v1.Status/Get", library::StatusGetRequest {}).await?;
        assert!(status.status.and_then(|value| value.identity).is_some());
        healthy.close().await?;
        Ok(())
    })
}

#[test]
fn library_trace_snapshot_watch_and_verified_diagnostics() -> Result<(), Box<dyn StdError>> {
    dokime_gui::test_support::run(async {
        use library::status;
        use sha2::{Digest, Sha256};
        let root = dokime_gui::test_support::tempdir()?;
        let configuration = settings();
        let server =
            services::serve(([127, 0, 0, 1], 0).into(), root.path(), configuration.host.clone(), configuration.services.clone()).await?;
        let mut owner = Connection::new(configuration)?;
        let session = owner.attach_local(&format!("http://{}", server.endpoint())).await?;
        let journal = server.status();
        journal.record_trace(status::Event {
            level: "INFO".into(),
            fields: [("execution_id".into(), "desktop-run".into())].into(),
            ..Default::default()
        })?;
        let snapshot: status::TraceEventsListResponse = session
            .unary(
                "/harmos.v1.status.TraceEvents/List",
                status::TraceEventsListRequest {
                    page: Some(library::PageRequest {
                        page_size: 10,
                        max_bytes: 65536,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(snapshot.events.len(), 1);
        assert_eq!(snapshot.events[0].fields.get("execution_id").unwrap(), "desktop-run");
        let mut watch = session
            .watch_raw(
                "/harmos.v1.status.TraceEvents/Watch",
                &status::TraceEventsWatchRequest {
                    at: snapshot.resolved_at.clone(),
                    max_bytes: 65536,
                    max_events: 10,
                }
                .encode_to_vec(),
            )
            .await?;
        let first = watch.next().await.ok_or("Watch Trace: Missing Opening")??;
        let first = status::TraceEventsWatchResponse::decode(first.as_slice())?;
        assert_eq!(first.resolved_at, snapshot.resolved_at);
        journal.record_trace(status::Event {
            level: "WARN".into(),
            ..Default::default()
        })?;
        let live = watch.next().await.ok_or("Watch Trace: Missing New Event")??;
        let live = status::TraceEventsWatchResponse::decode(live.as_slice())?;
        assert_eq!(live.events.len(), 1);
        assert!(live.resolved_at.unwrap().sequence > snapshot.resolved_at.unwrap().sequence);
        let mut name = String::new();
        let mut bytes = Vec::new();
        let mut digest = Vec::new();
        loop {
            let part: status::DiagnosticsGetResponse = session
                .unary(
                    "/harmos.v1.status.Diagnostics/Get",
                    status::DiagnosticsGetRequest {
                        name: name.clone(),
                        offset: bytes.len() as u64,
                        max_bytes: 1024,
                    },
                )
                .await?;
            if name.is_empty() {
                name = part.name.clone();
                digest = part.sha256.clone();
            }
            assert_eq!(part.name, name);
            assert_eq!(part.sha256, digest);
            bytes.extend(part.data);
            if bytes.len() as u64 == part.total_length {
                break;
            }
        }
        assert_eq!(Sha256::digest(&bytes).as_slice(), digest);
        owner.close().await?;
        assert!(session.cancellation().is_cancelled());
        while let Some(frame) = watch.next().await {
            frame?;
        }
        server.close().await?;
        Ok(())
    })
}

#[test]
fn failed_story_cleanup() {
    let path = Arc::new(Mutex::new(PathBuf::new()));
    let observed = path.clone();
    let failure = std::panic::catch_unwind(|| {
        dokime_gui::test_support::run(async {
            let root = dokime_gui::test_support::tempdir().unwrap();
            *observed.lock().unwrap() = root.path().to_owned();
            let owner = dokime_gui::native_session::DesktopSession::with_local_backend(
                dokime_gui::connection::ConnectionSession::default(),
                dokime_gui::native_session::backend::Backend::new(root.path().to_owned()),
            );
            owner.select(dokime_gui::native_session::BackendSelection::OwnedLocal).await.unwrap();
            panic!("fixture.story.failed");
        })
    });
    assert!(failure.is_err());
    assert!(!path.lock().unwrap().exists());
}
