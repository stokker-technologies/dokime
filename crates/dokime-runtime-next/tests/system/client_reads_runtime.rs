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
async fn client_reads_runtime() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Start a Real Host and Inspect Desktop Bootstrap Identity
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = dokime::Client::from_channel(channel, super::support::client_settings())?;
        let status = client.status.get(tonic::Request::new(harmos_protocol::v1::StatusGetRequest {})).await?.into_inner();
        let identity = status.status.as_ref().unwrap().identity.as_ref().unwrap();
        assert!(status.status.as_ref().unwrap().available);
        assert!(
            status
                .status
                .as_ref()
                .unwrap()
                .components
                .iter()
                .all(|component| component.state == harmos_protocol::v1::ComponentState::Healthy as i32)
        );

        // Watch the Same Status Snapshot Through the Library Root
        let mut watch =
            client.status.watch(tonic::Request::new(harmos_protocol::v1::StatusWatchRequest { at: status.resolved_at.clone() })).await?;
        let opening = watch.next().await?.ok_or("Watch Status: Missing Opening Snapshot")?;
        assert_eq!(opening.status, status.status);
        drop(watch);
        process.close().await?;

        // Reopen the Same Root with a New History and Stable Identity
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = dokime::Client::from_channel(channel, super::support::client_settings())?;
        let reopened = client.status.get(tonic::Request::new(harmos_protocol::v1::StatusGetRequest {})).await?.into_inner();
        assert_eq!(reopened.status.unwrap().identity.unwrap().host_id, identity.host_id);
        assert_ne!(reopened.resolved_at.unwrap().history.unwrap().epoch, status.resolved_at.unwrap().history.unwrap().epoch);
        process.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
