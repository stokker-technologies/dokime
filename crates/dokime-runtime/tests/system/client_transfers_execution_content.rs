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
use dokime_protocol::host::evidence_subject;
use dokime_protocol::model::origin;
use dokime_protocol::model::{Metadata, Value, value};
use dokime_storage::CONTENT_CHUNK_BYTES;
use dokime_storage::Execution;
use dokime_storage::Store;
use harmos_protocol::model::FileMetadata;
use harmos_protocol::model::PageRequest;
use sha2::Digest;
use sha2::Sha256;
use std::collections::HashMap;
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

#[test]
fn native_archive_preserves_file_bytes_and_history() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            use prost::Message;
            use std::io::Read;

            // Create Product Evidence and Publish Its File Through the Registered Backend
            let root = super::support::tempdir()?;
            let store = Store::open(root.path())?;
            store.create(&Execution {
                identifier: "native-archive".into(),
                cycle: "native-archive".into(),
                origin: Some(Origin {
                    kind: Some(origin::Kind::Import(Import::default())),
                }),
                ..Default::default()
            })?;
            let process = HostProcess::start_with_environment(root.path(), &[]).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, super::support::client_settings())?;
            let bytes = vec![37; 140003];
            let created = client
                .files
                .uploads
                .create(FileMetadata {
                    id: "capture".into(),
                    scope: Some("native-archive".into()),
                    media_type: "application/octet-stream".into(),
                    expected_length: Some(bytes.len() as u64),
                    expected_sha256: Some(Sha256::digest(&bytes).to_vec()),
                })
                .await?;
            let upload = created.upload.unwrap();
            for (index, data) in bytes.chunks(created.chunk_bytes as usize).enumerate() {
                client.files.uploads.append(upload.clone(), (index * created.chunk_bytes as usize) as u64, data.to_vec()).await?;
            }
            let reference =
                client.files.uploads.publish(upload, bytes.len() as u64, Sha256::digest(&bytes).to_vec()).await?.upload.unwrap();
            let artifact = client
                .content
                .artifacts
                .create(
                    reference.clone(),
                    ArtifactScope {
                        execution_id: Some("native-archive".into()),
                        ..Default::default()
                    },
                    "capture.bin",
                    "application/octet-stream",
                )
                .await?
                .artifact
                .unwrap();
            store.close_streams("native-archive")?;
            store.close("native-archive", Termination::Completed, ExecutionOutcome::default())?;
            store.seal("native-archive")?;

            // Read the Archive Through the Same Native File Resource
            let archived = client
                .evidence
                .archive(EvidenceSubject {
                    kind: Some(evidence_subject::Kind::Execution("native-archive".into())),
                })
                .await?
                .file
                .unwrap();
            let mut bytes_out = Vec::new();
            while bytes_out.len() < archived.length as usize {
                let page = client.files.read(archived.clone(), bytes_out.len() as u64, 1024).await?;
                assert!(!page.data.is_empty());
                bytes_out.extend(page.data);
            }
            assert_eq!(Sha256::digest(&bytes_out).as_slice(), archived.sha256);
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes_out))?;
            let manifest: serde_json::Value = serde_json::from_reader(archive.by_name("manifest.json")?)?;
            assert_eq!(manifest["version"], 1);
            assert_eq!(manifest["files"].as_array().unwrap().len(), 1);
            let member = &manifest["files"][0];
            let mut original = Vec::new();
            archive.by_name(member["path"].as_str().unwrap())?.read_to_end(&mut original)?;
            assert_eq!(original, bytes);
            let mut retained = Vec::new();
            archive.by_name(member["reference"].as_str().unwrap())?.read_to_end(&mut retained)?;
            assert_eq!(harmos_protocol::model::FileReference::decode(retained.as_slice())?, reference);
            assert_eq!(member["size"], original.len());
            assert_eq!(member["sha256"], Sha256::digest(&original).iter().map(|byte| format!("{byte:02x}")).collect::<String>());
            let mut evidence = Vec::new();
            archive.by_name("evidence.pb")?.read_to_end(&mut evidence)?;
            let evidence = dokime_storage::ExecutionArchive::decode(evidence.as_slice())?;
            assert_eq!(evidence.artifacts.len(), 1);
            assert_eq!(evidence.artifacts[0].artifact.as_ref(), Some(&artifact));
            assert_eq!(evidence.artifacts[0].request.as_ref().unwrap().file.as_ref(), Some(&reference));
            process.close().await?;
            Ok::<_, Box<dyn Error>>(())
        })
        .await?
    }))
}

#[test]
fn client_transfers_execution_content() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
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
                        kind: Some(origin::Kind::Import(Import::default())),
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
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel.clone(), super::support::client_settings())?;
            let original = super::support::native_file(&client, "file-original", "content-a", b"original\n").await?;
            let replacement = super::support::native_file(&client, "file-replacement", "content-a", b"replacement\n").await?;
            process.close().await?;
            super::support::register_execution_files(
                root.path(),
                &credentials.environment,
                vec![
                    dokime_storage::StoredExecutionFile {
                        metadata: Some(file.clone()),
                        file: Some(original.clone()),
                    },
                    dokime_storage::StoredExecutionFile {
                        metadata: Some(file),
                        file: Some(replacement),
                    },
                ],
            )
            .await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel.clone(), super::support::client_settings())?;
            let bytes = vec![42; 140003];
            let reference = super::support::native_file(&client, "uploaded", "content-a", &bytes).await?;
            let uploaded = client
                .content
                .artifacts
                .create(
                    reference,
                    ArtifactScope {
                        execution_id: Some("content-a".into()),
                        cycle_id: Some("content-cycle".into()),
                        ..Default::default()
                    },
                    "capture.bin",
                    "application/octet-stream",
                )
                .await?
                .artifact
                .unwrap();
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
            let uploaded = artifacts.get(uploaded.artifact_id).await?;
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
                metadata: Some(Metadata {
                    attributes: HashMap::from([
                        (
                            "sensor".into(),
                            Value {
                                kind: Some(value::Kind::Text("temperature".into())),
                            },
                        ),
                        ("run".into(), Value { kind: Some(value::Kind::Integer(42)) }),
                    ]),
                    ..Default::default()
                }),
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
            let mut changed = request.clone();
            changed.filename = "changed.bin".into();
            assert!(
                artifacts
                    .create_with(
                        changed.file.unwrap(),
                        changed.scope.unwrap(),
                        changed.filename,
                        changed.content_type,
                        ContentArtifactsCreateOptions {
                            request_id: Some(changed.request_id),
                            metadata: changed.metadata,
                            ..Default::default()
                        },
                    )
                    .await
                    .is_err()
            );
            let mut foreign = request.clone();
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
            assert_eq!(client.files.read(file.clone(), 0, 64).await?.data, b"replacement\n");
            assert_eq!(client.files.read(original.clone(), 0, 64).await?.data, b"original\n");
            assert!(
                client
                    .files
                    .delete_with(
                        original,
                        FilesDeleteOptions {
                            request_id: Some("retain-replaced-file".into()),
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );
            assert!(files.get("content-b", "logs/output.txt").await.is_err());
            assert!(files.get("content-a", "../logs/output.txt").await.is_err());
            let listed = files.list("content-a").await?;
            assert_eq!(listed.files.len(), 1);
            let committed = attached.clone();
            let attached = attached.artifact.unwrap();
            let stored = artifacts.get(attached.artifact_id.clone()).await?.file.unwrap();
            assert_eq!(stored.length, bytes.len() as u64);
            let mut actual = Vec::new();
            while actual.len() < bytes.len() {
                actual.extend(client.files.read(stored.clone(), actual.len() as u64, CONTENT_CHUNK_BYTES as u64).await?.data);
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
            assert!(first.receipt.as_ref().unwrap().durable_at.is_none());
            // Await Graceful Settlement Before Verifying the Durable Prefix After Restart
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
            let sdk =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?;
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
                    kind: Some(evidence_subject::Kind::Execution("content-a".into())),
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
            let replay = artifacts
                .create_with(
                    request.file.clone().unwrap(),
                    request.scope.clone().unwrap(),
                    request.filename.clone(),
                    request.content_type.clone(),
                    ContentArtifactsCreateOptions {
                        request_id: Some(request.request_id.clone()),
                        metadata: request.metadata.clone(),
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(replay, committed);
            let restored = artifacts.get(attached.artifact_id.clone()).await?;
            assert_eq!(restored.metadata, Some(attached));
            assert_eq!(restored.file.unwrap().sha256, reference.sha256);
            process.close().await?;
            Ok::<_, Box<dyn Error>>(())
        })
        .await?
    }))
}
