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
use std::sync::Arc;

// Third Party Imports
use harmos_protocol::host::ComponentState;
use tokio::sync::mpsc::UnboundedSender;
use tonic::transport::Endpoint;

// Local Imports
use dokime::Client;
use dokime::client::StatusWatchOptions;
use dokime_runtime::ControlledClock;
use dokime_runtime::SchedulerClock;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct ObservedClock {
    clock: Arc<ControlledClock>,
    waits: UnboundedSender<i64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[async_trait::async_trait]
impl SchedulerClock for ObservedClock {
    fn now_ns(&self) -> i64 {
        self.clock.now_ns()
    }
    async fn wait_until(&self, at_ns: i64) -> dokime_runtime::Result<()> {
        self.clock.wait_until(at_ns).await
    }
    async fn reached(&self, at_ns: i64) {
        let _ = self.waits.send(at_ns);
        self.clock.reached(at_ns).await;
    }
}

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

        for id in [
            "dokime.host.v1.evidence.Records/Append",
            "dokime.host.v1.Files/Capture",
            "dokime.host.v1.files.Captures/Start",
            "dokime.host.v1.files.Captures/Stop",
            "dokime.host.v1.Effects/Create",
            "dokime.host.v1.Effects/Get",
            "dokime.host.v1.Effects/Set",
        ] {
            let contract = client.operations.get(id).await?.contract.ok_or("Find Plugin-Only Contract")?;
            let refused = client.invocations.create(id, contract.fingerprint.clone(), Vec::new()).await.unwrap_err();
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
fn library_time_uses_the_execution_clock() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        use dokime_runtime::{ControlledClock, SchedulerClock, services};
        use prost_types::Duration;
        use std::sync::Arc;
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Compose the Library Clock With an Observed Controlled Scheduler
            let root = super::support::tempdir()?;
            let clock = Arc::new(ControlledClock::new(0));
            let (waits, mut waiting) = tokio::sync::mpsc::unbounded_channel();
            let mut settings = super::support::settings();
            settings.clock = Arc::new(ObservedClock { clock: clock.clone(), waits });

            let server = services::serve("127.0.0.1:0".parse()?, root.path(), settings, super::support::service_settings()).await?;

            let channel = Endpoint::from_shared(format!("http://{}", server.endpoint()))?.connect().await?;
            let client = Client::from_channel(channel, super::support::client_settings())?;

            let before = client.time.get().await?;
            assert_eq!((before.seconds, before.nanos), (0, 0));

            // Advance Only After the Library Has Registered This Sleep
            let sleeping = client.clone();
            let sleep = tokio::spawn(async move { sleeping.time.sleep(Duration { seconds: 7, nanos: 0 }).await });
            while waiting.recv().await.ok_or("Sleep Through Library Time: Missing Scheduler Wait")? != 7_000_000_000 {}

            clock.wait_until(7_000_000_000).await?;

            sleep.await??;
            let after = client.time.get().await?;
            assert_eq!((after.seconds, after.nanos), (7, 0));

            server.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
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
            assert!(status.status.as_ref().unwrap().available);
            assert!(status.status.as_ref().unwrap().components.iter().all(|component| component.state == ComponentState::Healthy as i32));

            // Watch the Same Status Snapshot Through the Library Root
            let mut watch = client
                .status
                .watch_with(StatusWatchOptions {
                    at: status.resolved_at.clone(),
                    ..Default::default()
                })
                .await?;
            let opening = watch.next().await?.ok_or("Watch Status: Missing Opening Snapshot")?;
            assert_eq!(opening.status, status.status);
            drop(watch);
            process.close().await?;

            // Reopen the Same Root with a New History and Stable Identity
            let process =
                super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment)
                    .await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, super::support::client_settings())?;
            let reopened = client.status.get().await?;
            assert_eq!(reopened.status.unwrap().identity.unwrap().host_id, identity.host_id);
            assert_ne!(reopened.resolved_at.unwrap().history.unwrap().epoch, status.resolved_at.unwrap().history.unwrap().epoch);
            process.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}
