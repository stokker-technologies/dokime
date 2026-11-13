/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Collects Backend Diagnostics

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
async fn client_collects_backend_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        use dokime::v1;

        use harmos_protocol::v1 as library;

        // Read Host Identity and Subscribe Before a Component Appears
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = dokime::Client::from_channel(channel.clone(), super::support::client_settings())?;
        let status = client.status.get(tonic::Request::new(library::StatusGetRequest {})).await?.into_inner();
        let initial = status.status.unwrap();
        assert!(initial.available);
        assert_eq!(initial.identity.as_ref().unwrap().application_version, env!("CARGO_PKG_VERSION"));
        let mut status_watch = client.status.watch(tonic::Request::new(library::StatusWatchRequest { at: status.resolved_at })).await?;
        assert_eq!(status_watch.next().await?.unwrap().status, Some(initial.clone()));

        // Secret Values Are Registered Before Host Events Can Retain Them
        let secret = "thermal";
        let mut secrets = v1::secrets_client::SecretsClient::new(channel.clone());
        let generation = secrets.list(v1::SecretsListRequest {}).await?.into_inner().generation;
        let saved = secrets
            .put(v1::SecretsPutRequest {
                metadata: Some(v1::Metadata {
                    identifier: "diagnostics.access".into(),
                    label: "Diagnostics Access".into(),
                    ..Default::default()
                }),
                value: Some(secret.into()),
                generation,
            })
            .await?
            .into_inner();
        let mut logs = client
            .status
            .logs
            .watch(tonic::Request::new(library::status::LogsWatchRequest {
                at: None,
                max_events: 16,
                max_bytes: 4096,
            }))
            .await?;
        assert!(logs.next().await?.unwrap().events.is_empty());
        let mut trace = client
            .status
            .trace_events
            .watch(tonic::Request::new(library::status::TraceEventsWatchRequest {
                at: None,
                max_events: 16,
                max_bytes: 4096,
            }))
            .await?;
        assert!(trace.next().await?.unwrap().events.is_empty());

        // Real Plugin Admission Drives Both Event Histories and Product Health
        let mut plugin =
            dokime_runtime_next::PluginProcess::start(&super::support::thermal_fixture()?, true, &super::support::settings().placement)
                .await?;
        let mut plugins = v1::plugins_client::PluginsClient::new(channel);
        let entry = plugins.add(v1::AddRemotePlugin { url: plugin.endpoint().into() }).await?.into_inner();
        assert!(entry.description.as_ref().unwrap().id.contains(secret));
        loop {
            let frame = logs.next().await?.ok_or("Watch Logs: Closed Before Plugin Admission")?;
            for event in &frame.events {
                assert!(!event.message.contains(secret));
                assert!(event.fields.values().all(|value| !value.contains(secret)));
            }
            if frame.events.iter().any(|event| event.fields.contains_key("plugin")) {
                break;
            }
        }
        let traced = trace.next().await?.ok_or("Watch Trace Events: Closed Before Plugin Admission")?;
        assert!(!traced.events.is_empty());
        assert!(traced.events.iter().flat_map(|event| event.fields.values()).all(|value| !value.contains(secret)));
        loop {
            let changed = status_watch.next().await?.ok_or("Watch Status: Closed Before Plugin Health")?.status.unwrap();
            let admitted = changed.components.len() > initial.components.len();
            let healthy = changed.components.iter().all(|component| component.state == library::ComponentState::Healthy as i32);
            if admitted && healthy {
                break;
            }
        }
        drop((logs, trace, status_watch));

        // Continue a Trace Page at Its Issued Boundary
        let first = client
            .status
            .trace_events
            .list(tonic::Request::new(library::status::TraceEventsListRequest {
                at: None,
                page: Some(library::PageRequest {
                    page_size: 1,
                    max_bytes: 4096,
                    page_token: String::new(),
                }),
            }))
            .await?
            .into_inner();
        assert_eq!(first.events.len(), 1);
        assert!(!first.next_page_token.is_empty());
        let second = client
            .status
            .trace_events
            .list(tonic::Request::new(library::status::TraceEventsListRequest {
                at: first.resolved_at.clone(),
                page: Some(library::PageRequest {
                    page_size: 1,
                    max_bytes: 4096,
                    page_token: first.next_page_token,
                }),
            }))
            .await?
            .into_inner();
        assert_eq!(second.resolved_at, first.resolved_at);
        assert!(second.events[0].sequence > first.events[0].sequence);

        // Resume Bounded Archive Ranges and Verify Their Shared Representation
        let mut name = String::new();
        let mut bytes = Vec::new();
        let digest = loop {
            let range = client
                .status
                .diagnostics
                .get(tonic::Request::new(library::status::DiagnosticsGetRequest {
                    name: name.clone(),
                    offset: bytes.len() as u64,
                    max_bytes: 1024,
                }))
                .await?
                .into_inner();
            if name.is_empty() {
                name = range.name.clone();
            }
            assert_eq!(range.name, name);
            bytes.extend(range.data);
            if bytes.len() as u64 == range.total_length {
                break range.sha256;
            }
        };
        use sha2::Digest;
        assert_eq!(digest, sha2::Sha256::digest(&bytes).to_vec());
        let mut archive = tar::Archive::new(bytes.as_slice());
        let mut paths = Vec::new();
        for entry in archive.entries()? {
            use std::io::Read;
            let mut entry = entry?;
            let path = entry.path()?.into_owned();
            let mut content = String::new();
            entry.read_to_string(&mut content)?;
            assert!(!content.contains(secret));
            if path == std::path::Path::new("dokime/protocol.json") {
                let protocol: serde_json::Value = serde_json::from_str(&content)?;
                assert_eq!(
                    protocol["descriptor_sha256"],
                    serde_json::json!(sha2::Sha256::digest(dokime_protocol::FILE_DESCRIPTOR_SET).to_vec())
                );
            }
            paths.push(path);
        }
        assert!(paths.contains(&std::path::PathBuf::from("dokime/protocol.json")));
        secrets
            .remove(v1::SecretsRemoveRequest {
                identifiers: vec!["diagnostics.access".into()],
                generation: saved.generation,
            })
            .await?;
        process.close().await?;
        plugin.close().await?;

        // Refuse an Incompatible Root While Preserving Every Existing Byte
        let incompatible = super::support::tempdir()?;
        std::fs::write(incompatible.path().join("existing-data"), b"preserve")?;
        let output = tokio::process::Command::from(dokime_runtime_next::process::command(super::support::host_fixture()?))
            .args(["--endpoint", "127.0.0.1:0", "--storage"])
            .arg(incompatible.path())
            .output()
            .await?;
        assert!(!output.status.success());
        assert_eq!(std::fs::read(incompatible.path().join("existing-data"))?, b"preserve");
        assert!(matches!(dokime_storage::Store::open(incompatible.path()), Err(dokime_storage::Error::Incompatible(_))));
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
