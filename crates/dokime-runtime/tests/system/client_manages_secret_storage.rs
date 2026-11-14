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
use dokime::Client;
use dokime::Metadata;
use dokime::SecretWritable;
use dokime::client::SecretImportAction;
use dokime::client::SecretImportChoice;
use dokime::client::SecretsDeleteOptions;
use dokime::client::SecretsImportOptions;
use dokime::client::SecretsSetOptions;
use std::error::Error;
use tonic::Code;
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
async fn client_manages_secret_storage() -> Result<(), Box<dyn Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Save a Secret by Identifier and Reject Stale Edits
        let root = super::support::tempdir()?;
        let storage = root.path().to_owned();
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(&storage, &credentials.environment).await?;
        let secrets =
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .secrets
                .clone();
        let initial = secrets.list().await?;
        let secret = uuid::Uuid::now_v7().to_string();
        let metadata = Metadata {
            identifier: "bench.access".into(),
            label: "Bench Access".into(),
            ..Default::default()
        };
        let saved = secrets
            .set_with(
                metadata.identifier.clone(),
                SecretWritable {
                    metadata: Some(metadata.clone()),
                    value: Some(secret.clone()),
                },
                initial.generation.to_be_bytes().to_vec(),
                SecretsSetOptions {
                    request_id: Some("save".into()),
                    ..Default::default()
                },
            )
            .await?;
        let actual = secrets.get(metadata.identifier.clone()).await?;
        assert_eq!(actual.value.unwrap().value.as_ref(), Some(&secret));
        assert_eq!(
            secrets
                .set_with(
                    metadata.identifier.clone(),
                    SecretWritable {
                        metadata: Some(metadata.clone()),
                        value: None
                    },
                    initial.generation.to_be_bytes().to_vec(),
                    SecretsSetOptions {
                        request_id: Some("stale".into()),
                        ..Default::default()
                    }
                )
                .await
                .unwrap_err()
                .code(),
            Code::Aborted
        );
        let generation = u64::from_be_bytes(saved.revision.as_slice().try_into()?);
        let archive = secrets.export(vec![metadata.identifier.clone()], generation).await?.archive.unwrap();
        let preview = secrets.import(archive.data.clone(), Default::default(), Default::default(), true).await?.preview.unwrap();
        assert!(preview.entries[0].exists);
        assert!(secrets.import(b"invalid".to_vec(), Default::default(), Default::default(), true).await.is_err());
        process.close().await?;
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(&storage, &credentials.environment).await?;
        let secrets =
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .secrets
                .clone();
        assert_eq!(secrets.get(metadata.identifier.clone()).await?.value.unwrap().value.as_ref(), Some(&secret));
        secrets
            .delete_with(
                vec![metadata.identifier.clone()],
                saved.revision,
                SecretsDeleteOptions {
                    request_id: Some("remove".into()),
                    ..Default::default()
                },
            )
            .await?;
        assert!(secrets.get(metadata.identifier.clone()).await.is_err());
        let empty = secrets.list().await?;
        let imported = secrets
            .import_with(
                archive.data,
                vec![SecretImportChoice {
                    identifier: metadata.identifier.clone(),
                    action: SecretImportAction::Replace as i32,
                }],
                empty.generation,
                Default::default(),
                SecretsImportOptions {
                    request_id: Some("import".into()),
                    ..Default::default()
                },
            )
            .await?
            .snapshot
            .unwrap();
        assert_eq!(secrets.get(metadata.identifier.clone()).await?.value.unwrap().value.as_ref(), Some(&secret));
        let cleaned = secrets
            .delete_with(
                vec![metadata.identifier],
                imported.generation.to_be_bytes().to_vec(),
                SecretsDeleteOptions {
                    request_id: Some("cleanup".into()),
                    ..Default::default()
                },
            )
            .await?;
        assert!(!cleaned.cleanup_pending);
        assert!(secrets.list().await?.entries.is_empty());
        process.close().await?;
        Ok::<(), Box<dyn Error>>(())
    })
    .await?
}
