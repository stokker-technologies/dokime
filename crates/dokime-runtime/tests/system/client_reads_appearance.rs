/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Reads Appearance

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::Client;
use dokime::client::BackendAppearanceOptions;
use dokime::client::ThemeModePolicy;
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
async fn client_reads_appearance() -> Result<(), Box<dyn Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Open the Desktop Appearance on a Fresh Host
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let appearance =
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .backend
                .clone();
        let snapshot =
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .backend
                .clone()
                .appearance()
                .await?
                .appearance
                .unwrap();
        assert!(!snapshot.app_name.trim().is_empty());
        assert!(ThemeModePolicy::try_from(snapshot.mode_policy).is_ok());
        let reply = appearance.appearance().await?;
        let replay = appearance
            .appearance_with(BackendAppearanceOptions {
                at: reply.resolved_at.clone(),
                ..Default::default()
            })
            .await?;
        assert_eq!(reply, replay);
        process.close().await?;
        Ok::<(), Box<dyn Error>>(())
    })
    .await?
}
