/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Transfers Execution Content

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use sha2::{Digest, Sha256};

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::v1;
use dokime_storage::{Execution, Store};

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

#[tokio::test]
async fn client_transfers_execution_content() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Prepare Registered Files and an Independent Execution
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        for id in ["content-a", "content-b"] {
            store.create(&Execution { identifier: id.into(), cycle: "content-cycle".into(),
                origin: Some(v1::Origin { kind: Some(v1::origin::Kind::Import(v1::Import::default())) }), ..Default::default() })?;
        }
        let file = v1::ExecutionFileMetadata { relative_path: "logs/output.txt".into(), content_type: Some("text/plain".into()),
            kind: v1::ExecutionFileKind::Text as i32, state: v1::ExecutionFileState::Streaming as i32, ..Default::default() };
        store.register_execution_file("file-original", "content-a", file.clone())?;
        store.append_content("file-original", 0, b"original\n")?;
        store.finish_execution_file("file-original", false)?;
        store.register_execution_file("file-replacement", "content-a", file)?;
        store.append_content("file-replacement", 0, b"replacement\n")?;
        store.finish_execution_file("file-replacement", false)?;

        // Upload Ordered Chunks with an Explicit Digest Completion
        let process = HostProcess::start(root.path()).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let mut content = v1::content_client::ContentClient::new(channel);
        let bytes = vec![42; 140_003];
        let sha256 = Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        let header = v1::UploadArtifactHeader { scope: Some(v1::ArtifactScope { cycle_id: Some("content-cycle".into()), execution_id: Some("content-a".into()), ..Default::default() }),
            filename: "capture.bin".into(), size_bytes: bytes.len() as u64, sha256: Some(sha256.clone()), ..Default::default() };
        let mut input = vec![v1::ContentUploadArtifactInput { value: Some(v1::content_upload_artifact_input::Value::Request(v1::ContentUploadArtifactRequest {})) },
            v1::ContentUploadArtifactInput { value: Some(v1::content_upload_artifact_input::Value::Item(v1::UploadArtifactFrame { payload: Some(v1::UploadArtifactFramePayload {
                kind: Some(v1::upload_artifact_frame_payload::Kind::Header(header.clone())) }) })) }];
        for (index, chunk) in bytes.chunks(64 * 1024).enumerate() {
            input.push(v1::ContentUploadArtifactInput { value: Some(v1::content_upload_artifact_input::Value::Item(v1::UploadArtifactFrame { payload: Some(v1::UploadArtifactFramePayload {
                kind: Some(v1::upload_artifact_frame_payload::Kind::Chunk(v1::ArtifactChunk { offset: (index * 64 * 1024) as u64, data: chunk.to_vec() })) }) })) });
        }
        let mut missing_completion = input.clone();
        assert!(content.upload_artifact(tokio_stream::iter(missing_completion.drain(..).collect::<Vec<_>>())).await.is_err());
        input.push(v1::ContentUploadArtifactInput { value: Some(v1::content_upload_artifact_input::Value::Item(v1::UploadArtifactFrame { payload: Some(v1::UploadArtifactFramePayload {
            kind: Some(v1::upload_artifact_frame_payload::Kind::Complete(v1::ArtifactCompletion { size_bytes: bytes.len() as u64, sha256: sha256.clone() })) }) })) });
        let artifact = content.upload_artifact(tokio_stream::iter(input)).await?.into_inner();
        assert_eq!(artifact.sha256, sha256);
        let page = content.list_artifacts(v1::ContentListArtifactsRequest::default()).await?.into_inner();
        assert_eq!(page.artifacts, vec![artifact.clone()]);

        // Cancel a Staged Run Upload Without Exposing Its Prefix
        let input=vec![
            v1::ContentUploadArtifactRunInput {value:Some(v1::content_upload_artifact_run_input::Value::Request(v1::ContentUploadArtifactRequest {}))},
            v1::ContentUploadArtifactRunInput {value:Some(v1::content_upload_artifact_run_input::Value::Item(v1::UploadArtifactFrame {payload:Some(v1::UploadArtifactFramePayload {kind:Some(v1::upload_artifact_frame_payload::Kind::Header(header))})}))},
            v1::ContentUploadArtifactRunInput {value:Some(v1::content_upload_artifact_run_input::Value::Cancel(v1::ClientEmpty {}))},
        ];
        assert_eq!(content.run_upload_artifact(tokio_stream::iter(input)).await.unwrap_err().code(),tonic::Code::Cancelled);
        assert_eq!(content.list_artifacts(v1::ContentListArtifactsRequest::default()).await?.into_inner().artifacts.len(),1);

        // Verify Metadata, Contiguous Download Chunks, Digest and Final Status
        let mut download = content.download_artifact(v1::ContentDownloadArtifactRequest { artifact_id: artifact.artifact_id.clone() }).await?.into_inner();
        let mut downloaded = Vec::new();
        let mut complete = false;
        let mut terminal = false;
        while let Some(frame) = download.message().await? {
            match frame.value.unwrap() {
                v1::content_download_artifact_output::Value::Item(frame) => match frame.payload.unwrap().kind.unwrap() {
                    v1::artifact_frame_payload::Kind::Metadata(metadata) => assert_eq!(metadata, artifact),
                    v1::artifact_frame_payload::Kind::Chunk(chunk) => { assert_eq!(chunk.offset, downloaded.len() as u64); assert!(chunk.data.len() <= 64 * 1024); downloaded.extend(chunk.data); }
                    v1::artifact_frame_payload::Kind::Complete(value) => { assert_eq!(value.sha256, sha256); assert_eq!(value.size_bytes, bytes.len() as u64); complete = true; }
                },
                v1::content_download_artifact_output::Value::Response(_) => { assert!(complete); terminal = true; }
            }
        }
        assert!(terminal);
        assert_eq!(downloaded, bytes);

        // Finish Run Input While Draining the Complete Download
        let request=v1::ContentDownloadArtifactRunInput {value:Some(v1::content_download_artifact_run_input::Value::Request(v1::ContentDownloadArtifactRequest {artifact_id:artifact.artifact_id.clone()}))};
        let finish=v1::ContentDownloadArtifactRunInput {value:Some(v1::content_download_artifact_run_input::Value::Finish(v1::ClientEmpty {}))};
        let mut run=content.run_download_artifact(tokio_stream::iter([request,finish])).await?.into_inner();
        let mut run_complete=false;
        let mut run_terminal=false;
        while let Some(frame)=run.message().await? {
            match frame.value.unwrap() {
                v1::content_download_artifact_run_output::Value::Item(frame)=>{if matches!(frame.payload.and_then(|payload|payload.kind),Some(v1::artifact_frame_payload::Kind::Complete(_))){run_complete=true;}},
                v1::content_download_artifact_run_output::Value::Response(_)=>{assert!(run_complete);run_terminal=true;}
            }
        }
        assert!(run_terminal);

        // List Logical Files, Clamp a Window and Reject Arbitrary Paths
        let files = content.list_execution_files(v1::ContentListExecutionFilesRequest { execution_id: "content-a".into(), ..Default::default() }).await?.into_inner();
        assert_eq!(files.files.len(), 1);
        let window = content.read_execution_file(v1::ContentReadExecutionFileRequest { execution_id: "content-a".into(), relative_path: "logs/output.txt".into(), offset: 3, max_bytes: 4 }).await?.into_inner();
        assert_eq!(window.data, b"lace");
        let window = content.read_execution_file(v1::ContentReadExecutionFileRequest { execution_id: "content-a".into(), relative_path: "logs/output.txt".into(), offset: 99, max_bytes: 4 }).await?.into_inner();
        assert_eq!(window.offset, 12);
        assert!(window.data.is_empty());
        assert!(content.read_execution_file(v1::ContentReadExecutionFileRequest { execution_id: "content-b".into(), relative_path: "logs/output.txt".into(), ..Default::default() }).await.is_err());
        assert!(content.read_execution_file(v1::ContentReadExecutionFileRequest { execution_id: "content-a".into(), relative_path: "../execution.sqlite3".into(), ..Default::default() }).await.is_err());

        // Follow the Original Retained Identity and Require It When Resuming
        let request = v1::ContentFollowExecutionFileRequest { execution_id: "content-a".into(), relative_path: "logs/output.txt".into(), ..Default::default() };
        let mut follow = content.follow_execution_file(request.clone()).await?.into_inner();
        let mut identity = None;
        let mut followed = Vec::new();
        let mut final_length = None;
        while let Some(frame) = follow.message().await? {
            if let Some(v1::content_follow_execution_file_output::Value::Item(frame)) = frame.value {
                match frame.payload.unwrap().kind.unwrap() {
                    v1::execution_file_follow_frame_payload::Kind::Metadata(metadata) => identity = metadata.identity,
                    v1::execution_file_follow_frame_payload::Kind::Chunk(chunk) => { assert_eq!(chunk.offset, followed.len() as u64); followed.extend(chunk.data); }
                    v1::execution_file_follow_frame_payload::Kind::Terminal(terminal) => final_length = Some(terminal.length),
                }
            }
        }
        assert_eq!(followed, b"replacement\n");
        assert_eq!(final_length, Some(12));
        assert!(content.follow_execution_file(v1::ContentFollowExecutionFileRequest { offset: 4, ..request.clone() }).await.is_err());
        let mut resumed = content.follow_execution_file(v1::ContentFollowExecutionFileRequest { offset: 4, identity, ..request }).await?.into_inner();
        while resumed.message().await?.is_some() {}

        // Run Unary Plugin Artifact Calls with Replacement, Replay and Abort
        let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/examples").join(format!("content_fixture{}",std::env::consts::EXE_SUFFIX));
        let settings=super::support::settings();
        let mut plugin=dokime_runtime_next::PluginProcess::start(&fixture,true,&settings.placement).await?;
        let channel=tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client=dokime::client::HostClient::new(channel);
        client.plugins.add(plugin.endpoint()).await?;
        let cases=client.templates.materialize("content", "content.capture",std::collections::HashMap::new()).await?;
        let cycle=client.cycles.start(cases).await?;
        let execution=cycle.executions[0].clone();
        let mut progress=cycle.watch();
        while progress.next().await?.is_some() {}
        let detail=client.executions.get(execution.clone()).await?;
        assert_eq!(detail.summary.unwrap().state,v1::ExecutionState::Sealed as i32);
        let artifacts=content.list_artifacts(v1::ContentListArtifactsRequest::default()).await?.into_inner().artifacts;
        assert_eq!(artifacts.len(),2);
        let produced=artifacts.iter().find(|artifact|artifact.filename=="producer.txt").unwrap();
        assert_eq!(produced.size_bytes,12);
        let mut download=content.download_artifact(v1::ContentDownloadArtifactRequest {artifact_id:produced.artifact_id.clone()}).await?.into_inner();
        let mut produced_bytes=Vec::new();
        while let Some(frame)=download.message().await? {
            if let Some(v1::content_download_artifact_output::Value::Item(frame))=frame.value {
                if let Some(v1::artifact_frame_payload::Kind::Chunk(chunk))=frame.payload.and_then(|payload|payload.kind){produced_bytes.extend(chunk.data);}
            }
        }
        assert_eq!(produced_bytes,b"replacement\n");

        // Download a Sealed Evidence Archive Through Ordered Frames
        let mut evidence=content.download_evidence(v1::ContentDownloadEvidenceRequest {subject:Some(v1::EvidenceSubject {kind:Some(v1::evidence_subject::Kind::Execution(execution.clone()))})}).await?.into_inner();
        let mut archive=Vec::new();
        let mut evidence_complete=false;
        let mut evidence_terminal=false;
        while let Some(frame)=evidence.message().await? {
            match frame.value.unwrap() {
                v1::content_download_evidence_output::Value::Item(frame)=>match frame.payload.unwrap().kind.unwrap() {
                    v1::evidence_export_payload::Kind::Metadata(_)=>(),
                    v1::evidence_export_payload::Kind::Chunk(chunk)=>{assert_eq!(chunk.offset,archive.len() as u64);archive.extend(chunk.data);},
                    v1::evidence_export_payload::Kind::Complete(complete)=>{assert_eq!(complete.size_bytes,archive.len() as u64);evidence_complete=true;}
                },
                v1::content_download_evidence_output::Value::Response(_)=>{assert!(evidence_complete);evidence_terminal=true;}
            }
        }
        assert!(evidence_terminal);
        use dokime::encoding::Message;
        let archive=dokime_storage::ExecutionArchive::decode(archive.as_slice())?;
        assert_eq!(archive.execution.unwrap().identifier,execution);
        assert!(archive.content.iter().any(|content|content.identifier==produced.artifact_id));
        assert_eq!(archive.content_chunks.iter().filter(|chunk|chunk.identifier==produced.artifact_id).flat_map(|chunk|chunk.payload.clone()).collect::<Vec<_>>(),b"replacement\n");

        // Reopen Published Content Without Exposing Abandoned Uploads
        process.close().await?;
        plugin.close().await?;
        let process = HostProcess::start(root.path()).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let mut content = v1::content_client::ContentClient::new(channel);
        assert_eq!(content.list_artifacts(v1::ContentListArtifactsRequest::default()).await?.into_inner().artifacts, artifacts);
        process.close().await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    }).await?
}
