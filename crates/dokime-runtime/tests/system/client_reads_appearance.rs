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
async fn client_reads_appearance() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Open the Desktop Appearance on a Fresh Host
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let mut appearance = dokime_protocol::client::backend_client::BackendClient::connect(process.url.clone()).await?;
        let snapshot = dokime_protocol::client::backend_client::BackendClient::connect(process.url.clone())
            .await?
            .appearance(dokime_protocol::client::BackendAppearanceRequest { ..Default::default() })
            .await?
            .into_inner()
            .appearance
            .unwrap();
        assert!(!snapshot.app_name.trim().is_empty());
        assert!(dokime_protocol::client::ThemeModePolicy::try_from(snapshot.mode_policy).is_ok());
        let reply = appearance.appearance(dokime_protocol::client::BackendAppearanceRequest::default()).await?.into_inner();
        let replay =
            appearance.appearance(dokime_protocol::client::BackendAppearanceRequest { at: reply.resolved_at.clone() }).await?.into_inner();
        assert_eq!(reply, replay);
        process.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
