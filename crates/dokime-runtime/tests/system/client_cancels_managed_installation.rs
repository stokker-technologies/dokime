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
async fn client_cancels_managed_installation() -> Result<(), Box<dyn Error>> {
    tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
        // Await a Startup Event From the Owned Managed Child
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let executable = std::fs::read(support::content_fixture()?)?;
        let bytes = support::bundles::bundle(&executable, "content", "1.0.0")?;
        let root = support::tempdir()?;
        let credential_root = support::tempdir()?;
        let credentials = support::CredentialStore::start(credential_root.path()).await?;
        let mut environment = credentials.environment.clone();
        environment.push(("DOKIME_PENDING_INSTALL", listener.local_addr()?.to_string().into()));
        let process = HostProcess::start_with_environment(root.path(), &environment).await?;
        let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = Client::from_channel(channel, support::client_settings())?;
        let content = support::bundles::upload(&client, bytes.clone()).await?;
        let placement = PluginPlacement {
            kind: Some(PlacementKind::Managed(PluginManaged {
                content: Some(content),
                sha256: Sha256::digest(&bytes).to_vec(),
            })),
        };
        let installing = client.clone();
        let mut task = tokio::spawn(async move { installing.plugins.create("content", placement, PluginValue { enabled: true }).await });
        let (mut child, _) = tokio::select! {
            accepted = listener.accept() => accepted?,
            result = &mut task => return Err(format!("Create Managed Plugin: Settled Before Child Startup: {result:?}").into()),
        };
        assert_eq!(child.read_u8().await?, 1);

        // Cancellation Reaps the Child and Completes Rollback Before the Final Read
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        let mut remaining = Vec::new();
        match child.read_to_end(&mut remaining).await {
            Ok(_) => (),
            Err(error) if matches!(error.kind(), ErrorKind::ConnectionReset | ErrorKind::BrokenPipe) => (),
            Err(error) => return Err(error.into()),
        }
        assert!(remaining.is_empty());
        assert!(client.plugins.list().await?.entries.is_empty());
        process.close().await?;
        Ok::<(), Box<dyn Error>>(())
    })
    .await?
}
