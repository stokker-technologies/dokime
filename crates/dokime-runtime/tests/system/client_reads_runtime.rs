/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Reads Runtime

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::error::Error;

// Third Party Imports
use harmos_protocol::host::ComponentState;
use harmos_protocol::model::PageRequest;
use harmos_runtime::operations::OperationFactory;
use tonic::transport::Endpoint;

// Local Imports
use dokime_client_internal::Client;
use dokime_client_internal::StatusLogsListOptions;
use dokime_client_internal::StatusTraceEventsListOptions;
use dokime_client_internal::StatusWatchOptions;
use dokime_operations::host;
use dokime_protocol::host::StatusGetResponse;

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
fn encoded_invocations_share_product_handlers_and_enforce_callers() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        use dokime_protocol::host::{CyclesCreateRequest, CyclesCreateResponse};
        use prost::Message;
        use tonic::Code;
        let root = super::support::tempdir()?;
        let server = dokime_runtime::services::serve(
            "127.0.0.1:0".parse()?,
            root.path(),
            super::support::settings(),
            super::support::service_settings(),
        )
        .await?;
        let channel = Endpoint::from_shared(format!("http://{}", server.endpoint()))?.connect().await?;
        let client = Client::from_channel(channel, super::support::client_settings())?;
        let id = "dokime.host.v1.Cycles/Create";
        let contract = client.operations.get(id).await?.contract.ok_or("Discover Cycle Creation: Missing Contract")?;
        let payload = client
            .invocations
            .create(
                id,
                contract.fingerprint,
                CyclesCreateRequest {
                    request_id: "encoded-cycle".into(),
                    ..Default::default()
                }
                .encode_to_vec(),
            )
            .await?;
        let created = CyclesCreateResponse::decode(payload.as_slice())?;
        let cycle = created.cycle.ok_or("Create Encoded Cycle: Missing Cycle")?;
        let direct = client.cycles.get(cycle.cycle_id.clone()).await?;
        assert_eq!(direct.cycle.as_ref().map(|cycle| &cycle.cycle_id), Some(&cycle.cycle_id));

        // Use Local Contracts for Operations Excluded from Client Discovery
        for factory in [
            OperationFactory::protobuf::<host::evidence::records::append::Operation>()?,
            OperationFactory::protobuf::<host::files::capture::Operation>()?,
            OperationFactory::protobuf::<host::files::captures::start::Operation>()?,
            OperationFactory::protobuf::<host::files::captures::stop::Operation>()?,
        ] {
            let contract = factory.contract()?;
            let id = contract.operation().id.as_str();
            let refused = client.invocations.create(id, contract.fingerprint().0.to_vec(), Vec::new()).await.unwrap_err();
            assert_eq!(refused.code(), Code::PermissionDenied);
        }
        for id in [
            "dokime.host.v1.Aggregates/Get",
            "dokime.host.v1.media.Captures/Create",
            "dokime.host.v1.media.Captures/Get",
            "dokime.host.v1.media.Captures/Stop",
        ] {
            let contract = client.operations.get(id).await?.contract.ok_or("Discover Unsupported Operation: Missing Contract")?;
            let refused = client.invocations.create(id, contract.fingerprint, Vec::new()).await.unwrap_err();
            assert_eq!(refused.code(), Code::Unimplemented);
        }
        server.close().await?;
        Ok(())
    }))
}

#[test]
fn registered_backend_retains_uploaded_content_after_restart() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        use super::{client_runs_thermal_suite_over_grpc::HostProcess, support};
        use sha2::{Digest, Sha256};
        tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Publish Through Product Files Backed by the Library Storage Manager
            let root = support::tempdir()?;
            let credential_root = support::tempdir()?;
            let credentials = support::CredentialStore::start(credential_root.path()).await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, support::client_settings())?;
            let bytes = (0..4096).map(|index| (index % 251) as u8).collect::<Vec<_>>();
            let file = support::bundles::upload(&client, bytes.clone()).await?;
            let first = client.files.read(file.clone(), 0, bytes.len() as u64).await?;
            assert_eq!(first.data, bytes);
            assert_eq!(first.sha256, Sha256::digest(&bytes).to_vec());
            process.close().await?;

            // Reopen the Same Backend and Read the Original Durable Reference
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, support::client_settings())?;
            let retained = client.files.get(file.clone()).await?;
            assert_eq!(retained.file, Some(file.clone()));
            let reopened = client.files.read(file.clone(), 0, bytes.len() as u64).await?;
            assert_eq!(reopened.data, first.data);
            assert_eq!(reopened.sha256, first.sha256);
            // Deletion Resolves Ownership From Storage, Even If the Client Changes the Scope.
            let mut deletion = file.clone();
            deletion.metadata.as_mut().unwrap().scope = Some("execution/untrusted-selector".into());
            client.files.delete(deletion).await?;
            assert!(client.files.read(file, 0, bytes.len() as u64).await.is_err());
            process.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}

#[test]
fn client_reads_runtime() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Start a Real Host and Inspect Desktop Bootstrap Identity
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
            let process =
                super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment)
                    .await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, super::support::client_settings())?;
            let status = client.status.get().await?;
            let identity = status.status.as_ref().unwrap().identity.as_ref().unwrap();
            let build = status.build.as_ref().unwrap();
            assert_eq!(build.source_revision, env!("DOKIME_SOURCE_REVISION"));
            assert_eq!(build.channel, env!("DOKIME_BUILD_CHANNEL"));
            assert!(status.status.as_ref().unwrap().available);
            assert!(status.status.as_ref().unwrap().components.iter().all(|component| component.state == ComponentState::Healthy as i32));

            // Watch the Same Product Status and Build Snapshot
            let mut watch = client
                .status
                .watch_with(StatusWatchOptions {
                    at: status.resolved_at.clone(),
                    ..Default::default()
                })
                .await?;
            let opening = watch.next().await?.ok_or("Watch Status: Missing Opening Snapshot")?;
            assert_eq!(opening.status, status.status);
            assert_eq!(opening.build, status.build);
            drop(watch);
            process.close().await?;

            // Reopen the Same Root with a New History and Stable Identity
            let process =
                super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment)
                    .await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, super::support::client_settings())?;
            let reopened = client.status.get().await?;
            assert_eq!(reopened.build, status.build);
            assert_eq!(reopened.status.unwrap().identity.unwrap().host_id, identity.host_id);
            assert_ne!(reopened.resolved_at.unwrap().history.unwrap().epoch, status.resolved_at.unwrap().history.unwrap().epoch);
            process.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}

#[test]
fn clients_reach_every_wrapper_service() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        // Assemble the Product Listener and Call Its Generated Client Resources.
        let root = super::support::tempdir()?;
        let server = dokime_runtime::services::serve(
            "127.0.0.1:0".parse()?,
            root.path(),
            super::support::settings(),
            super::support::service_settings(),
        )
        .await?;
        let channel = Endpoint::from_shared(format!("http://{}", server.endpoint()))?.connect().await?;
        let client = Client::from_channel(channel, super::support::client_settings())?;

        // Enable an Ephemeral Listener and Exercise Pairing and Certificate Administration.
        let listener = client.access.listener.get().await?;
        let mut value = listener.value.unwrap();
        value.enabled = true;
        value.endpoint = "127.0.0.1:0".into();
        client.access.listener.set(value, listener.revision).await?;
        let pairing = client.access.pairings.create("Wrapper Audience").await?;
        let pairing = pairing.pairing.unwrap();
        assert!(client.access.pairings.list().await?.pairings.iter().any(|entry| entry.id == pairing.id));
        client.access.pairings.delete(pairing.id).await?;
        assert!(!client.access.certificates.create().await?.is_empty());

        // Exercise All Status Services and Their Opening Frames.
        assert!(client.status.get().await?.status.is_some());
        let mut status = client.status.watch().await?;
        assert!(status.next().await?.is_some());
        drop(status);
        client
            .status
            .logs
            .list_with(StatusLogsListOptions {
                page: Some(PageRequest {
                    page_size: 1,
                    max_bytes: 4096,
                    page_token: String::new(),
                }),
                ..Default::default()
            })
            .await?;
        let mut logs = client.status.logs.watch(4096, 16).await?;
        assert!(logs.next().await?.is_some());
        drop(logs);
        client
            .status
            .trace_events
            .list_with(StatusTraceEventsListOptions {
                page: Some(PageRequest {
                    page_size: 1,
                    max_bytes: 4096,
                    page_token: String::new(),
                }),
                ..Default::default()
            })
            .await?;
        let mut traces = client.status.trace_events.watch(4096, 16).await?;
        assert!(traces.next().await?.is_some());
        drop(traces);
        assert!(!client.status.diagnostics.get(String::new(), 0, 1024).await?.name.is_empty());

        // Discover and Invoke the Product Status Wrapper Through Native Invocation Ownership.
        let discovered = client.operations.list().await?;
        assert!(!discovered.is_empty());
        assert!(
            discovered
                .iter()
                .filter_map(|operation| operation.contract.as_ref())
                .all(|contract| { contract.callers != harmos_protocol::model::Callers::Plugins as i32 })
        );
        let id = "dokime.host.v1.Status/Get";
        let contract = client.operations.get(id).await?.contract.unwrap();
        let payload = client.invocations.create(id, contract.fingerprint, Vec::new()).await?;
        let status = <StatusGetResponse as prost::Message>::decode(payload.as_slice())?;
        assert!(status.status.is_some());
        server.close().await?;
        Ok(())
    }))
}
