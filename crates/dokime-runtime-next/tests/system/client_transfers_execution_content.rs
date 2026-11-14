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
use dokime_protocol::client::EvidenceArchiveRequest;
use dokime_protocol::client::EvidenceSubject;
use dokime_protocol::client::FilesDeleteRequest;
use dokime_protocol::client::FilesGetRequest;
use dokime_protocol::client::FilesReadRequest;
use dokime_protocol::client::content::ArtifactsCreateRequest;
use dokime_protocol::client::content::ArtifactsGetRequest;
use dokime_protocol::client::content::ArtifactsListRequest;
use dokime_protocol::client::content::ExecutionFileKind;
use dokime_protocol::client::content::ExecutionFileMetadata;
use dokime_protocol::client::content::ExecutionFileState;
use dokime_protocol::client::content::ExecutionFilesGetRequest;
use dokime_protocol::client::content::ExecutionFilesListRequest;
use dokime_protocol::client::content::artifacts_client::ArtifactsClient;
use dokime_protocol::client::content::execution_files_client::ExecutionFilesClient;
use dokime_protocol::client::evidence_client::EvidenceClient;
use dokime_protocol::client::evidence_subject::Kind as EvidenceSubjectKind;
use dokime_protocol::client::files::UploadsAppendRequest;
use dokime_protocol::client::files::UploadsCreateRequest;
use dokime_protocol::client::files::UploadsGetRequest;
use dokime_protocol::client::files::UploadsPublishRequest;
use dokime_protocol::client::files::UploadsResumeRequest;
use dokime_protocol::client::files::uploads_client::UploadsClient;
use dokime_protocol::client::files_client::FilesClient;
use dokime_storage::CONTENT_CHUNK_BYTES;
use dokime_storage::{Execution, Store};
use harmos_protocol::storage::FileMetadata;
use harmos_protocol::storage::StagedFile;
use harmos_protocol::v1::PageRequest;
use std::error::Error;
use tonic::transport::Endpoint;

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
async fn client_transfers_execution_content() -> Result<(), Box<dyn Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Prepare Registered Files and an Independent Execution
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let store = Store::open(root.path())?;
        for id in ["content-a", "content-b"] {
            store.create(&Execution {
                identifier: id.into(),
                cycle: "content-cycle".into(),
                origin: Some(v1::Origin {
                    kind: Some(v1::origin::Kind::Import(v1::Import::default())),
                }),
                ..Default::default()
            })?;
        }
        let file = ExecutionFileMetadata {
            relative_path: "logs/output.txt".into(),
            content_type: Some("text/plain".into()),
            kind: ExecutionFileKind::Text as i32,
            state: ExecutionFileState::Streaming as i32,
            ..Default::default()
        };
        store.register_execution_file("file-original", "content-a", file.clone())?;
        store.append_content("file-original", 0, b"original\n")?;
        store.finish_execution_file("file-original", false)?;
        store.register_execution_file("file-replacement", "content-a", file)?;
        store.append_content("file-replacement", 0, b"replacement\n")?;
        store.finish_execution_file("file-replacement", false)?;

        let bytes = vec![42; 140003];
        let digest = Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        store.begin_artifact(v1::ArtifactMetadata {
            artifact_id: "uploaded".into(),
            scope: Some(v1::ArtifactScope {
                execution_id: Some("content-a".into()),
                cycle_id: Some("content-cycle".into()),
                ..Default::default()
            }),
            filename: "capture.bin".into(),
            size_bytes: bytes.len() as u64,
            sha256: digest.clone(),
            ..Default::default()
        })?;
        for (index, chunk) in bytes.chunks(CONTENT_CHUNK_BYTES).enumerate() {
            store.append_content("uploaded", (index * CONTENT_CHUNK_BYTES) as u64, chunk)?;
        }
        store.finish_artifact("uploaded", bytes.len() as u64, &digest)?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
        let mut artifacts = ArtifactsClient::new(channel.clone());
        let mut files = ExecutionFilesClient::new(channel);
        let first = artifacts
            .list(ArtifactsListRequest {
                page: Some(PageRequest { page_size: 1, ..Default::default() }),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(first.artifacts.len(), 1);
        let uploaded = artifacts
            .get(ArtifactsGetRequest {
                name: "uploaded".into(),
                ..Default::default()
            })
            .await?
            .into_inner();
        let reference = uploaded.file.unwrap();
        assert_eq!(reference.length, bytes.len() as u64);
        assert_eq!(reference.sha256, Sha256::digest(&bytes).as_slice());
        let request = ArtifactsCreateRequest {
            file: Some(reference.clone()),
            scope: Some(v1::ArtifactScope {
                execution_id: Some("content-a".into()),
                cycle_id: Some("content-cycle".into()),
                ..Default::default()
            }),
            filename: "attached.bin".into(),
            content_type: "application/octet-stream".into(),
            request_id: "attach".into(),
        };
        let attached = artifacts.create(request.clone()).await?.into_inner();
        assert_eq!(artifacts.create(request.clone()).await?.into_inner(), attached);
        let mut foreign = request;
        foreign.scope.as_mut().unwrap().execution_id = Some("content-b".into());
        assert!(artifacts.create(foreign).await.is_err());
        let file = files
            .get(ExecutionFilesGetRequest {
                parent: "content-a".into(),
                name: "logs/output.txt".into(),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(file.metadata.as_ref().unwrap().size_bytes, 12);
        let file = file.file.unwrap();
        assert_eq!(store.content_window(&file.metadata.as_ref().unwrap().id, 0, 64)?, b"replacement\n");
        assert_eq!(store.content_window("file-original", 0, 64)?, b"original\n");
        assert!(
            files
                .get(ExecutionFilesGetRequest {
                    parent: "content-b".into(),
                    name: "logs/output.txt".into(),
                    ..Default::default()
                })
                .await
                .is_err()
        );
        assert!(
            files
                .get(ExecutionFilesGetRequest {
                    parent: "content-a".into(),
                    name: "../logs/output.txt".into(),
                    ..Default::default()
                })
                .await
                .is_err()
        );
        let listed = files
            .list(ExecutionFilesListRequest {
                parent: "content-a".into(),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(listed.files.len(), 1);
        let attached = attached.artifact.unwrap();
        let stored = store.content(&attached.artifact_id)?;
        assert_eq!(stored.length, bytes.len() as u64);
        let mut actual = Vec::new();
        while actual.len() < bytes.len() {
            actual.extend(store.content_window(&stored.identifier, actual.len() as u64, CONTENT_CHUNK_BYTES)?);
        }
        assert_eq!(actual, bytes);
        // Resume a Lost-Reply Upload Without Reapplying Its Chunks.
        let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
        let mut uploads = UploadsClient::new(channel.clone());
        let mut file_client = FilesClient::new(channel);
        let opening = UploadsCreateRequest {
            metadata: Some(FileMetadata {
                scope: Some("content-a".into()),
                media_type: "application/octet-stream".into(),
                expected_length: Some(bytes.len() as u64),
                ..Default::default()
            }),
            request_id: "resumable-file".into(),
        };
        let created = uploads.create(opening.clone()).await?.into_inner();
        assert_eq!(uploads.create(opening).await?.into_inner(), created);
        let original_session = created.production.unwrap().session.unwrap();
        let split = bytes.len().min(created.chunk_bytes as usize).min(17);
        let append = UploadsAppendRequest {
            session: Some(original_session.clone()),
            offset: 0,
            data: bytes[..split].to_vec(),
            request_id: "first-chunk".into(),
        };
        let first_chunk = uploads.append(append.clone()).await?.into_inner();
        assert_eq!(uploads.append(append.clone()).await?.into_inner(), first_chunk);
        let mut changed = append.clone();
        changed.data[0] ^= 1;
        assert!(uploads.append(changed).await.is_err());
        let staged = StagedFile {
            history: original_session.history.clone(),
            id: original_session.id.clone(),
        };
        let progress = uploads.get(UploadsGetRequest { file: Some(staged.clone()), at: None }).await?.into_inner();
        assert_eq!(progress.accepted_length, split as u64);
        assert_eq!(progress.durable_length, split as u64);
        let resumed = uploads
            .resume(UploadsResumeRequest {
                session: Some(original_session.clone()),
                request_id: "resume-file".into(),
            })
            .await?
            .into_inner();
        let session = resumed.production.unwrap().session.unwrap();
        let stale = UploadsAppendRequest {
            session: Some(original_session),
            offset: split as u64,
            data: vec![1],
            request_id: "stale-writer".into(),
        };
        assert!(uploads.append(stale).await.is_err());
        let mut offset = split;
        while offset < bytes.len() {
            let end = bytes.len().min(offset + resumed.chunk_bytes as usize);
            uploads
                .append(UploadsAppendRequest {
                    session: Some(session.clone()),
                    offset: offset as u64,
                    data: bytes[offset..end].to_vec(),
                    request_id: format!("remaining-{offset}"),
                })
                .await?;
            offset = end;
        }
        let publish = UploadsPublishRequest {
            session: Some(session),
            length: bytes.len() as u64,
            sha256: Sha256::digest(&bytes).to_vec(),
            request_id: "publish-file".into(),
        };
        let published = uploads.publish(publish.clone()).await?.into_inner();
        assert_eq!(uploads.publish(publish).await?.into_inner(), published);
        let selected = file_client.get(FilesGetRequest { file: published.file, at: None }).await?.into_inner();
        let mut read = Vec::new();
        while read.len() < bytes.len() {
            let page = file_client
                .read(FilesReadRequest {
                    file: selected.file.clone(),
                    offset: read.len() as u64,
                    max_bytes: selected.chunk_bytes,
                    at: selected.resolved_at.clone(),
                })
                .await?
                .into_inner();
            assert_eq!(page.resolved_at, selected.resolved_at);
            assert!(!page.data.is_empty());
            read.extend(page.data);
        }
        assert_eq!(read, bytes);
        // Follow a Growing File and Drain Its Final Append Before Completion.
        store.register_execution_file(
            "growing",
            "content-a",
            ExecutionFileMetadata {
                relative_path: "growing.txt".into(),
                state: ExecutionFileState::Streaming as i32,
                ..Default::default()
            },
        )?;
        store.append_content("growing", 0, b"first")?;
        let growing = file_client
            .get(FilesGetRequest {
                file: Some(harmos_protocol::storage::FileReference {
                    metadata: Some(harmos_protocol::storage::FileMetadata {
                        id: "growing".into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                at: None,
            })
            .await?
            .into_inner();
        let mut watch =
            file_client.watch(dokime_protocol::client::FilesWatchRequest { file: growing.file, offset: 0, at: None }).await?.into_inner();
        let first = watch.message().await?.unwrap().data.unwrap();
        assert_eq!(first.bytes, b"first");
        store.append_content("growing", 5, b"last")?;
        store.finish_execution_file("growing", false)?;
        let mut followed = first.bytes;
        let mut completed = false;
        while let Some(frame) = watch.message().await? {
            if let Some(data) = frame.data {
                assert_eq!(data.offset, followed.len() as u64);
                followed.extend(data.bytes);
            }
            if let Some(completion) = frame.completion {
                assert_eq!(completion.bytes, followed.len() as u64);
                completed = true;
            }
        }
        assert!(completed);
        assert_eq!(followed, b"firstlast");
        // The SDK Convenience Publishes an Unscoped Host File with the Same Resource.
        let upload_path = root.path().join("client-upload.bin");
        std::fs::write(&upload_path, &bytes)?;
        let sdk = dokime::client::HostClient::new(
            Endpoint::from_shared(process.url.clone())?.connect().await?,
            super::support::client_settings(),
        )?;
        let uploaded = sdk.files.upload(&upload_path).await?;
        assert_eq!(uploaded.sha256, Sha256::digest(&bytes).to_vec());
        let deleted = FilesDeleteRequest {
            file: Some(uploaded),
            request_id: "delete-file".into(),
        };
        let receipt = file_client.delete(deleted.clone()).await?.into_inner();
        assert_eq!(file_client.delete(deleted).await?.into_inner(), receipt);
        store.close_streams("content-a")?;
        store.close("content-a", v1::Termination::Completed, v1::ExecutionOutcome::default())?;
        store.seal("content-a")?;
        let mut evidence = EvidenceClient::connect(process.url.clone()).await?;
        let archived = evidence
            .archive(EvidenceArchiveRequest {
                subject: Some(EvidenceSubject {
                    kind: Some(EvidenceSubjectKind::Execution("content-a".into())),
                }),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert!(archived.file.unwrap().length > bytes.len() as u64);
        process.close().await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let mut artifacts = ArtifactsClient::connect(process.url.clone()).await?;
        let restored = artifacts
            .get(ArtifactsGetRequest {
                name: attached.artifact_id.clone(),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(restored.metadata, Some(attached));
        assert_eq!(restored.file.unwrap().sha256, reference.sha256);
        process.close().await?;
        Ok::<_, Box<dyn Error>>(())
    })
    .await?
}
