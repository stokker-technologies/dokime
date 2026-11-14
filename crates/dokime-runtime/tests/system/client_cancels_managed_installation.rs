/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Acknowledged Installation Cancellation Reaps the Child Before Rollback

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
async fn client_cancels_managed_installation() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
                use tokio::io::AsyncReadExt;

        // Prepare a Real Executable and an Awaited Startup Event.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let executable = std::fs::read(super::support::content_fixture()?)?;
        let bytes = super::support::bundles::bundle(&executable, "1.0.0")?;
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let mut environment = credentials.environment.clone();
        environment.push(("DOKIME_PENDING_INSTALL", listener.local_addr()?.to_string().into()));
        let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &environment).await?;
        let mut plugins = dokime_protocol::client::plugins_client::PluginsClient::connect(process.url.clone()).await?;

        // Cancel the Ordinary Installation Transport After Managed Startup.
        let frames=super::support::bundles::upload(bytes);
        let mut task=tokio::spawn(async move {plugins.install(tokio_stream::iter(frames)).await});
        let (mut child,_)=tokio::select! {accepted=listener.accept()=>accepted?,result=&mut task=>return Err(format!("Installation Settled Before Child Startup: {result:?}").into())};
        assert_eq!(child.read_u8().await?,1);
        task.abort();assert!(task.await.unwrap_err().is_cancelled());
        let mut plugins=dokime_protocol::client::plugins_client::PluginsClient::connect(process.url.clone()).await?;
        let mut remaining = Vec::new();
        match child.read_to_end(&mut remaining).await {
            Ok(_) => (),
            Err(error) if matches!(error.kind(), std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::BrokenPipe) => (),
            Err(error) => return Err(error.into()),
        }
        assert!(remaining.is_empty());

        // Rollback Has Removed Published Entries and Installed Files on Every Platform.
        assert!(dokime_protocol::client::plugins_client::PluginsClient::connect(process.url.clone()).await?.list(dokime_protocol::client::ListPlugins {}).await?.into_inner().plugins.is_empty());
        assert_eq!(plugins.list_installations(dokime_protocol::client::PluginsListInstallationsRequest::default()).await?.into_inner().total_count, 0);
        assert_eq!(std::fs::read_dir(root.path().join("plugins"))?.count(), 0);
        process.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
