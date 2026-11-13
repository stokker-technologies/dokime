/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Manages Secret Storage

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

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
async fn client_manages_secret_storage() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        use dokime::v1;

        // Save a Secret by Identifier and Reject Stale Edits
        let root = super::support::tempdir()?;
        let storage = root.path().to_owned();
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(&storage, &credentials.environment).await?;
        let mut secrets = dokime_protocol::client::secrets_client::SecretsClient::connect(process.url.clone()).await?;
        let initial = secrets.list(dokime_protocol::client::SecretsListRequest::default()).await?.into_inner();
        let secret = uuid::Uuid::now_v7().to_string();
        let metadata = v1::Metadata {
            identifier: "bench.access".into(),
            label: "Bench Access".into(),
            ..Default::default()
        };
        let saved = secrets
            .set(dokime_protocol::client::SecretsSetRequest {
                name: metadata.identifier.clone(),
                value: Some(dokime_protocol::client::SecretWritable {
                    metadata: Some(metadata.clone()),
                    value: Some(secret.clone()),
                }),
                expected_revision: initial.generation.to_be_bytes().to_vec(),
                request_id: "save".into(),
            })
            .await?
            .into_inner();
        let actual = secrets
            .get(dokime_protocol::client::SecretsGetRequest {
                name: metadata.identifier.clone(),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(actual.value.unwrap().value.as_ref(), Some(&secret));
        assert_eq!(
            secrets
                .set(dokime_protocol::client::SecretsSetRequest {
                    name: metadata.identifier.clone(),
                    value: Some(dokime_protocol::client::SecretWritable {
                        metadata: Some(metadata.clone()),
                        value: None
                    }),
                    expected_revision: initial.generation.to_be_bytes().to_vec(),
                    request_id: "stale".into(),
                })
                .await
                .unwrap_err()
                .code(),
            tonic::Code::Aborted
        );
        let generation = u64::from_be_bytes(saved.revision.as_slice().try_into()?);
        let archive = secrets
            .export(dokime_protocol::client::SecretsExportRequest {
                names: vec![metadata.identifier.clone()],
                generation,
                ..Default::default()
            })
            .await?
            .into_inner()
            .archive
            .unwrap();
        let preview = secrets
            .import(dokime_protocol::client::SecretsImportRequest {
                data: archive.data.clone(),
                validate_only: true,
                ..Default::default()
            })
            .await?
            .into_inner()
            .preview
            .unwrap();
        assert!(preview.entries[0].exists);
        assert!(
            secrets
                .import(dokime_protocol::client::SecretsImportRequest {
                    data: b"invalid".to_vec(),
                    validate_only: true,
                    ..Default::default()
                })
                .await
                .is_err()
        );
        process.close().await?;
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(&storage, &credentials.environment).await?;
        let mut secrets = dokime_protocol::client::secrets_client::SecretsClient::connect(process.url.clone()).await?;
        assert_eq!(
            secrets
                .get(dokime_protocol::client::SecretsGetRequest {
                    name: metadata.identifier.clone(),
                    ..Default::default()
                })
                .await?
                .into_inner()
                .value
                .unwrap()
                .value
                .as_ref(),
            Some(&secret)
        );
        secrets
            .delete(dokime_protocol::client::SecretsDeleteRequest {
                names: vec![metadata.identifier.clone()],
                expected_revision: saved.revision,
                request_id: "remove".into(),
            })
            .await?;
        assert!(
            secrets
                .get(dokime_protocol::client::SecretsGetRequest {
                    name: metadata.identifier.clone(),
                    ..Default::default()
                })
                .await
                .is_err()
        );
        let empty = secrets.list(dokime_protocol::client::SecretsListRequest::default()).await?.into_inner();
        let imported = secrets
            .import(dokime_protocol::client::SecretsImportRequest {
                data: archive.data,
                choices: vec![dokime_protocol::client::SecretImportChoice {
                    identifier: metadata.identifier.clone(),
                    action: dokime_protocol::client::SecretImportAction::Replace as i32,
                }],
                generation: empty.generation,
                request_id: "import".into(),
                ..Default::default()
            })
            .await?
            .into_inner()
            .snapshot
            .unwrap();
        assert_eq!(
            secrets
                .get(dokime_protocol::client::SecretsGetRequest {
                    name: metadata.identifier.clone(),
                    ..Default::default()
                })
                .await?
                .into_inner()
                .value
                .unwrap()
                .value
                .as_ref(),
            Some(&secret)
        );
        let cleaned = secrets
            .delete(dokime_protocol::client::SecretsDeleteRequest {
                names: vec![metadata.identifier],
                expected_revision: imported.generation.to_be_bytes().to_vec(),
                request_id: "cleanup".into(),
            })
            .await?
            .into_inner();
        assert!(!cleaned.cleanup_pending);
        assert!(secrets.list(dokime_protocol::client::SecretsListRequest::default()).await?.into_inner().entries.is_empty());
        process.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
