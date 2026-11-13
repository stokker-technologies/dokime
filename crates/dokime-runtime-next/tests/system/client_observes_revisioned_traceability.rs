/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugin Traceability Has Independent Revisions and Paged Retained Reads

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{client_runs_thermal_suite_over_grpc::HostProcess, support};
use dokime::v1::{
    self,
    evidence::{TraceabilityGetRequest, TraceabilityWatchRequest},
};

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
async fn client_observes_revisioned_traceability() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
        // Start the Host and Run a Real Plugin Until Its Prompt
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
        let cases = host.templates.materialize("content", "content.traceability", std::collections::HashMap::new()).await?;
        let cycle = host.cycles.start(cases).await?;
        let name = cycle.executions[0].clone();
        let mut progress = client
            .executions
            .progress
            .watch(tonic::Request::new(v1::executions::ProgressWatchRequest { name: name.clone(), at: None }))
            .await?;
        loop {
            if progress.next().await?.ok_or("Watch Traceability: Missing Prompt")?.phase == "prompt" {
                break;
            }
        }
        let scope = v1::EvidenceScope {
            execution_id: Some(name.clone()),
            template_id: Some("content.traceability".into()),
            suite_id: Some("content".into()),
            ..Default::default()
        };

        // Page a Stable Opening and Reject Continuations for a Different Selection
        let request = TraceabilityGetRequest {
            scope: Some(scope.clone()),
            page: Some(harmos_protocol::v1::PageRequest { page_size: 1, ..Default::default() }),
            at: None,
        };
        let first = client.evidence.traceability.get(tonic::Request::new(request.clone())).await?.into_inner();
        assert_eq!(first.snapshots.len(), 1);
        assert!(!first.next_page_token.is_empty());
        assert_eq!(first.snapshots[0].entries[0].metadata.as_ref().unwrap().identifier, "serial");
        assert!(first.snapshots[0].scope.as_ref().unwrap().origin.is_some());
        let continuation = TraceabilityGetRequest {
            at: first.resolved_at.clone(),
            page: Some(harmos_protocol::v1::PageRequest {
                page_size: 1,
                page_token: first.next_page_token.clone(),
                ..Default::default()
            }),
            ..request.clone()
        };
        let second = client.evidence.traceability.get(tonic::Request::new(continuation.clone())).await?.into_inner();
        assert_eq!(second.snapshots[0].entries[0].metadata.as_ref().unwrap().identifier, "batch");
        assert!(second.next_page_token.is_empty());
        let mut wrong = continuation;
        wrong.scope.as_mut().unwrap().step_id = Some("different-step".into());
        assert!(client.evidence.traceability.get(tonic::Request::new(wrong)).await.is_err());

        let mut tiny = request.clone();
        tiny.page.as_mut().unwrap().max_bytes = 1;
        assert!(client.evidence.traceability.get(tonic::Request::new(tiny)).await.is_err());

        // Observe the Replacement and Cancel Only the Observer
        let mut watch = client
            .evidence
            .traceability
            .watch(tonic::Request::new(TraceabilityWatchRequest {
                scope: Some(scope),
                page: None,
                at: first.resolved_at.clone(),
            }))
            .await?;
        assert_eq!(watch.next().await?.unwrap().snapshots.len(), 2);
        let prompt = host.prompts.pending(name.clone()).await?.prompts[0].prompt_id.clone();
        host.prompts.respond(prompt, "continue", true).await?;
        loop {
            let frame = watch.next().await?.ok_or("Watch Traceability: Missing Replacement")?;
            if let Some(row) = frame.snapshots.iter().find(|row| row.entries[0].metadata.as_ref().unwrap().identifier == "serial") {
                assert!(!frame.opening);
                assert!(row.entries[0].source.is_empty() && row.entries[0].unit.is_empty());
                assert!(row.entries[0].metadata.as_ref().unwrap().label.is_empty());
                assert!(row.entries[0].metadata.as_ref().unwrap().attributes.is_empty());
                assert_ne!(row.revision, first.snapshots[0].revision);
                break;
            }
        }
        drop(watch);
        while progress.next().await?.is_some() {}
        let pinned = TraceabilityGetRequest {
            at: first.resolved_at.clone(),
            ..request.clone()
        };
        assert_eq!(client.evidence.traceability.get(tonic::Request::new(pinned.clone())).await?.into_inner(), first);
        let mut foreign = pinned.clone();
        foreign.at.as_mut().unwrap().positions[0].history.as_mut().unwrap().store = "another-store".into();
        assert!(client.evidence.traceability.get(tonic::Request::new(foreign)).await.is_err());
        process.close().await?;

        // Retain the Same Scoped Page Across a Host Restart
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = dokime::Client::from_channel(channel, support::client_settings())?;
        let restored = client.evidence.traceability.get(tonic::Request::new(pinned)).await?.into_inner();
        assert_eq!(restored.snapshots[0].entries, first.snapshots[0].entries);
        assert_eq!(restored.resolved_at, first.resolved_at);
        process.close().await?;
        plugin.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
