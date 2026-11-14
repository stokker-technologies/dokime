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
use std::{error::Error, sync::Arc};

// Third Party Imports
use harmos_protocol::v1::PageRequest;
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, Value};
use tonic::transport::Endpoint;

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::client::{FrozenSchemaCache, HostClient};
use dokime_protocol::client::{
    RemovePlugin,
    executions::{SchemasGetRequest, schemas_client::SchemasClient},
    plugins_client::PluginsClient,
};
use dokime_runtime_next::PluginProcess;
use tonic::Code;

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
async fn client_reuses_frozen_schemas() -> Result<(), Box<dyn Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Start the Host with a Controlled Clock and the Thermal Plugin.
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let settings = super::support::settings();
        let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &settings.placement).await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = HostClient::new(channel.clone(), super::support::client_settings())?;
        let entry = client.plugins.add(plugin.endpoint()).await?;

        // Run Two Executions with the Same Frozen Descriptors.
        let cases = client.templates.materialize("thermal", "thermal.cycle", Default::default()).await?;
        let mut executions = Vec::new();
        for case in [cases[0].clone(), cases[0].clone()] {
            let cycle = client.cycles.start(vec![case]).await?;
            let execution = cycle.executions[0].clone();
            let mut progress = cycle.watch();
            while let Some(frame) = progress.next().await? {
                if frame.phase != "prompt" {
                    continue;
                }
                let pending = client.prompts.pending(execution.clone()).await?;
                for prompt in pending.prompts {
                    client.prompts.respond(prompt.prompt.unwrap().prompt_id, "yes", true).await?;
                }
            }
            executions.push(execution);
        }
        let first_reply = client.executions.get(executions[0].clone()).await?;
        eprintln!("Execution Get Reply: {} Bytes", first_reply.encoded_len());
        let first = first_reply.execution.unwrap();
        let second = client.executions.get(executions[1].clone()).await?.execution.unwrap();
        assert_eq!(first.schemas.as_ref().unwrap().digest, second.schemas.as_ref().unwrap().digest);
        let records = client.records.list(executions[0].clone()).await?.records;
        let second_records = client.records.list(executions[1].clone()).await?.records;

        // Fetch Complete Descriptors Once After the Plugin Stops and Decode a Payload.
        PluginsClient::new(channel.clone()).remove(RemovePlugin { entry: entry.entry }).await?;
        plugin.close().await?;
        let mut cache = FrozenSchemaCache::default();
        let types = cache.get(&client, &first).await?;
        let pool = DescriptorPool::from_file_descriptor_set((*types).clone())?;
        let message = pool.get_message_by_name("thermal.v1.Door").unwrap();
        let door = cache.decode(&client, &first, &records[0].record_type, records[0].payload.as_slice()).await?;
        assert_eq!(door.get_field_by_name("closed").unwrap().as_ref(), &Value::Bool(true));

        // Refuse Invalid Continuations and a Page Too Small for One Descriptor.
        let mut schemas = SchemasClient::new(channel);
        let mut request = SchemasGetRequest {
            name: executions[0].clone(),
            at: first.resolved_at.clone(),
            page: Some(PageRequest {
                page_size: 1,
                page_token: "invalid:1".into(),
                ..Default::default()
            }),
        };
        assert!(schemas.get(request.clone()).await.is_err());
        request.at = None;
        assert!(schemas.get(request.clone()).await.is_err());
        request.name = executions[1].clone();
        request.at = first.resolved_at.clone();
        request.page.as_mut().unwrap().page_token.clear();
        assert!(schemas.get(request.clone()).await.is_err());
        request.name = executions[0].clone();
        request.page.as_mut().unwrap().max_bytes = 1;
        assert_eq!(schemas.get(request).await.unwrap_err().code(), Code::ResourceExhausted);

        // Close the Host: A Second Execution Must Decode Without Another Fetch.
        process.close().await?;
        let reused = cache.get(&client, &second).await?;
        assert!(Arc::ptr_eq(&types, &reused));
        let door = DynamicMessage::decode(message, second_records[0].payload.as_slice())?;
        assert_eq!(door.get_field_by_name("closed").unwrap().as_ref(), &Value::Bool(true));
        client.close();
        Ok(())
    })
    .await?
}
