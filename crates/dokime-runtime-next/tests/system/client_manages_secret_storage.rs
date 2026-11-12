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
        let root = tempfile::tempdir()?;
        let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start(root.path()).await?;
        let mut secrets = v1::secrets_client::SecretsClient::connect(process.url.clone()).await?;
        let initial = secrets.list(v1::SecretsListRequest {}).await?.into_inner();
        let saved = secrets
            .put(v1::SecretsPutRequest {
                metadata: Some(v1::Metadata {
                    identifier: "bench.access".into(),
                    label: "Bench Access".into(),
                    ..Default::default()
                }),
                value: Some("story-secret-value".into()),
                generation: initial.generation,
            })
            .await?
            .into_inner();
        assert_eq!(
            secrets
                .lookup(v1::SecretsLookupRequest { identifier: "bench.access".into() })
                .await
                .map_err(|error| format!("Read the Newly Saved Slot: {error}"))?
                .into_inner()
                .value
                .as_deref(),
            Some("story-secret-value")
        );
        assert!(
            secrets
                .put(v1::SecretsPutRequest {
                    metadata: saved.entries[0].metadata.clone(),
                    value: None,
                    generation: initial.generation
                })
                .await
                .is_err()
        );
        let archive = secrets
            .export(v1::SecretsExportRequest {
                identifiers: vec!["bench.access".into()],
                generation: saved.generation,
            })
            .await?
            .into_inner();
        let preview = secrets.preview(v1::SecretsPreviewRequest { data: archive.data.clone() }).await?.into_inner();
        assert!(preview.entries[0].exists);
        assert!(secrets.preview(v1::SecretsPreviewRequest { data: b"invalid file".to_vec() }).await.is_err());
        process.close().await?;

        // Reopen Protected Storage and Round-Trip an Explicit Import
        let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start(root.path()).await?;
        let mut secrets = v1::secrets_client::SecretsClient::connect(process.url.clone()).await?;
        assert_eq!(
            secrets
                .lookup(v1::SecretsLookupRequest { identifier: "bench.access".into() })
                .await
                .map_err(|error| format!("Read the Slot After Restart: {error}"))?
                .into_inner()
                .value
                .as_deref(),
            Some("story-secret-value")
        );
        let removed = secrets
            .remove(v1::SecretsRemoveRequest {
                identifiers: vec!["bench.access".into()],
                generation: saved.generation,
            })
            .await?
            .into_inner();
        assert!(secrets.lookup(v1::SecretsLookupRequest { identifier: "bench.access".into() }).await.is_err());
        let imported = secrets
            .import(v1::SecretsImportRequest {
                data: archive.data,
                choices: vec![v1::SecretImportChoice {
                    identifier: "bench.access".into(),
                    action: v1::SecretImportAction::Replace as i32,
                }],
                generation: removed.generation,
            })
            .await?
            .into_inner();
        assert_eq!(
            secrets
                .lookup(v1::SecretsLookupRequest { identifier: "bench.access".into() })
                .await
                .map_err(|error| format!("Read the Imported Slot: {error}"))?
                .into_inner()
                .value
                .as_deref(),
            Some("story-secret-value")
        );
        let cleaned = secrets
            .remove(v1::SecretsRemoveRequest {
                identifiers: vec!["bench.access".into()],
                generation: imported.generation,
            })
            .await?
            .into_inner();
        assert!(cleaned.entries.is_empty());
        assert!(!cleaned.cleanup_pending);
        process.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
