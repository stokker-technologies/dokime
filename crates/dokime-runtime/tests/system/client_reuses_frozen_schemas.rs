/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Reuses Frozen Schemas

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::error::Error;
use std::sync::Arc;

// Third Party Imports
use dokime_protocol::host::plugin_placement;
use harmos_protocol::model::PageRequest;
use prost::Message;
use prost_reflect::DescriptorPool;
use prost_reflect::DynamicMessage;
use prost_reflect::Value;
use tonic::Code;
use tonic::transport::Endpoint;

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::MapValue;
use dokime::RecordScope;
use dokime::record_scope;
use dokime_client_internal::Client;
use dokime_client_internal::EvidenceRecordsListOptions;
use dokime_client_internal::ExecutionsSchemasGetOptions;
use dokime_client_internal::FrozenSchemaCache;
use dokime_client_internal::host::PluginPlacement;
use dokime_client_internal::host::PluginRemote;
use dokime_client_internal::host::PluginValue;
use dokime_protocol::host::executions::SchemasGetRequest;
use dokime_runtime::PluginProcess;

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
fn client_reuses_frozen_schemas() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Start the Host with a Controlled Clock and the Thermal Plugin.
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
            let settings = super::support::settings();
            let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &settings.placement).await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel.clone(), super::support::client_settings())?;
            let entry = client
                .plugins
                .create(
                    "thermal",
                    PluginPlacement {
                        kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: plugin.endpoint().into() })),
                    },
                    PluginValue { enabled: true },
                )
                .await?
                .entry
                .unwrap();
            let entry = super::support::plugin_state(&client.plugins, entry, dokime_protocol::host::PluginState::Ready).await?;

            // Run Two Executions with the Same Frozen Descriptors.
            let cases = super::support::materialize(&client, "thermal", "thermal.cycle", Default::default()).await?;
            let mut executions = Vec::new();
            for case in [cases[0].clone(), cases[0].clone()] {
                let cycle = super::support::admit(&client, vec![case]).await?;
                let execution = cycle[0].clone();
                let mut progress = client.executions.progress.watch(cycle[0].clone()).await?;
                while let Some(frame) = progress.next().await? {
                    if frame.phase != "prompt" {
                        continue;
                    }
                    let pending = client.prompts.list(execution.clone(), true).await?;
                    for prompt in pending.prompts {
                        client.prompts.answers.create(prompt.prompt.unwrap().prompt_id, "0", MapValue::default(), true).await?;
                    }
                }
                executions.push(execution);
            }
            let first_reply = client.executions.get(executions[0].clone()).await?;
            eprintln!("Execution Get Reply: {} Bytes", first_reply.encoded_len());
            let first = first_reply.execution.unwrap();
            let second = client.executions.get(executions[1].clone()).await?.execution.unwrap();
            assert_eq!(first.schemas.as_ref().unwrap().digest, second.schemas.as_ref().unwrap().digest);
            let records = client
                .evidence
                .records
                .list_with(
                    String::new(),
                    EvidenceRecordsListOptions {
                        scope: Some(RecordScope {
                            kind: Some(record_scope::Kind::Execution(executions[0].clone())),
                        }),
                        versions: Some(Vec::new()),
                        between: Some(Default::default()),
                        step: Some(String::new()),
                        selection: Some(Default::default()),
                        ..Default::default()
                    },
                )
                .await?
                .records;
            let second_records = client
                .evidence
                .records
                .list_with(
                    String::new(),
                    EvidenceRecordsListOptions {
                        scope: Some(RecordScope {
                            kind: Some(record_scope::Kind::Execution(executions[1].clone())),
                        }),
                        versions: Some(Vec::new()),
                        between: Some(Default::default()),
                        step: Some(String::new()),
                        selection: Some(Default::default()),
                        ..Default::default()
                    },
                )
                .await?
                .records;

            // Fetch Complete Descriptors Once After the Plugin Stops and Decode a Payload.
            Client::from_channel(channel.clone(), super::support::client_settings())?
                .plugins
                .clone()
                .delete(entry.name.clone(), entry.revision.clone())
                .await?;
            plugin.close().await?;
            let mut cache = FrozenSchemaCache::default();
            let types = cache.get(&client, &first).await?;
            let pool = DescriptorPool::from_file_descriptor_set((*types).clone())?;
            let message = pool.get_message_by_name("thermal.Door").unwrap();
            let door = cache.decode(&client, &first, &records[0].record_type, records[0].payload.as_slice()).await?;
            assert_eq!(door.get_field_by_name("closed").unwrap().as_ref(), &Value::Bool(true));

            // Refuse Invalid Continuations and a Page Too Small for One Descriptor.
            let schemas = Client::from_channel(channel, super::support::client_settings())?.executions.schemas.clone();
            let mut request = SchemasGetRequest {
                name: executions[0].clone(),
                at: first.resolved_at.clone(),
                page: Some(PageRequest {
                    page_size: 1,
                    page_token: "invalid:1".into(),
                    ..Default::default()
                }),
            };
            assert!(
                schemas
                    .get_with(
                        (request.clone()).name,
                        ExecutionsSchemasGetOptions {
                            at: (request.clone()).at,
                            page: (request.clone()).page,
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );
            request.at = None;
            assert!(
                schemas
                    .get_with(
                        (request.clone()).name,
                        ExecutionsSchemasGetOptions {
                            at: (request.clone()).at,
                            page: (request.clone()).page,
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );
            request.name = executions[1].clone();
            request.at = first.resolved_at.clone();
            request.page.as_mut().unwrap().page_token.clear();
            assert!(
                schemas
                    .get_with(
                        (request.clone()).name,
                        ExecutionsSchemasGetOptions {
                            at: (request.clone()).at,
                            page: (request.clone()).page,
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );
            request.name = executions[0].clone();
            request.page.as_mut().unwrap().max_bytes = 1;
            assert_eq!(
                schemas
                    .get_with(
                        (request.clone()).name,
                        ExecutionsSchemasGetOptions {
                            at: (request.clone()).at,
                            page: (request.clone()).page,
                            ..Default::default()
                        }
                    )
                    .await
                    .unwrap_err()
                    .code(),
                Code::ResourceExhausted
            );

            // Close the Host: A Second Execution Must Decode Without Another Fetch.
            process.close().await?;
            let reused = cache.get(&client, &second).await?;
            assert!(Arc::ptr_eq(&types, &reused));
            let door = DynamicMessage::decode(message, second_records[0].payload.as_slice())?;
            assert_eq!(door.get_field_by_name("closed").unwrap().as_ref(), &Value::Bool(true));

            Ok(())
        })
        .await?
    }))
}
