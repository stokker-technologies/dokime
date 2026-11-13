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
fn embedded_readiness_attachment_and_restart() -> Result<(), Box<dyn std::error::Error>> {
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
fn incompatible_storage_is_preserved_and_retry_uses_an_explicit_fresh_root() -> Result<(), Box<dyn std::error::Error>> {
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
        assert_eq!(entries, vec![std::ffi::OsString::from("existing-evidence.bin"), std::ffi::OsString::from("local.lock")]);
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
fn sdk_calls_and_watches_share_the_owned_connection() -> Result<(), Box<dyn std::error::Error>> {
    crate::test_support::run(async {
        // Observe Current State Through the SDK Watch Transport
        let root = crate::test_support::tempdir()?;
        let mut owner = Connection::new(settings())?;
        let session = owner.start(root.path()).await?;
        let discovered = crate::connection::transport::backend(&session);
        assert!(discovered.methods.iter().any(|method| method == "/harmos.v1.access.Listener/Get"));
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
    })
}

#[test]
fn pinned_pairing_identity_and_certificate_rotation() -> Result<(), Box<dyn std::error::Error>> {
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
fn thermal_catalog_execution_prompts_evidence_and_report() -> Result<(), Box<dyn std::error::Error>> {
    crate::test_support::run(async {
        // Start the Prepared Shared Plugin and the Embedded Desktop Host
        let executable = std::env::var_os("DOKIME_THERMAL_PLUGIN")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| crate::test_support::fixture("dokime-rust-thermal"));
        if !executable.is_file() {
            return Err("Prepare Desktop Execution: Run mise run build:host-fixtures -- --language rust".into());
        }
        let configuration = settings();
        let mut plugin = dokime_runtime_next::PluginProcess::start(&executable, true, &configuration.host.placement).await?;
        let root = crate::test_support::tempdir()?;
        let mut owner = Connection::new(configuration)?;
        let result = async {
        let session = owner.start(root.path()).await?;
        let request = v1::AddRemotePlugin { url: plugin.endpoint().into() }.encode_to_vec();
        let response = session.call_raw("/dokime.v1.Plugins/Add", &request).await?;
        let admitted = v1::Plugin::decode(response.as_slice())?;
        assert!(!admitted.entry.is_empty());

        // Read the Canonical Catalog and Materialize Its Frozen Case
        let request = v1::CatalogListSuitesRequest::default().encode_to_vec();
        let response = session.call_raw("/dokime.v1.Catalog/ListSuites", &request).await?;
        let suites = v1::ListSuitesReply::decode(response.as_slice())?;
        let suite = suites.suites.first().ok_or("Read Desktop Catalog: Missing Prepared Suite")?;
        let request = v1::CatalogGetSuiteRequest { suite_id: suite.suite_id.clone() }.encode_to_vec();
        let response = session.call_raw("/dokime.v1.Catalog/GetSuite", &request).await?;
        let suite = v1::Suite::decode(response.as_slice())?;
        let template = suite.templates.first().and_then(|template| template.metadata.as_ref()).ok_or("Read Desktop Catalog: Missing Template")?;
        let request = v1::PlanningMaterializeTemplateRequest {
            selection: Some(v1::RegisteredSelection {
                suite_id: suite.metadata.as_ref().ok_or("Read Desktop Catalog: Missing Suite Identity")?.identifier.clone(),
                template_id: template.identifier.clone(),
                ..Default::default()
            }),
        }.encode_to_vec();
        let response = session.call_raw("/dokime.v1.Planning/MaterializeTemplate", &request).await?;
        let plan = v1::PreviewSelectionReply::decode(response.as_slice())?;
        let origins = plan.cases.into_iter().map(|case| case.origin.ok_or("Materialize Desktop Case: Missing Frozen Origin")).collect::<Result<Vec<_>, _>>()?;

        // Start the Execution and Wait for Its Operator Question Through SDK Watch
        let request = v1::ExecutionCreateCycleRequest { origins, continue_automatically: true, ..Default::default() }.encode_to_vec();
        let response = session.call_raw("/dokime.v1.Execution/CreateCycle", &request).await?;
        let cycle = v1::CreateCycleReply::decode(response.as_slice())?;
        let execution = cycle.execution_ids.first().ok_or("Start Desktop Execution: Missing Execution Identity")?.clone();
        let request = v1::ObservationWatchExecutionProgressRequest { test_execution_id: execution.clone() }.encode_to_vec();
        let mut progress = session.watch_raw("/dokime.v1.Observation/WatchExecutionProgress", &request).await?;
        loop {
            let frame = progress.next().await.ok_or("Observe Desktop Execution: Closed Before Prompt")??;
            let frame = v1::ObservationWatchExecutionProgressOutput::decode(frame.as_slice())?;
            let Some(v1::observation_watch_execution_progress_output::Value::Item(frame)) = frame.value else {
                return Err("Observe Desktop Execution: Finished Before Prompt".into());
            };
            if frame.progress.is_some_and(|progress| progress.phase == "prompt") { break; }
        }
        let request = v1::PromptsListPendingRequest { execution_id: Some(execution.clone()), ..Default::default() }.encode_to_vec();
        let response = session.call_raw("/dokime.v1.Prompts/ListPending", &request).await?;
        let prompts = v1::ListPendingPromptsReply::decode(response.as_slice())?;
        let prompt = prompts.prompts.first().ok_or("Answer Desktop Prompt: Missing Pending Question")?;
        // Upload Through the Same Owned SDK Session While Its Execution Is Open
        use sha2::Digest;
        let attachment = vec![42; 140_003];
        let attachment_digest = sha2::Sha256::digest(&attachment).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        let mut upload = vec![v1::UploadArtifactFrame {
            payload: Some(v1::UploadArtifactFramePayload { kind: Some(v1::upload_artifact_frame_payload::Kind::Header(v1::UploadArtifactHeader {
                scope: Some(v1::ArtifactScope { execution_id: Some(execution.clone()), cycle_id: Some(cycle.cycle_id.clone()), ..Default::default() }),
                filename: "attachment.bin".into(), size_bytes: attachment.len() as u64, sha256: Some(attachment_digest.clone()), ..Default::default()
            })) }),
        }];
        for (index, data) in attachment.chunks(64 * 1024).enumerate() {
            upload.push(v1::UploadArtifactFrame { payload: Some(v1::UploadArtifactFramePayload { kind: Some(v1::upload_artifact_frame_payload::Kind::Chunk(v1::ArtifactChunk { offset: (index * 64 * 1024) as u64, data: data.to_vec() })) }) });
        }
        upload.push(v1::UploadArtifactFrame { payload: Some(v1::UploadArtifactFramePayload { kind: Some(v1::upload_artifact_frame_payload::Kind::Complete(v1::ArtifactCompletion { size_bytes: attachment.len() as u64, sha256: attachment_digest.clone() })) }) });
        let artifact = session.upload_artifact(tokio_stream::iter(upload)).await?;

        // Subscribe to the Execution Plot Before Answering the Live Prompt
        let plot_request = v1::PlotRequest {
            plot_id: Some("temperature".into()), stream: "thermal.temperature".into(), y_keys: vec!["celsius".into()],
            scope: Some(v1::EvidenceScope { execution_id: Some(execution.clone()), ..Default::default() }),
            technique: v1::Downsample::Raw as i32, max_points: Some(128), ..Default::default()
        };
        let request = v1::PlotsWatchRequest { requests: vec![plot_request.clone()] }.encode_to_vec();
        let mut plots = session.watch_raw("/dokime.v1.Plots/Watch", &request).await?;
        let opening = plots.next().await.ok_or("Watch Desktop Plot: Missing Opening Snapshot")??;
        let opening = v1::PlotsWatchOutput::decode(opening.as_slice())?;
        let Some(v1::plots_watch_output::Value::Item(opening)) = opening.value else { return Err("Watch Desktop Plot: Missing Opening Item".into()); };
        assert_eq!(opening.updates.len(), 1);
        assert_eq!(opening.updates[0].kind, v1::PlotStreamKind::Snapshot as i32);
        assert_eq!(opening.updates[0].envelope.as_ref().unwrap().plot_id, "temperature");

        let request = v1::PromptsRespondRequest { prompt_id: prompt.prompt_id.clone(), action: "yes".into(), accepted: true, ..Default::default() }.encode_to_vec();
        session.call_raw("/dokime.v1.Prompts/Respond", &request).await?;
        let mut terminal = false;
        while let Some(frame) = progress.next().await {
            let frame = v1::ObservationWatchExecutionProgressOutput::decode(frame?.as_slice())?;
            match frame.value {
                Some(v1::observation_watch_execution_progress_output::Value::Item(_)) if !terminal => {}
                Some(v1::observation_watch_execution_progress_output::Value::Response(_)) if !terminal => terminal = true,
                _ => return Err("Observe Desktop Execution: Invalid Terminal Sequence".into()),
            }
        }
        assert!(terminal);

        // Inspect the Sealed Execution and Its Captured Record and Telemetry
        let request = v1::ExecutionGetExecutionRequest { execution_id: execution.clone() }.encode_to_vec();
        let response = session.call_raw("/dokime.v1.Execution/GetExecution", &request).await?;
        let detail = v1::ExecutionDetail::decode(response.as_slice())?;
        assert_eq!(detail.summary.as_ref().unwrap().state, v1::ExecutionState::Sealed as i32);
        assert!(!detail.seal.is_empty());
        assert!(matches!(detail.outcome.as_ref().and_then(|outcome| outcome.attribution.as_ref()), Some(v1::execution_outcome::Attribution::Evaluated(outcome)) if outcome.status == v1::ResultStatus::Passed as i32));
        let request = v1::EvidenceListRecordsRequest { selection: Some(v1::CapturedRecordSelection {
            scope: Some(v1::EvidenceScope { execution_id: Some(execution.clone()), ..Default::default() }), ..Default::default()
        }), ..Default::default() }.encode_to_vec();
        let response = session.call_raw("/dokime.v1.Evidence/ListRecords", &request).await?;
        let records = v1::RecordPage::decode(response.as_slice())?;
        assert!(records.records.iter().any(|record| !record.payload.is_empty()));
        let request = v1::TelemetryListSamplesRequest {
            scope: Some(v1::EvidenceScope { execution_id: Some(execution.clone()), ..Default::default() }), ..Default::default()
        }.encode_to_vec();
        let response = session.call_raw("/dokime.v1.Telemetry/ListSamples", &request).await?;
        let samples = v1::TelemetrySamplesReply::decode(response.as_slice())?;
        assert!(samples.samples.iter().any(|sample| !sample.payload.is_empty()));

        // Settle the Live Plot and Read the Same Saved Sample Set
        let mut plot_terminal = false;
        let mut plotted = false;
        while let Some(frame) = plots.next().await {
            let frame = v1::PlotsWatchOutput::decode(frame?.as_slice())?;
            match frame.value {
                Some(v1::plots_watch_output::Value::Item(frame)) if !plot_terminal => {
                    for update in frame.updates {
                        assert_eq!(update.kind, v1::PlotStreamKind::Snapshot as i32);
                        let payload = update.envelope.and_then(|envelope| envelope.payload).ok_or("Watch Desktop Plot: Missing Payload")?;
                        plotted |= payload.series.iter().any(|series| series.raw_points > 0);
                    }
                }
                Some(v1::plots_watch_output::Value::Response(_)) if !plot_terminal => plot_terminal = true,
                _ => return Err("Watch Desktop Plot: Invalid Terminal Sequence".into()),
            }
        }
        assert!(plot_terminal && plotted);
        let request = v1::PlotsQuerySnapshotsRequest { requests: vec![plot_request] }.encode_to_vec();
        let response = session.call_raw("/dokime.v1.Plots/QuerySnapshots", &request).await?;
        let saved = v1::PlotSnapshotsReply::decode(response.as_slice())?;
        let saved = saved.envelopes.first().and_then(|envelope| envelope.payload.as_ref()).ok_or("Read Desktop Plot: Missing Saved Payload")?;
        assert!(saved.series.iter().any(|series| series.raw_points == samples.samples.len() as u64));

        // Capture the Canonical Default Document with Its Product Process Epoch
        let request = v1::DocumentsCaptureCurrentDocumentRequest {
            scope: Some(v1::DocumentScope { cycle_id: cycle.cycle_id.clone(), execution_id: execution.clone() }),
            ..Default::default()
        }.encode_to_vec();
        let mut document = session.watch_raw("/dokime.v1.Documents/CaptureCurrentDocument", &request).await?;
        let mut document_header = false;
        let mut document_complete = false;
        let mut document_terminal = false;
        while let Some(frame) = document.next().await {
            let frame = v1::DocumentsCaptureCurrentDocumentOutput::decode(frame?.as_slice())?;
            match frame.value {
                Some(v1::documents_capture_current_document_output::Value::Item(frame)) if !document_terminal => match frame.frame.and_then(|frame| frame.kind) {
                    Some(v1::document_frame_frame::Kind::Header(header)) => {
                        assert!(!document_header);
                        assert_eq!(header.document_id, "execution");
                        assert_eq!(header.frontiers.as_ref().unwrap().process_epoch, session.product_epoch());
                        document_header = true;
                    }
                    Some(v1::document_frame_frame::Kind::Completion(_)) => document_complete = true,
                    _ => {}
                },
                Some(v1::documents_capture_current_document_output::Value::Response(_)) if !document_terminal => document_terminal = true,
                _ => return Err("Read Desktop Document: Invalid Terminal Sequence".into()),
            }
        }
        assert!(document_header && document_complete && document_terminal);

        // Read the Report Through SDK Delivery and Verify Every Byte Against Its Completion
        let request = v1::ReportsDownloadExecutionReportRequest { execution_id: execution.clone() }.encode_to_vec();
        let mut report = session.watch_raw("/dokime.v1.Reports/DownloadExecutionReport", &request).await?;
        let mut bytes = Vec::new();
        let mut completion = None;
        let mut terminal = false;
        while let Some(frame) = report.next().await {
            let frame = v1::ReportsDownloadExecutionReportOutput::decode(frame?.as_slice())?;
            match frame.value {
                Some(v1::reports_download_execution_report_output::Value::Item(frame)) if !terminal => match frame.payload.and_then(|payload| payload.kind) {
                    Some(v1::report_html_frame_payload::Kind::Report(_)) if bytes.is_empty() => {}
                    Some(v1::report_html_frame_payload::Kind::Chunk(chunk)) if completion.is_none() => {
                        assert_eq!(chunk.offset, bytes.len() as u64);
                        bytes.extend(chunk.data);
                    }
                    Some(v1::report_html_frame_payload::Kind::Complete(value)) if completion.is_none() => completion = Some(value),
                    _ => return Err("Read Desktop Report: Invalid Transfer Sequence".into()),
                },
                Some(v1::reports_download_execution_report_output::Value::Response(_)) if !terminal => terminal = true,
                _ => return Err("Read Desktop Report: Invalid Terminal Sequence".into()),
            }
        }
        let completion = completion.ok_or("Read Desktop Report: Missing Verified Completion")?;
        assert!(terminal);
        assert_eq!(completion.size_bytes, bytes.len() as u64);
        let digest = sha2::Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        assert_eq!(completion.sha256, digest);

        // Export Through the Native Delivery Owner, Requiring Completion and Final Status
        let active = crate::connection::ConnectionSession::default();
        let connected = active.connect(crate::connection::ConnectRequest { target: crate::connection::ConnectionTarget::Local { endpoint: session.endpoint().to_owned() }, expected_backend_id: None }).await.map_err(|error| format!("{error:?}"))?;
        let lease = active.lease(&connected.generation).map_err(|error| format!("{error:?}"))?;
        let destination = root.path().join("native-export.html");
        let exported = crate::app::content::fetch_verified(&lease, &crate::app::desktop::ArtifactSource::ExecutionReport { execution_id: execution }, &destination, crate::rpc::protocol::FRAME_BYTES as usize).await?;
        assert_eq!(exported.size_bytes, completion.size_bytes);
        assert_eq!(exported.sha256, completion.sha256);
        assert_eq!(std::fs::read(destination)?, bytes);
        let destination = root.path().join("native-attachment.bin");
        let exported = crate::app::content::fetch_verified(&lease, &crate::app::desktop::ArtifactSource::Artifact { artifact_id: artifact.artifact_id }, &destination, crate::rpc::protocol::FRAME_BYTES as usize).await?;
        assert_eq!(exported.sha256, attachment_digest);
        assert_eq!(std::fs::read(destination)?, attachment);
        active.disconnect().map_err(|error| format!("{error:?}"))?;

        Ok::<(), Box<dyn std::error::Error>>(())
    }.await;
        let host_close = owner.close().await;
        let plugin_close = plugin.close().await;
        result?;
        host_close?;
        plugin_close?;
        Ok(())
    })
}

#[test]
fn desktop_owner_routes_canonical_ipc_and_replaces_generation() -> Result<(), Box<dyn std::error::Error>> {
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
        let suites: v1::ListSuitesReply =
            next.client().unary("/dokime.v1.Catalog/ListSuites", v1::CatalogListSuitesRequest::default()).await?;
        assert!(suites.suites.is_empty());
        owner.shutdown().await.map_err(|error| format!("{error:?}"))?;
        assert!(next.cancellation().is_cancelled());
        assert_eq!(owner.status().ownership, BackendOwnership::None);
        Ok(())
    })
}

#[test]
fn desktop_refuses_an_occupied_root_but_can_attach_to_its_owner() -> Result<(), Box<dyn std::error::Error>> {
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
fn desktop_attachment_preserves_an_incompatible_local_root() -> Result<(), Box<dyn std::error::Error>> {
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
fn library_trace_snapshot_watch_and_verified_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
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
