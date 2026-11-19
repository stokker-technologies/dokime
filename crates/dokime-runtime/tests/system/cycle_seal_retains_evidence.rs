/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cycle Seal Finality and Retention

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::HashMap,
    error::Error,
    io::{Cursor, Read},
};

// Third Party Imports
use harmos_protocol::host::{PluginState, plugin_placement};
use harmos_protocol::model::FileMetadata;
use prost::Message;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use tonic::{Code, transport::Endpoint};
use zip::ZipArchive;

// Local Imports
use super::{client_runs_thermal_suite_over_grpc::HostProcess, documents_and_reports::decode_report, support};
use dokime::Client;
use dokime::client::{self, ContentArtifactsCreateOptions};
use dokime_protocol::host::evidence::{AnnotationOwner, AnnotationOwnerOwner, ExecutionNoteOwner, NoteWritable, annotation_owner_owner};
use dokime_protocol::host::{CyclePhase, ExecutionState, evidence_subject, execution_outcome, report_subject};
use dokime_protocol::model::{ArtifactScope, Metadata, Termination};
use dokime_runtime::PluginProcess;
use dokime_storage::{CycleArchive, ExecutionArchive};

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
fn cycle_seal_retains_evidence() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Admit Two Executions Without Starting Them.
            let root = support::tempdir()?;
            let credential_root = support::tempdir()?;
            let credentials = support::CredentialStore::start(credential_root.path()).await?;
            let settings = support::settings();
            let mut plugin = PluginProcess::start(&support::thermal_fixture()?, true, &settings.placement).await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, support::client_settings())?;
            let entry = client.plugins.create("thermal", client::PluginPlacement {
                kind: Some(plugin_placement::Kind::Remote(client::PluginRemote { url: plugin.endpoint().into() })),
            }, client::PluginValue { enabled: true }).await?.entry.unwrap();
            support::plugin_state(&client.plugins, entry, PluginState::Ready).await?;
            let cases = support::materialize(&client, "thermal", "thermal.cycle", HashMap::new()).await?;
            let origin = cases[0].origin.clone().unwrap();
            let cycle = client.cycles.create(vec![origin.clone(), origin], Metadata { label: "Final Evidence".into(), ..Default::default() }, false, "").await?.cycle.unwrap();
            let name = cycle.cycle_id.clone();
            let executions = client.executions.list(name.clone(), Default::default(), Default::default(), Default::default()).await?.executions;
            let running = executions[0].summary.as_ref().unwrap().execution_id.clone();
            let unstarted = executions[1].summary.as_ref().unwrap().execution_id.clone();

            // Accept a Note and an Attached File While the Execution Runs.
            client.executions.start(running.clone(), client::ExecutionMode::Automatic).await?;
            let mut progress = client.executions.progress.watch(running.clone()).await?;
            while let Some(frame) = progress.next().await? { if frame.phase == "prompt" { break; } }
            let execution_owner = AnnotationOwner { owner: Some(AnnotationOwnerOwner { kind: Some(annotation_owner_owner::Kind::Execution(ExecutionNoteOwner { cycle_id: name.clone(), execution_id: running.clone() })) }) };
            let before = client.evidence.notes.create(execution_owner.clone(), NoteWritable { message: "captured-before-seal".into() }).await?.note.unwrap();
            let bytes = b"execution-source";
            let file = support::native_file(&client, "source", &running, bytes).await?;
            let scope = ArtifactScope { cycle_id: Some(name.clone()), execution_id: Some(running.clone()), ..Default::default() };
            client.content.artifacts.create_with(file.clone(), "source.bin", "application/octet-stream", ContentArtifactsCreateOptions { scope: Some(scope.clone()), ..Default::default() }).await?;

            // Refuse Seal While Naming the Running Execution.
            let refusal = match client.cycles.seal(name.clone()).await {
                Err(source) => source,
                Ok(mut completion) => completion.next().await.unwrap_err(),
            };
            assert_eq!(refusal.code(), Code::FailedPrecondition);
            assert!(refusal.message().contains(&running));
            client.executions.cancel(running.clone()).await?;
            let mut sealed = false;
            while let Some(frame) = progress.next().await? { sealed |= frame.sealed; }
            assert!(sealed);

            // Refuse Execution Changes but Accept Cycle Evidence Until Cycle Seal.
            let refusal = client.evidence.notes.create(execution_owner, NoteWritable { message: "after".into() }).await.unwrap_err();
            assert!(refusal.message().contains(&running));
            assert!(support::native_file(&client, "late", &running, b"later").await.is_err());
            assert!(client.content.artifacts.create_with(file, "late.bin", "application/octet-stream", ContentArtifactsCreateOptions { scope: Some(scope), ..Default::default() }).await.is_err());
            let cycle_owner = AnnotationOwner { owner: Some(AnnotationOwnerOwner { kind: Some(annotation_owner_owner::Kind::CycleId(name.clone())) }) };
            let note = client.evidence.notes.create(cycle_owner.clone(), NoteWritable { message: "cycle-source".into() }).await?.note.unwrap();
            let data = b"cycle-source";
            let created = client.files.uploads.create(FileMetadata { id: "cycle-source".into(), scope: Some(format!("cycle/{name}")), media_type: "application/octet-stream".into(), expected_length: Some(data.len() as u64), expected_sha256: Some(Sha256::digest(data).to_vec()) }).await?;
            let upload = created.upload.unwrap();
            client.files.uploads.append(upload.clone(), 0, data.to_vec()).await?;
            let file = client.files.uploads.publish(upload, data.len() as u64, Sha256::digest(data).to_vec()).await?.upload.unwrap();
            let scope = ArtifactScope { cycle_id: Some(name.clone()), ..Default::default() };
            let artifact = client.content.artifacts.create_with(file.clone(), "cycle.bin", "application/octet-stream", ContentArtifactsCreateOptions { scope: Some(scope.clone()), ..Default::default() }).await?.artifact.unwrap();

            // Observe Materialization Followed by Its Retained Completion.
            let mut completion = client.cycles.seal(name.clone()).await?;
            let mut phases = Vec::new();
            let mut retained = None;
            while let Some(frame) = completion.next().await? { let cycle = frame.cycle.unwrap(); phases.push(cycle.phase); retained = Some(cycle); }
            assert_eq!(phases, [CyclePhase::Sealing as i32, CyclePhase::Sealed as i32]);
            let retained = retained.unwrap();
            assert!(retained.sealed);
            assert!(retained.seal_failure.is_none());
            assert_eq!(retained.result.as_ref().map(|result| (result.passed, result.failed)), Some((0, 0)));
            let not_run = client.executions.get(unstarted.clone()).await?.execution.unwrap();
            assert_eq!(not_run.summary.as_ref().unwrap().state, ExecutionState::Sealed as i32);
            assert!(!not_run.controls.unwrap().can_start);
            assert!(matches!(not_run.outcome.unwrap().attribution, Some(execution_outcome::Attribution::NotEvaluated(outcome)) if outcome.termination == Termination::NotRun as i32));
            let refusal = client.executions.start(unstarted, client::ExecutionMode::Automatic).await.unwrap_err();
            assert!(refusal.message().contains(&name));
            assert!(client.evidence.notes.create(cycle_owner, NoteWritable { message: "late".into() }).await.is_err());
            assert!(client.content.artifacts.create_with(file, "late-cycle.bin", "application/octet-stream", ContentArtifactsCreateOptions { scope: Some(scope), ..Default::default() }).await.is_err());
            assert!(client.files.uploads.create(FileMetadata { scope: Some(format!("cycle/{name}")), ..Default::default() }).await.is_err());

            // Two Downloads Return Each Retained File, Including the Report in the Archive.
            let report_subject = client::ReportSubject { kind: Some(report_subject::Kind::Cycle(name.clone())) };
            let evidence_subject = client::EvidenceSubject { kind: Some(evidence_subject::Kind::Cycle(name.clone())) };
            let mut reports = Vec::new();
            let mut archives = Vec::new();
            for _ in 0..2 {
                let report = client.reports.create(report_subject.clone(), "").await?.file.unwrap();
                assert_eq!(Some(&report), retained.report_file.as_ref());
                let archive = client.evidence.archive(evidence_subject.clone()).await?.file.unwrap();
                assert_eq!(Some(&archive), retained.evidence_file.as_ref());
                for (file, outputs) in [(report, &mut reports), (archive, &mut archives)] {
                    let mut bytes = Vec::new();
                    while (bytes.len() as u64) < file.length {
                        let part = client.files.read(file.clone(), bytes.len() as u64, 1024).await?;
                        assert!(!part.data.is_empty());
                        bytes.extend(part.data);
                    }
                    assert_eq!(Sha256::digest(&bytes).as_slice(), file.sha256);
                    outputs.push(bytes);
                }
            }
            assert_eq!(Sha256::digest(&reports[0]), Sha256::digest(&reports[1]));
            assert_eq!(Sha256::digest(&archives[0]), Sha256::digest(&archives[1]));
            let capture = decode_report(&reports[0])?;
            assert_eq!(capture.executions.len(), 2);
            assert!(capture.executions.iter().any(|execution| matches!(execution.detail.as_ref().unwrap().execution.as_ref().unwrap().outcome.as_ref().unwrap().attribution.as_ref(), Some(execution_outcome::Attribution::NotEvaluated(outcome)) if outcome.termination == Termination::NotRun as i32)));
            let mut archive = ZipArchive::new(Cursor::new(&archives[0]))?;
            let mut html = Vec::new();
            archive.by_name("report.html")?.read_to_end(&mut html)?;
            assert_eq!(html, reports[0]);
            let mut metadata = Vec::new();
            archive.by_name("evidence.pb")?.read_to_end(&mut metadata)?;
            let metadata = CycleArchive::decode(metadata.as_slice())?;
            assert!(metadata.notes.iter().any(|saved| saved.note_id == note.note_id));
            assert!(metadata.artifacts.iter().any(|bytes| dokime_storage::StoredArtifact::decode(bytes.as_slice()).unwrap().artifact.unwrap().artifact_id == artifact.artifact_id));
            let mut nested = Vec::new();
            let member = format!("executions/{running}.zip");
            archive.by_name(&member)?.read_to_end(&mut nested)?;
            let mut nested = ZipArchive::new(Cursor::new(nested))?;
            let mut metadata = Vec::new();
            nested.by_name("evidence.pb")?.read_to_end(&mut metadata)?;
            let metadata = ExecutionArchive::decode(metadata.as_slice())?;
            assert!(metadata.notes.iter().any(|saved| saved.note_id == before.note_id));
            assert_eq!(metadata.artifacts.len(), 1);

            // Retained Downloads Survive Host Restart.
            process.close().await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let client = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, support::client_settings())?;
            assert_eq!(client.cycles.get(name.clone()).await?.cycle.unwrap().evidence_file, retained.evidence_file);
            let mut replay = client.cycles.seal(name).await?;
            assert_eq!(replay.next().await?.unwrap().cycle.unwrap().phase, CyclePhase::Sealed as i32);
            assert!(replay.next().await?.is_none());
            process.close().await?;
            plugin.close().await?;
            Ok(())
        }).await?
    }))
}

#[test]
fn cycle_seal_retries_materialization_failure() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Own an Empty Cycle and Its Native Storage Fault Independently of Retention Proof.
            let root = support::tempdir()?;
            let credential_root = support::tempdir()?;
            let credentials = support::CredentialStore::start(credential_root.path()).await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, support::client_settings())?;
            let cycle = client.cycles.create(Vec::new(), Metadata { label: "Seal Failure".into(), ..Default::default() }, false, "").await?.cycle.unwrap();
            let name = cycle.cycle_id;
            let cycle_owner = AnnotationOwner { owner: Some(AnnotationOwnerOwner { kind: Some(annotation_owner_owner::Kind::CycleId(name.clone())) }) };
            // Inject a Durable Publication Failure Without Timing or Deadline Changes.
            let database = Connection::open(root.path().join("storage.sqlite3"))?;
            database.execute_batch("CREATE TRIGGER refuse_seal_report BEFORE INSERT ON storage_files WHEN NEW.id LIKE 'prepared-report/cycle-seal/%' BEGIN SELECT RAISE(ABORT,'Injected Report Publication Failure'); END;")?;
            let mut completion = client.cycles.seal(name.clone()).await?;
            let mut phases = Vec::new();
            while let Some(frame) = completion.next().await? { phases.push(frame.cycle.unwrap().phase); }
            assert_eq!(phases, [CyclePhase::Sealing as i32, CyclePhase::SealFailed as i32]);
            let failed = client.cycles.get(name.clone()).await?.cycle.unwrap();
            assert!(!failed.sealed);
            assert!(failed.seal_failure.is_some());
            client.evidence.notes.create(cycle_owner.clone(), NoteWritable { message: "repair-reviewed".into() }).await?;
            database.execute_batch("DROP TRIGGER refuse_seal_report;")?;

            // Retry the Same Unsealed Cycle After Resolving Its Materialization Fault.
            let mut completion = client.cycles.seal(name.clone()).await?;
            let mut phases = Vec::new();
            while let Some(frame) = completion.next().await? { phases.push(frame.cycle.unwrap().phase); }
            assert_eq!(phases, [CyclePhase::Sealing as i32, CyclePhase::Sealed as i32]);
            assert!(client.cycles.get(name).await?.cycle.unwrap().sealed);
            process.close().await?;
            Ok(())
        }).await?
    }))
}
