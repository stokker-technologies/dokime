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

#[tokio::test]
async fn thermal_catalog_execution_prompts_evidence_and_report() -> Result<(), Box<dyn std::error::Error>> {
    // Start the Prepared Shared Plugin and the Embedded Desktop Host
    let executable = std::env::var_os("DOKIME_THERMAL_PLUGIN").map(std::path::PathBuf::from).unwrap_or_else(|| {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../target/debug")
            .join(format!("dokime-rust-thermal{}", std::env::consts::EXE_SUFFIX))
    });
    if !executable.is_file() {
        return Err("Prepare Desktop Execution: Run mise run build:host-fixtures -- --language rust".into());
    }
    let configuration = settings();
    let mut plugin = dokime_runtime_next::PluginProcess::start(&executable, true, &configuration.host.placement).await?;
    let root = tempfile::tempdir()?;
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

        // Read the Report Through SDK Delivery and Verify Every Byte Against Its Completion
        let request = v1::ReportsDownloadExecutionReportRequest { execution_id: execution }.encode_to_vec();
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
        use sha2::Digest;
        let digest = sha2::Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        assert_eq!(completion.sha256, digest);
        Ok::<(), Box<dyn std::error::Error>>(())
    }.await;
    let host_close = owner.close().await;
    let plugin_close = plugin.close().await;
    result?;
    host_close?;
    plugin_close?;
    Ok(())
}
