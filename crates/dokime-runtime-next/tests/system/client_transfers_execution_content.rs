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
        let file = v1::ExecutionFileMetadata {
            relative_path: "logs/output.txt".into(),
            content_type: Some("text/plain".into()),
            kind: v1::ExecutionFileKind::Text as i32,
            state: v1::ExecutionFileState::Streaming as i32,
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
        for (index, chunk) in bytes.chunks(dokime_storage::CONTENT_CHUNK_BYTES).enumerate() {
            store.append_content("uploaded", (index * dokime_storage::CONTENT_CHUNK_BYTES) as u64, chunk)?;
        }
        store.finish_artifact("uploaded", bytes.len() as u64, &digest)?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let mut artifacts = v1::content::artifacts_client::ArtifactsClient::new(channel.clone());
        let mut files = v1::content::execution_files_client::ExecutionFilesClient::new(channel);
        let first = artifacts
            .list(v1::content::ArtifactsListRequest {
                page: Some(harmos_protocol::v1::PageRequest { page_size: 1, ..Default::default() }),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(first.artifacts.len(), 1);
        let uploaded = artifacts
            .get(v1::content::ArtifactsGetRequest {
                name: "uploaded".into(),
                ..Default::default()
            })
            .await?
            .into_inner();
        let reference = uploaded.file.unwrap();
        assert_eq!(reference.length, bytes.len() as u64);
        assert_eq!(reference.sha256, sha2::Sha256::digest(&bytes).as_slice());
        let request = v1::content::ArtifactsCreateRequest {
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
            .get(v1::content::ExecutionFilesGetRequest {
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
                .get(v1::content::ExecutionFilesGetRequest {
                    parent: "content-b".into(),
                    name: "logs/output.txt".into(),
                    ..Default::default()
                })
                .await
                .is_err()
        );
        assert!(
            files
                .get(v1::content::ExecutionFilesGetRequest {
                    parent: "content-a".into(),
                    name: "../logs/output.txt".into(),
                    ..Default::default()
                })
                .await
                .is_err()
        );
        let listed = files
            .list(v1::content::ExecutionFilesListRequest {
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
            actual.extend(store.content_window(&stored.identifier, actual.len() as u64, dokime_storage::CONTENT_CHUNK_BYTES)?);
        }
        assert_eq!(actual, bytes);
        store.close_streams("content-a")?;
        store.close("content-a", v1::Termination::Completed, v1::ExecutionOutcome::default())?;
        store.seal("content-a")?;
        let mut evidence = v1::evidence_client::EvidenceClient::connect(process.url.clone()).await?;
        let archived = evidence
            .archive(v1::EvidenceArchiveRequest {
                subject: Some(v1::EvidenceSubject {
                    kind: Some(v1::evidence_subject::Kind::Execution("content-a".into())),
                }),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert!(archived.file.unwrap().length > bytes.len() as u64);
        process.close().await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let mut artifacts = v1::content::artifacts_client::ArtifactsClient::connect(process.url.clone()).await?;
        let restored = artifacts
            .get(v1::content::ArtifactsGetRequest {
                name: attached.artifact_id.clone(),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(restored.metadata, Some(attached));
        assert_eq!(restored.file.unwrap().sha256, reference.sha256);
        process.close().await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    })
    .await?
}
