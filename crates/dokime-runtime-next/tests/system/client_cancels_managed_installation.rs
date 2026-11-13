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
        use dokime::v1;
        use tokio::io::AsyncReadExt;

        // Prepare a Real Executable and an Awaited Startup Event.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let executable = std::fs::read(super::support::content_fixture()?)?;
        let bytes = super::support::bundles::bundle(&executable, "1.0.0")?;
        let root = tempfile::tempdir()?;
        let credential_root = tempfile::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let mut environment = credentials.environment.clone();
        environment.push(("DOKIME_PENDING_INSTALL", listener.local_addr()?.to_string().into()));
        let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &environment).await?;
        let mut plugins = v1::plugins_client::PluginsClient::connect(process.url.clone()).await?;

        // Upload the Verified Bundle and Keep the Cancellation Direction Open.
        let (sender, receiver) = tokio::sync::mpsc::channel(8);
        let mut output = plugins.run_install(tokio_stream::wrappers::ReceiverStream::new(receiver)).await?.into_inner();
        for frame in super::support::bundles::upload(bytes) {
            sender
                .send(v1::PluginsInstallRunInput {
                    value: frame.value.map(|value| match value {
                        v1::plugins_install_input::Value::Request(request) => v1::plugins_install_run_input::Value::Request(request),
                        v1::plugins_install_input::Value::Item(item) => v1::plugins_install_run_input::Value::Item(item),
                    }),
                })
                .await?;
        }
        sender
            .send(v1::PluginsInstallRunInput {
                value: Some(v1::plugins_install_run_input::Value::Finish(v1::ClientEmpty {})),
            })
            .await?;
        let (mut child, _) = tokio::select! {
            accepted = listener.accept() => accepted?,
            result = output.message() => return Err(format!("Installation Settled Before Child Startup: {result:?}").into()),
        };
        assert_eq!(child.read_u8().await?, 1);

        // Cancel After Startup and Await Both Settlement and Child Connection Closure.
        sender
            .send(v1::PluginsInstallRunInput {
                value: Some(v1::plugins_install_run_input::Value::Cancel(v1::ClientEmpty {})),
            })
            .await?;
        let error = output.message().await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::Cancelled);
        let mut remaining = Vec::new();
        match child.read_to_end(&mut remaining).await {
            Ok(_) => (),
            Err(error) if matches!(error.kind(), std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::BrokenPipe) => (),
            Err(error) => return Err(error.into()),
        }
        assert!(remaining.is_empty());

        // Rollback Has Removed Published Entries and Installed Files on Every Platform.
        assert!(plugins.list(v1::ListPlugins {}).await?.into_inner().plugins.is_empty());
        assert_eq!(plugins.list_installations(v1::PluginsListInstallationsRequest::default()).await?.into_inner().total_count, 0);
        assert_eq!(std::fs::read_dir(root.path().join("plugins"))?.count(), 0);
        process.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
