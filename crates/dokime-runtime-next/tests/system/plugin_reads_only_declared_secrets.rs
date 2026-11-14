/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugin Reads Only Declared Secrets

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::HashMap, error::Error as StdError};

// Third Party Imports
use prost::Message;
use tokio::time;
use tonic::transport::Endpoint;

// Local Imports
use dokime::{client::HostClient, v1};
use dokime_protocol::client::{
    PluginState, SecretWritable, SecretsDeleteRequest, SecretsGetRequest, SecretsListRequest, SecretsSetRequest,
    secrets_client::SecretsClient,
};
use dokime_runtime_next::PluginProcess;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(PartialEq, Message)]
struct ReadResult {
    #[prost(string, tag = "1")]
    value: String,

    #[prost(string, tag = "2")]
    refusal: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

async fn read(client: &HostClient, plugin: &str, identifier: &str) -> Result<ReadResult, Box<dyn StdError>> {
    // Materialize the Requested Identifier and Execute the Plugin's Read
    let parameters = HashMap::from([(
        "identifier".into(),
        vec![v1::Value {
            kind: Some(v1::value::Kind::Text(identifier.into())),
        }],
    )]);
    let cases = client.templates.materialize(plugin, format!("{plugin}.read"), parameters).await?;
    let cycle = client.cycles.start(cases).await?;
    let mut progress = cycle.watch();
    while let Some(frame) = progress.next().await? {
        if frame.finished {
            break;
        }
    }

    // Read the Result Returned Through the Real Invocation Carrier
    let records = client.records.list(cycle.executions[0].clone()).await?;
    assert_eq!(records.records.len(), 1);
    Ok(ReadResult::decode(records.records[0].payload.as_slice())?)
}

/* === Tests === */

#[tokio::test]
async fn plugin_reads_only_declared_secrets() -> Result<(), Box<dyn StdError>> {
    time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Connect Two Plugins with Different Secret Declarations
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let settings = super::support::settings();
        let executable = super::support::secrets_fixture()?;
        let mut reader = PluginProcess::start(&executable, true, &settings.placement).await?;
        let mut owner = PluginProcess::start_with_environment(
            &executable,
            true,
            &settings.placement,
            &[("DOKIME_SECRET_FIXTURE_OWNER".into(), "1".into())],
        )
        .await?;
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = HostClient::new(channel.clone(), super::support::client_settings())?;
        let reader_entry = client.plugins.add(reader.endpoint()).await?;
        assert_eq!(reader_entry.state, PluginState::Ready as i32, "{reader_entry:?}");
        let owner_entry = client.plugins.add(owner.endpoint()).await?;
        assert_eq!(owner_entry.state, PluginState::Ready as i32, "{owner_entry:?}");
        let mut secrets = SecretsClient::new(channel);

        // Administrative Writes and Reads Need No Plugin Declaration
        for identifier in ["secrets.owned", "secrets.other", "secrets.text"] {
            let initial = secrets.list(SecretsListRequest::default()).await?.into_inner();
            secrets
                .set(SecretsSetRequest {
                    name: identifier.into(),
                    value: Some(SecretWritable {
                        metadata: Some(v1::Metadata {
                            identifier: identifier.into(),
                            label: "Fixture Secret".into(),
                            ..Default::default()
                        }),
                        value: Some("fixture-secret-value".into()),
                    }),
                    expected_revision: initial.generation.to_be_bytes().to_vec(),
                    request_id: format!("save-{identifier}"),
                })
                .await?;
            let actual = secrets
                .get(SecretsGetRequest {
                    name: identifier.into(),
                    ..Default::default()
                })
                .await?
                .into_inner();
            assert_eq!(actual.value.unwrap().value.as_deref(), Some("fixture-secret-value"));
        }

        // Each Plugin Can Read Its Own Declared Secret
        let own = read(&client, "secret_reader", "secrets.owned").await?;
        assert_eq!(own.value, "fixture-secret-value");
        assert!(own.refusal.is_empty());
        let other = read(&client, "secret_owner", "secrets.other").await?;
        assert_eq!(other, own);

        // Another Plugin's Declaration Does Not Grant Access
        let existing = read(&client, "secret_reader", "secrets.other").await?;
        assert!(existing.value.is_empty());
        assert!(existing.refusal.contains("Read Secret: Declare Secret \"secrets.other\" in the Calling Plugin"));

        // The Same Identifier Has the Same Refusal After Administrative Deletion
        let initial = secrets.list(SecretsListRequest::default()).await?.into_inner();
        let deleted = secrets
            .delete(SecretsDeleteRequest {
                names: vec!["secrets.other".into()],
                expected_revision: initial.generation.to_be_bytes().to_vec(),
                request_id: "remove-other".into(),
            })
            .await?
            .into_inner();
        assert!(!deleted.cleanup_pending);
        let absent = read(&client, "secret_reader", "secrets.other").await?;
        assert_eq!(absent, existing);

        // A Declared but Missing Secret Retains Its Store Failure
        let missing = read(&client, "secret_owner", "secrets.other").await?;
        assert!(missing.value.is_empty());
        assert!(missing.refusal.contains("Set the Named Secret Before Reading It"));

        // A Text Setting Is Not a Secret Declaration
        let text = read(&client, "secret_reader", "secrets.text").await?;
        assert!(text.value.is_empty());
        assert!(text.refusal.contains("Read Secret: Declare Secret \"secrets.text\" in the Calling Plugin"));

        // Remove Every Credential Created by This Story
        let initial = secrets.list(SecretsListRequest::default()).await?.into_inner();
        let deleted = secrets
            .delete(SecretsDeleteRequest {
                names: vec!["secrets.owned".into(), "secrets.text".into()],
                expected_revision: initial.generation.to_be_bytes().to_vec(),
                request_id: "cleanup".into(),
            })
            .await?
            .into_inner();
        assert!(!deleted.cleanup_pending);
        assert!(secrets.list(SecretsListRequest::default()).await?.into_inner().entries.is_empty());
        client.close();
        process.close().await?;
        reader.close().await?;
        owner.close().await?;
        Ok::<(), Box<dyn StdError>>(())
    })
    .await?
}
