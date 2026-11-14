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
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use super::support;
use dokime::Client;
use dokime::EvidenceScope;
use dokime::MapValue;
use dokime::client::CyclesCreateOptions;
use dokime::client::EvidenceTraceabilityGetOptions;
use dokime::client::EvidenceTraceabilityWatchOptions;
use dokime::client::ExecutionsListOptions;
use dokime::client::ExecutionsProgressWatchOptions;
use dokime::client::PluginPlacement;
use dokime::client::PluginRemote;
use dokime::client::PluginValue;
use dokime_protocol::host::evidence::TraceabilityGetRequest;
use dokime_runtime::PluginProcess;
use harmos_protocol::host::plugin_placement::Kind as PluginPlacementKind;
use harmos_protocol::model::PageRequest;
use harmos_protocol::model::ReadCut;
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

#[tokio::test]
async fn client_observes_revisioned_traceability() -> Result<(), Box<dyn Error>> {
    tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
        // Start the Host and Run a Real Plugin Until Its Prompt
        let root = support::tempdir()?;
        let credential_root = support::tempdir()?;
        let credentials = support::CredentialStore::start(credential_root.path()).await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let settings = support::settings();
        let mut plugin = PluginProcess::start(&support::content_fixture()?, true, &settings.placement).await?;
        let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
        let host = Client::from_channel(channel.clone(), support::client_settings())?;
        let client = Client::from_channel(channel.clone(), support::client_settings())?;
        host.plugins
            .create(
                "content",
                PluginPlacement {
                    kind: Some(PluginPlacementKind::Remote(PluginRemote { url: plugin.endpoint().into() })),
                },
                PluginValue { enabled: true },
            )
            .await?
            .entry
            .unwrap();
        let cases = super::support::materialize(&host, "content", "content.traceability", HashMap::new()).await?;
        let cycle = Client::from_channel(channel.clone(), super::support::client_settings())?
            .cycles
            .clone()
            .create_with(
                cases.into_iter().map(|case| case.origin.unwrap()).collect(),
                Default::default(),
                true,
                String::new(),
                CyclesCreateOptions {
                    request_id: Some(uuid::Uuid::now_v7().to_string()),
                    ..Default::default()
                },
            )
            .await?;
        let admitted = Client::from_channel(channel.clone(), super::support::client_settings())?
            .executions
            .clone()
            .list_with(
                cycle.cycle.unwrap().cycle_id,
                Default::default(),
                Default::default(),
                Default::default(),
                ExecutionsListOptions {
                    at: Some(ReadCut {
                        positions: vec![cycle.receipt.unwrap().durable_at.unwrap()],
                        revisions: Vec::new(),
                    }),
                    ..Default::default()
                },
            )
            .await?;
        let name = admitted.executions[0].summary.as_ref().unwrap().execution_id.clone();
        let mut progress =
            client.executions.progress.watch_with(name.clone(), ExecutionsProgressWatchOptions { at: None, ..Default::default() }).await?;
        loop {
            if progress.next().await?.ok_or("Watch Traceability: Missing Prompt")?.phase == "prompt" {
                break;
            }
        }
        let scope = EvidenceScope {
            execution_id: Some(name.clone()),
            template_id: Some("content.traceability".into()),
            suite_id: Some("content".into()),
            ..Default::default()
        };

        // Page a Stable Opening and Reject Continuations for a Different Selection
        let request = TraceabilityGetRequest {
            scope: Some(scope.clone()),
            page: Some(PageRequest { page_size: 1, ..Default::default() }),
            at: None,
            ..Default::default()
        };
        let first = client
            .evidence
            .traceability
            .get_with(
                ((request.clone()).scope).unwrap_or_default(),
                EvidenceTraceabilityGetOptions {
                    page: (request.clone()).page,
                    at: (request.clone()).at,
                    entry: (request.clone()).entry,
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(first.snapshots.len(), 1);
        assert!(!first.next_page_token.is_empty());
        assert_eq!(first.snapshots[0].entries[0].metadata.as_ref().unwrap().identifier, "serial");
        assert!(first.snapshots[0].scope.as_ref().unwrap().origin.is_some());
        let continuation = TraceabilityGetRequest {
            at: first.resolved_at.clone(),
            page: Some(PageRequest {
                page_size: 1,
                page_token: first.next_page_token.clone(),
                ..Default::default()
            }),
            ..request.clone()
        };
        let second = client
            .evidence
            .traceability
            .get_with(
                ((continuation.clone()).scope).unwrap_or_default(),
                EvidenceTraceabilityGetOptions {
                    page: (continuation.clone()).page,
                    at: (continuation.clone()).at,
                    entry: (continuation.clone()).entry,
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(second.snapshots[0].entries[0].metadata.as_ref().unwrap().identifier, "batch");
        assert!(second.next_page_token.is_empty());
        let mut wrong = continuation;
        wrong.scope.as_mut().unwrap().step_id = Some("different-step".into());
        assert!(
            client
                .evidence
                .traceability
                .get_with(
                    ((wrong.clone()).scope).unwrap_or_default(),
                    EvidenceTraceabilityGetOptions {
                        page: (wrong.clone()).page,
                        at: (wrong.clone()).at,
                        entry: (wrong.clone()).entry,
                        ..Default::default()
                    }
                )
                .await
                .is_err()
        );

        let mut tiny = request.clone();
        tiny.page.as_mut().unwrap().max_bytes = 1;
        assert!(
            client
                .evidence
                .traceability
                .get_with(
                    ((tiny.clone()).scope).unwrap_or_default(),
                    EvidenceTraceabilityGetOptions {
                        page: (tiny.clone()).page,
                        at: (tiny.clone()).at,
                        entry: (tiny.clone()).entry,
                        ..Default::default()
                    }
                )
                .await
                .is_err()
        );

        // Observe the Replacement and Cancel Only the Observer
        let mut watch = client
            .evidence
            .traceability
            .watch_with(
                scope,
                EvidenceTraceabilityWatchOptions {
                    page: None,
                    at: first.resolved_at.clone(),
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(watch.next().await?.unwrap().snapshots.len(), 2);
        let prompt = host.prompts.list(name.clone(), true).await?.prompts[0].prompt.as_ref().unwrap().prompt_id.clone();
        host.prompts.answers.create(prompt, "continue", MapValue::default(), true).await?;
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
        assert_eq!(
            client
                .evidence
                .traceability
                .get_with(
                    ((pinned.clone()).scope).unwrap_or_default(),
                    EvidenceTraceabilityGetOptions {
                        page: (pinned.clone()).page,
                        at: (pinned.clone()).at,
                        entry: (pinned.clone()).entry,
                        ..Default::default()
                    }
                )
                .await?,
            first
        );
        let mut foreign = pinned.clone();
        foreign.at.as_mut().unwrap().positions[0].history.as_mut().unwrap().store = "another-store".into();
        assert!(
            client
                .evidence
                .traceability
                .get_with(
                    ((foreign.clone()).scope).unwrap_or_default(),
                    EvidenceTraceabilityGetOptions {
                        page: (foreign.clone()).page,
                        at: (foreign.clone()).at,
                        entry: (foreign.clone()).entry,
                        ..Default::default()
                    }
                )
                .await
                .is_err()
        );
        process.close().await?;

        // Retain the Same Scoped Page Across a Host Restart
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = Client::from_channel(channel, support::client_settings())?;
        let restored = client
            .evidence
            .traceability
            .get_with(
                ((pinned.clone()).scope).unwrap_or_default(),
                EvidenceTraceabilityGetOptions {
                    page: (pinned.clone()).page,
                    at: (pinned.clone()).at,
                    entry: (pinned.clone()).entry,
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(restored.snapshots[0].entries, first.snapshots[0].entries);
        assert_eq!(restored.resolved_at, first.resolved_at);
        process.close().await?;
        plugin.close().await?;
        Ok::<(), Box<dyn Error>>(())
    })
    .await?
}
