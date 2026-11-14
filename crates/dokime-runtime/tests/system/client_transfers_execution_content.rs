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

use dokime::client::FilesWatchOptions;

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::ArtifactMetadata;
use dokime::ArtifactScope;
use dokime::Client;
use dokime::Import;
use dokime::Origin;
use dokime::Termination;
use dokime::client::ContentArtifactsCreateOptions;
use dokime::client::ContentArtifactsListOptions;
use dokime::client::ContentExecutionFileKind;
use dokime::client::ContentExecutionFileMetadata;
use dokime::client::ContentExecutionFileState;
use dokime::client::EvidenceSubject;
use dokime::client::ExecutionOutcome;
use dokime::client::FilesDeleteOptions;
use dokime::client::FilesReadOptions;
use dokime::client::FilesUploadsAppendOptions;
use dokime::client::FilesUploadsCreateOptions;
use dokime::client::FilesUploadsPublishOptions;
use dokime::client::FilesUploadsResumeOptions;
use dokime_protocol::host::FilesDeleteRequest;
use dokime_protocol::host::content::ArtifactsCreateRequest;
use dokime_protocol::host::evidence_subject::Kind as EvidenceSubjectKind;
use dokime_protocol::model::origin::Kind as OriginKind;
use dokime_storage::CONTENT_CHUNK_BYTES;
use dokime_storage::Execution;
use dokime_storage::Store;
use harmos_protocol::model::FileMetadata;
use harmos_protocol::model::PageRequest;
use sha2::Digest;
use sha2::Sha256;
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
                origin: Some(Origin {
                    kind: Some(OriginKind::Import(Import::default())),
                }),
                ..Default::default()
            })?;
        }
        let file = ContentExecutionFileMetadata {
            relative_path: "logs/output.txt".into(),
            content_type: Some("text/plain".into()),
            kind: ContentExecutionFileKind::Text as i32,
            state: ContentExecutionFileState::Streaming as i32,
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
        store.begin_artifact(ArtifactMetadata {
            artifact_id: "uploaded".into(),
            scope: Some(ArtifactScope {
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
        let artifacts = Client::from_channel(channel.clone(), super::support::client_settings())?.content.artifacts.clone();
        let files = Client::from_channel(channel, super::support::client_settings())?.content.execution_files.clone();
        let first = artifacts
            .list_with(
                Default::default(),
                ContentArtifactsListOptions {
                    page: Some(PageRequest { page_size: 1, ..Default::default() }),
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(first.artifacts.len(), 1);
        let uploaded = artifacts.get("uploaded").await?;
        let reference = uploaded.file.unwrap();
        assert_eq!(reference.length, bytes.len() as u64);
        assert_eq!(reference.sha256, Sha256::digest(&bytes).as_slice());
        let request = ArtifactsCreateRequest {
            file: Some(reference.clone()),
            scope: Some(ArtifactScope {
                execution_id: Some("content-a".into()),
                cycle_id: Some("content-cycle".into()),
                ..Default::default()
            }),
            filename: "attached.bin".into(),
            content_type: "application/octet-stream".into(),
            request_id: "attach".into(),
            metadata: None,
        };
        let attached = artifacts
            .create_with(
                ((request.clone()).file).unwrap_or_default(),
                ((request.clone()).scope).unwrap_or_default(),
                (request.clone()).filename,
                (request.clone()).content_type,
                ContentArtifactsCreateOptions {
                    request_id: Some((request.clone()).request_id),
                    metadata: (request.clone()).metadata,
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(
            artifacts
                .create_with(
                    ((request.clone()).file).unwrap_or_default(),
                    ((request.clone()).scope).unwrap_or_default(),
                    (request.clone()).filename,
                    (request.clone()).content_type,
                    ContentArtifactsCreateOptions {
                        request_id: Some((request.clone()).request_id),
                        metadata: (request.clone()).metadata,
                        ..Default::default()
                    }
                )
                .await?,
            attached
        );
        let mut foreign = request;
        foreign.scope.as_mut().unwrap().execution_id = Some("content-b".into());
        assert!(
            artifacts
                .create_with(
                    ((foreign.clone()).file).unwrap_or_default(),
                    ((foreign.clone()).scope).unwrap_or_default(),
                    (foreign.clone()).filename,
                    (foreign.clone()).content_type,
                    ContentArtifactsCreateOptions {
                        request_id: Some((foreign.clone()).request_id),
                        metadata: (foreign.clone()).metadata,
                        ..Default::default()
                    }
                )
                .await
                .is_err()
        );
        let file = files.get("content-a", "logs/output.txt").await?;
        assert_eq!(file.metadata.as_ref().unwrap().size_bytes, 12);
        let file = file.file.unwrap();
        assert_eq!(store.content_window(&file.metadata.as_ref().unwrap().id, 0, 64)?, b"replacement\n");
        assert_eq!(store.content_window("file-original", 0, 64)?, b"original\n");
        assert!(files.get("content-b", "logs/output.txt").await.is_err());
        assert!(files.get("content-a", "../logs/output.txt").await.is_err());
        let listed = files.list("content-a").await?;
        assert_eq!(listed.files.len(), 1);
        let attached = attached.artifact.unwrap();
        let stored = store.content(&attached.artifact_id)?;
        assert_eq!(stored.length, bytes.len() as u64);
        let mut actual = Vec::new();
        while actual.len() < bytes.len() {
            actual.extend(store.content_window(&stored.identifier, actual.len() as u64, CONTENT_CHUNK_BYTES)?);
        }
        assert_eq!(actual, bytes);
        // Replay Upload Creation and a Lost Chunk Reply Without Appending Twice
        let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = Client::from_channel(channel, super::support::client_settings())?;
        let metadata = FileMetadata {
            scope: Some("content-a".into()),
            media_type: "application/octet-stream".into(),
            expected_length: Some(bytes.len() as u64),
            ..Default::default()
        };
        let options = FilesUploadsCreateOptions {
            request_id: Some("resumable-file".into()),
            ..Default::default()
        };
        let created = client
            .files
            .uploads
            .create_with(
                metadata.clone(),
                FilesUploadsCreateOptions {
                    request_id: options.request_id.clone(),
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(client.files.uploads.create_with(metadata, options).await?, created);
        let original = created.upload.unwrap();
        let split = bytes.len().min(created.chunk_bytes as usize).min(17);
        let first_bytes = bytes[..split].to_vec();
        let options = FilesUploadsAppendOptions {
            request_id: Some("first-chunk".into()),
            ..Default::default()
        };
        let first = client
            .files
            .uploads
            .append_with(
                original.clone(),
                0,
                first_bytes.clone(),
                FilesUploadsAppendOptions {
                    request_id: options.request_id.clone(),
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(client.files.uploads.append_with(original.clone(), 0, first_bytes.clone(), options).await?, first);
        let mut changed = first_bytes;
        changed[0] ^= 1;
        let options = FilesUploadsAppendOptions {
            request_id: Some("first-chunk".into()),
            ..Default::default()
        };
        assert!(client.files.uploads.append_with(original.clone(), 0, changed, options).await.is_err());
        let progress = client.files.uploads.get(original.clone()).await?.upload.unwrap();
        assert_eq!(progress.accepted_length, split as u64);
        assert_eq!(progress.durable_length, split as u64);
        process.close().await?;

        // Reopen the Backend, Resume the Upload and Fence Its Previous Writer
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = Client::from_channel(channel, super::support::client_settings())?;
        let uploads = client.files.uploads.clone();
        let file_client = client.files.clone();
        let retained = uploads.get(original.clone()).await?.upload.unwrap();
        assert_eq!(retained.durable_length, split as u64);
        let resumed = uploads
            .resume_with(
                original.clone(),
                FilesUploadsResumeOptions {
                    request_id: Some("resume-file".into()),
                    ..Default::default()
                },
            )
            .await?;
        let upload = resumed.upload.unwrap();
        assert_ne!(upload.writer_generation, original.writer_generation);
        assert!(uploads.append(original, split as u64, vec![1]).await.is_err());
        let mut offset = split;
        while offset < bytes.len() {
            let end = bytes.len().min(offset + resumed.chunk_bytes as usize);
            let options = FilesUploadsAppendOptions {
                request_id: Some(format!("remaining-{offset}")),
                ..Default::default()
            };
            uploads.append_with(upload.clone(), offset as u64, bytes[offset..end].to_vec(), options).await?;
            offset = end;
        }
        let options = FilesUploadsPublishOptions {
            request_id: Some("publish-file".into()),
            ..Default::default()
        };
        let digest = Sha256::digest(&bytes).to_vec();
        let published = uploads
            .publish_with(
                upload.clone(),
                bytes.len() as u64,
                digest.clone(),
                FilesUploadsPublishOptions {
                    request_id: options.request_id.clone(),
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(uploads.publish_with(upload, bytes.len() as u64, digest, options).await?, published);
        let selected = file_client.get(published.upload.unwrap()).await?;
        let reference = selected.file.clone().unwrap();
        let mut read = Vec::new();
        while read.len() < bytes.len() {
            let options = FilesReadOptions {
                at: selected.resolved_at.clone(),
                ..Default::default()
            };
            let page = file_client.read_with(reference.clone(), read.len() as u64, selected.chunk_bytes, options).await?;
            assert_eq!(page.resolved_at, selected.resolved_at);
            assert!(!page.data.is_empty());
            read.extend(page.data);
        }
        assert_eq!(read, bytes);

        // Watch Immutable Published Content in Order Through Its Final Completion
        let mut watch = file_client
            .watch_with(
                0,
                FilesWatchOptions {
                    file: Some(reference.clone()),
                    ..Default::default()
                },
            )
            .await?;
        let mut followed = Vec::new();
        let mut completed = false;
        while let Some(frame) = watch.next().await? {
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
        assert_eq!(followed, bytes);
        // The SDK Convenience Publishes an Unscoped Host File with the Same Resource.
        let upload_path = root.path().join("client-upload.bin");
        std::fs::write(&upload_path, &bytes)?;
        let sdk = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?;
        let uploaded = sdk.files.upload(&upload_path).await?;
        assert_eq!(uploaded.sha256, Sha256::digest(&bytes).to_vec());
        let deleted = FilesDeleteRequest {
            file: Some(uploaded),
            request_id: "delete-file".into(),
        };
        let receipt = file_client
            .delete_with(
                ((deleted.clone()).file).unwrap_or_default(),
                FilesDeleteOptions {
                    request_id: Some((deleted.clone()).request_id),
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(
            file_client
                .delete_with(
                    ((deleted.clone()).file).unwrap_or_default(),
                    FilesDeleteOptions {
                        request_id: Some((deleted.clone()).request_id),
                        ..Default::default()
                    }
                )
                .await?,
            receipt
        );
        store.close_streams("content-a")?;
        store.close("content-a", Termination::Completed, ExecutionOutcome::default())?;
        store.seal("content-a")?;
        let evidence =
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .evidence
                .clone();
        let archived = evidence
            .archive(EvidenceSubject {
                kind: Some(EvidenceSubjectKind::Execution("content-a".into())),
            })
            .await?;
        assert!(archived.file.unwrap().length > bytes.len() as u64);
        process.close().await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let artifacts =
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .content
                .artifacts
                .clone();
        let restored = artifacts.get(attached.artifact_id.clone()).await?;
        assert_eq!(restored.metadata, Some(attached));
        assert_eq!(restored.file.unwrap().sha256, reference.sha256);
        process.close().await?;
        Ok::<_, Box<dyn Error>>(())
    })
    .await?
}
