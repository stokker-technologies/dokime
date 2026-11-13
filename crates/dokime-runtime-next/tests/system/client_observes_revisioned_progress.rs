/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugin Progress Has Revisioned Values and Retained Client Reads

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{client_runs_thermal_suite_over_grpc::HostProcess, support};
use dokime::v1::executions::{ProgressGetRequest, ProgressWatchRequest};

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
async fn client_observes_revisioned_progress() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
        // Start the Product and Admit the Plugin Through Its Client API
        let root = support::tempdir()?;
        let credential_root = support::tempdir()?;
        let credentials = support::CredentialStore::start(credential_root.path()).await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let settings = support::settings();
        let mut plugin = dokime_runtime_next::PluginProcess::start(&support::content_fixture()?, true, &settings.placement).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let host = dokime::client::HostClient::new(channel.clone(), support::client_settings())?;
        let client = dokime::Client::from_channel(channel, support::client_settings())?;
        host.plugins.add(plugin.endpoint()).await?;
        let cases = host.templates.materialize("content", "content.progress", std::collections::HashMap::new()).await?;
        let cycle = host.cycles.start(cases).await?;
        let name = cycle.executions[0].clone();

        // Open the Descriptor-Derived Hierarchy and Retain a Durable Read
        let mut watch =
            client.executions.progress.watch(tonic::Request::new(ProgressWatchRequest { name: name.clone(), at: None })).await?;
        loop {
            let response = watch.next().await?.ok_or("Watch Progress: Missing Prompt Event")?;
            if response.phase == "prompt" {
                break;
            }
        }
        let first =
            client.executions.progress.get(tonic::Request::new(ProgressGetRequest { name: name.clone(), at: None })).await?.into_inner();
        assert_eq!(first.value.as_ref().unwrap().completed, 1);
        assert_eq!(first.value.as_ref().unwrap().label, "First Value");
        let prompt = host.prompts.pending(name.clone()).await?.prompts[0].prompt_id.clone();
        host.prompts.respond(prompt, "continue", true).await?;
        let mut finished = false;
        while let Some(response) = watch.next().await? {
            finished |= response.finished;
        }
        assert!(finished);

        // Whole-Value Replacement Clears Fields and Execution Detail Agrees
        let current =
            client.executions.progress.get(tonic::Request::new(ProgressGetRequest { name: name.clone(), at: None })).await?.into_inner();
        assert_eq!(current.value.as_ref().unwrap().completed, 2);
        assert!(current.value.as_ref().unwrap().label.is_empty());
        assert_ne!(current.revision, first.revision);
        let detail = host.executions.get(name.clone()).await?;
        assert_eq!(detail.progress, current.value);
        assert_eq!(detail.resolved_at, current.resolved_at);
        let pinned = ProgressGetRequest {
            name: name.clone(),
            at: first.resolved_at.clone(),
        };
        assert_eq!(client.executions.progress.get(tonic::Request::new(pinned.clone())).await?.into_inner(), first);
        let mut unavailable = pinned.clone();
        unavailable.at.as_mut().unwrap().history.as_mut().unwrap().epoch = "another-execution".into();
        assert!(client.executions.progress.get(tonic::Request::new(unavailable)).await.is_err());
        process.close().await?;

        // Reopen Without Losing a Returned Position or Its Value
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = dokime::Client::from_channel(channel, support::client_settings())?;
        assert_eq!(client.executions.progress.get(tonic::Request::new(pinned)).await?.into_inner(), first);
        process.close().await?;
        plugin.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
