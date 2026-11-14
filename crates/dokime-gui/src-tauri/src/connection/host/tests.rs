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
use dokime_protocol::{client, v1};
use dokime_runtime_next::{ControlledClock, PlacementSettings};
use dokime_sdk::client::v1::{self as library, access};
use sha2::Digest;
use sha2::Sha256;
use std::error::Error as StdError;
use std::ffi::OsString;
use std::path::PathBuf;
use tokio_stream::StreamExt;
use tonic::Code;

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
            transfer: dokime_sdk::client::StorageTransferSettings { chunk_bytes: Some(64 * 1024) },
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
    crate::test_support::run(async {
        // Start the Embedded Host and Read Its Ready SDK Identity
        let root = crate::test_support::tempdir()?;
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
    crate::test_support::run(async {
        // A Foreign Root Must Stay Byte-for-Byte Available for Backup
        let existing = crate::test_support::tempdir()?;
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
        let fresh = crate::test_support::tempdir()?;
        owner.start(fresh.path()).await?;
        assert_eq!(std::fs::read(&evidence)?, original);
        owner.close().await?;
        Ok(())
    })
}

#[test]
fn sdk_calls_and_watches_share_the_owned_connection() -> Result<(), Box<dyn StdError>> {
    crate::test_support::run(async {
        // Observe Current State Through the SDK Watch Transport
        let root = crate::test_support::tempdir()?;
        let mut owner = Connection::new(settings())?;
        let session = owner.start(root.path()).await?;
        let discovered = crate::connection::transport::backend(&session);
        assert!(discovered.methods.iter().any(|method| method == "/harmos.v1.access.Listener/Get"));
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
        assert!(session.watches.lock().await.is_empty());
        Ok(())
    })
}

#[test]
fn pinned_pairing_identity_and_certificate_rotation() -> Result<(), Box<dyn StdError>> {
    crate::test_support::run(async {
        // Enable an Isolated Authenticated Listener Through the SDK Root
        let root = crate::test_support::tempdir()?;
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
    })
}

#[test]
fn thermal_execution_prompts_evidence_and_report() -> Result<(), Box<dyn StdError>> {
    crate::test_support::run(async {
        let executable = std::env::var_os("DOKIME_THERMAL_PLUGIN")
            .map(PathBuf::from)
            .unwrap_or_else(|| crate::test_support::fixture("dokime-rust-thermal"));
        let configuration = settings();
        let mut plugin = dokime_runtime_next::PluginProcess::start(&executable, true, &configuration.host.placement).await?;
        let root = crate::test_support::tempdir()?;
        let mut owner = Connection::new(configuration)?;
        let result = async {
            // Admit the Prepared Plugin and Read Its Top-Level Suite Resource.
            let session = owner.start(root.path()).await?;
            let admitted: client::Plugin =
                session.unary("/dokime.client.v1.Plugins/Add", client::AddRemotePlugin { url: plugin.endpoint().into() }).await?;
            assert!(!admitted.entry.is_empty());
            let suites: client::SuitesListResponse =
                session.unary("/dokime.client.v1.Suites/List", client::SuitesListRequest::default()).await?;
            let suite = suites.values.first().ok_or("Read Suite: Missing Prepared Fixture")?;
            let suite_id = suite.metadata.as_ref().ok_or("Read Suite: Missing Identity")?.identifier.clone();
            let suite: client::SuitesGetResponse = session
                .unary(
                    "/dokime.client.v1.Suites/Get",
                    client::SuitesGetRequest {
                        name: suite_id.clone(),
                        ..Default::default()
                    },
                )
                .await?;
            let suite = suite.value.ok_or("Read Suite: Missing Definition")?;
            let template = suite.templates.first().and_then(|template| template.metadata.as_ref()).ok_or("Read Suite: Missing Template")?;
            let plan: client::CasesCreateResponse = session
                .unary(
                    "/dokime.client.v1.Cases/Create",
                    client::CasesCreateRequest {
                        plan: Some(client::SelectionPlan {
                            selections: vec![client::RegisteredSelection {
                                suite_id,
                                template_id: template.identifier.clone(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        validate_only: true,
                        request_id: "desktop-preview".into(),
                    },
                )
                .await?;
            let origins =
                plan.cases.into_iter().map(|case| case.origin.ok_or("Materialize Case: Missing Origin")).collect::<Result<Vec<_>, _>>()?;
            let cycle: client::CyclesCreateResponse = session
                .unary(
                    "/dokime.client.v1.Cycles/Create",
                    client::CyclesCreateRequest {
                        origins,
                        continue_automatically: true,
                        request_id: "desktop-cycle".into(),
                        ..Default::default()
                    },
                )
                .await?;
            let cycle = cycle.cycle.ok_or("Create Cycle: Missing Value")?;
            let executions: client::ExecutionsListResponse = session
                .unary(
                    "/dokime.client.v1.Executions/List",
                    client::ExecutionsListRequest {
                        parent: cycle.cycle_id.clone(),
                        ..Default::default()
                    },
                )
                .await?;
            let execution = executions
                .executions
                .first()
                .and_then(|item| item.summary.as_ref())
                .ok_or("Start Execution: Missing Identity")?
                .execution_id
                .clone();

            // Await the Operator Question Through the Owned Progress Watch.
            let request = client::executions::ProgressWatchRequest { name: execution.clone(), at: None }.encode_to_vec();
            let mut progress = session.watch_raw("/dokime.client.v1.executions.Progress/Watch", &request).await?;
            loop {
                let frame = progress.next().await.ok_or("Observe Execution: Closed Before Prompt")??;
                let frame = client::executions::ProgressWatchResponse::decode(frame.as_slice())?;
                assert!(!frame.finished);
                if frame.phase == "prompt" {
                    break;
                }
            }
            let prompts: client::PromptsListResponse = session
                .unary(
                    "/dokime.client.v1.Prompts/List",
                    client::PromptsListRequest {
                        execution_id: Some(execution.clone()),
                        pending_only: true,
                        ..Default::default()
                    },
                )
                .await?;
            let prompt = prompts.prompts.first().and_then(|view| view.prompt.as_ref()).ok_or("Answer Prompt: Missing Question")?;

            // Observe a Plot Before Answering, Then Join the Terminal Progress.
            let plot_request = v1::PlotRequest {
                plot_id: Some("temperature".into()),
                stream: "thermal.temperature".into(),
                y_keys: vec!["celsius".into()],
                scope: Some(v1::EvidenceScope {
                    execution_id: Some(execution.clone()),
                    ..Default::default()
                }),
                technique: v1::Downsample::Raw as i32,
                max_points: Some(128),
                ..Default::default()
            };
            let request = client::plots::SnapshotsWatchRequest {
                requests: vec![plot_request.clone()],
                ..Default::default()
            }
            .encode_to_vec();
            let mut plots = session.watch_raw("/dokime.client.v1.plots.Snapshots/Watch", &request).await?;
            let opening = plots.next().await.ok_or("Watch Plot: Missing Opening")??;
            let opening = client::plots::SnapshotsWatchResponse::decode(opening.as_slice())?;
            assert!(opening.opening);
            assert_eq!(opening.updates.len(), 1);
            let _: client::prompts::AnswersCreateResponse = session
                .unary(
                    "/dokime.client.v1.prompts.Answers/Create",
                    client::prompts::AnswersCreateRequest {
                        parent: prompt.prompt_id.clone(),
                        action: "yes".into(),
                        accepted: true,
                        request_id: "desktop-answer".into(),
                        ..Default::default()
                    },
                )
                .await?;
            let mut terminal = false;
            while let Some(frame) = progress.next().await {
                assert!(!terminal);
                terminal = client::executions::ProgressWatchResponse::decode(frame?.as_slice())?.finished;
            }
            assert!(terminal);
            drop(plots);

            // Decode Sealed Evidence With the Digest-Cached Descriptor Set.
            let detail: client::ExecutionsGetResponse = session
                .unary(
                    "/dokime.client.v1.Executions/Get",
                    client::ExecutionsGetRequest {
                        name: execution.clone(),
                        ..Default::default()
                    },
                )
                .await?;
            let detail = detail.execution.ok_or("Get Execution: Missing Value")?;
            assert_eq!(detail.summary.as_ref().unwrap().state, v1::ExecutionState::Sealed as i32);
            assert!(!detail.seal.is_empty());
            let mut cache = dokime_sdk::client::FrozenSchemaCache::default();
            let descriptors = cache.get(session.client(), &detail).await?;
            assert!(std::sync::Arc::ptr_eq(&descriptors, &cache.get(session.client(), &detail).await?));
            let records: client::evidence::RecordsListResponse = session
                .unary(
                    "/dokime.client.v1.evidence.Records/List",
                    client::evidence::RecordsListRequest {
                        selection: Some(client::evidence::CapturedRecordSelection {
                            scope: Some(v1::EvidenceScope {
                                execution_id: Some(execution.clone()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                )
                .await?;
            assert!(records.records.iter().any(|record| record.record_type == "thermal.door"));
            let samples: client::evidence::SamplesListResponse = session
                .unary(
                    "/dokime.client.v1.evidence.Samples/List",
                    client::evidence::SamplesListRequest {
                        scope: Some(v1::EvidenceScope {
                            execution_id: Some(execution.clone()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                )
                .await?;
            assert!(!samples.samples.is_empty());
            let snapshots: client::plots::SnapshotsListResponse = session
                .unary(
                    "/dokime.client.v1.plots.Snapshots/List",
                    client::plots::SnapshotsListRequest {
                        requests: vec![plot_request],
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(snapshots.envelopes.len(), 1);

            // Consume Every Ordered Document Page at the Returned Cut.
            let mut request = client::DocumentsGetRequest {
                scope: Some(v1::DocumentScope {
                    cycle_id: cycle.cycle_id.clone(),
                    execution_id: execution.clone(),
                }),
                ..Default::default()
            };
            let mut header = false;
            let mut completed = false;
            loop {
                let reply: client::DocumentsGetResponse = session.unary("/dokime.client.v1.Documents/Get", request.clone()).await?;
                for frame in reply.frames {
                    assert!(!completed);
                    match frame.frame.and_then(|frame| frame.kind) {
                        Some(client::document_frame_frame::Kind::Header(value)) => {
                            assert!(!header);
                            assert_eq!(value.frontiers.unwrap().process_epoch, session.product_epoch());
                            header = true;
                        }
                        Some(client::document_frame_frame::Kind::Completion(_)) => completed = true,
                        _ => {}
                    }
                }
                if reply.next_page_token.is_empty() {
                    break;
                }
                assert_ne!(request.page_token, reply.next_page_token);
                request.page_token = reply.next_page_token;
                request.at = reply.resolved_at;
            }
            assert!(header && completed);

            // Deliver the Report Through Product File Ranges and the Native Verified Writer.
            let subject = client::ReportSubject {
                kind: Some(client::report_subject::Kind::Execution(execution.clone())),
            };
            let prepared: client::ReportsCreateResponse = session
                .unary(
                    "/dokime.client.v1.Reports/Create",
                    client::ReportsCreateRequest {
                        subject: Some(subject.clone()),
                        request_id: "desktop-prepare-report".into(),
                        ..Default::default()
                    },
                )
                .await?;
            let file = prepared.file.ok_or("Prepare Report: Missing File Reference")?;
            let _: client::ReportsPublishResponse = session
                .unary(
                    "/dokime.client.v1.Reports/Publish",
                    client::ReportsPublishRequest {
                        name: file.metadata.ok_or("Publish Report: Missing File Identity")?.id,
                        subject: Some(subject),
                        expected_revision: file.sha256,
                        request_id: "desktop-publish-report".into(),
                    },
                )
                .await?;
            let report: client::ReportsGetResponse = session
                .unary(
                    "/dokime.client.v1.Reports/Get",
                    client::ReportsGetRequest {
                        subject: Some(client::ReportSubject {
                            kind: Some(client::report_subject::Kind::Execution(execution.clone())),
                        }),
                        ..Default::default()
                    },
                )
                .await?;
            let reference = report.file.ok_or("Read Report: Missing File Reference")?;
            let selected: client::FilesGetResponse =
                session.unary("/dokime.client.v1.Files/Get", client::FilesGetRequest { file: Some(reference.clone()), at: None }).await?;
            let mut bytes = Vec::new();
            let mut at = selected.resolved_at;
            while bytes.len() < reference.length as usize {
                let reply: client::FilesReadResponse = session
                    .unary(
                        "/dokime.client.v1.Files/Read",
                        client::FilesReadRequest {
                            file: Some(reference.clone()),
                            offset: bytes.len() as u64,
                            max_bytes: selected.chunk_bytes,
                            at: at.clone(),
                        },
                    )
                    .await?;
                assert_eq!(reply.total_length, reference.length);
                assert_eq!(reply.sha256, reference.sha256);
                assert!(!reply.data.is_empty());
                if at.is_some() {
                    assert_eq!(at, reply.resolved_at);
                }
                at = reply.resolved_at;
                bytes.extend(reply.data);
            }
            assert_eq!(Sha256::digest(&bytes).as_slice(), reference.sha256);
            let active = crate::connection::ConnectionSession::default();
            let connected = active
                .connect(crate::connection::ConnectRequest {
                    target: crate::connection::ConnectionTarget::Local { endpoint: session.endpoint().to_owned() },
                    expected_backend_id: None,
                })
                .await
                .map_err(|error| format!("{error:?}"))?;
            let lease = active.lease(&connected.generation).map_err(|error| format!("{error:?}"))?;
            let destination = root.path().join("native-export.html");
            let exported = crate::app::content::fetch_verified(
                &lease,
                &crate::app::desktop::ArtifactSource::ExecutionReport { execution_id: execution },
                &destination,
                crate::rpc::protocol::FRAME_BYTES as usize,
            )
            .await?;
            assert_eq!(exported.size_bytes, reference.length);
            assert_eq!(std::fs::read(destination)?, bytes);
            active.disconnect().map_err(|error| format!("{error:?}"))?;
            Ok::<(), Box<dyn StdError>>(())
        }
        .await;
        let host_close = owner.close().await;
        let plugin_close = plugin.close().await;
        result?;
        host_close?;
        plugin_close?;
        Ok(())
    })
}

#[test]
fn desktop_owner_routes_canonical_ipc_and_replaces_generation() -> Result<(), Box<dyn StdError>> {
    crate::test_support::run(async {
        use crate::connection::ConnectionSession;
        use crate::native_session::{BackendOwnership, BackendSelection, DesktopSession, backend::Backend};

        // Start Through the Application's Actual Native Owner
        let root = crate::test_support::tempdir()?;
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
                .manage(crate::rpc::Bridge::default())
                .invoke_handler(crate::rpc::handler())
                .build(tauri::test::mock_context(tauri::test::noop_assets()))
                .unwrap();
            let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();
            let mut bytes = generation.to_le_bytes().to_vec();
            bytes.extend(crate::rpc::descriptor_digest());
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
                        tauri::http::HeaderName::from_static(crate::rpc::PATH_HEADER),
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
                request: crate::connection::ConnectRequest {
                    target: crate::connection::ConnectionTarget::Local { endpoint },
                    expected_backend_id: None,
                },
            })
            .await
            .map_err(|error| format!("{error:?}"))?;
        assert!(first.cancellation().is_cancelled());
        assert!(connection.lease(&ready.connection.generation).is_err());
        let next = connection.lease(&connection.status().generation).map_err(|error| format!("{error:?}"))?;
        let suites: client::SuitesListResponse =
            next.client().unary("/dokime.client.v1.Suites/List", client::SuitesListRequest::default()).await?;
        assert!(suites.values.is_empty());
        owner.shutdown().await.map_err(|error| format!("{error:?}"))?;
        assert!(next.cancellation().is_cancelled());
        assert_eq!(owner.status().ownership, BackendOwnership::None);
        Ok(())
    })
}

#[test]
fn desktop_refuses_an_occupied_root_but_can_attach_to_its_owner() -> Result<(), Box<dyn StdError>> {
    crate::test_support::run(async {
        use crate::connection::{ConnectRequest, ConnectionSession, ConnectionTarget};
        use crate::native_session::{BackendOwnership, BackendSelection, DesktopSession, DesktopSessionError, backend::Backend};
        let root = crate::test_support::tempdir()?;
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
    crate::test_support::run(async {
        use crate::connection::{ConnectRequest, ConnectionSession, ConnectionTarget};
        use crate::native_session::{BackendOwnership, BackendSelection, DesktopSession, backend::Backend};
        let healthy_root = crate::test_support::tempdir()?;
        let mut healthy = Connection::new(settings())?;
        let session = healthy.start(healthy_root.path()).await?;
        let incompatible = crate::test_support::tempdir()?;
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
    crate::test_support::run(async {
        use library::status;
        use sha2::{Digest, Sha256};
        let root = crate::test_support::tempdir()?;
        let mut owner = Connection::new(settings())?;
        let session = owner.start(root.path()).await?;
        let journal = owner.server.as_ref().unwrap().status();
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
        Ok(())
    })
}
