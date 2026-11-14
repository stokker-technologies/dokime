/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Edits Settings

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct SettingsPlugin(InstanceGetResponse);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl Instance for SettingsPlugin {
    async fn get(&self, _: Request<InstanceGetRequest>) -> Result<Response<InstanceGetResponse>, Status> {
        Ok(Response::new(self.0.clone()))
    }
    type InvokeStream = Empty<Result<InstanceInvokeResponse, Status>>;
    async fn invoke(&self, _: Request<Streaming<InstanceInvokeRequest>>) -> Result<Response<Self::InvokeStream>, Status> {
        Err(Status::unimplemented("Invoke Settings Fixture: No Operations"))
    }
    type SamplesStream = Empty<Result<InstanceSamplesResponse, Status>>;
    async fn samples(&self, _: Request<Streaming<InstanceSamplesRequest>>) -> Result<Response<Self::SamplesStream>, Status> {
        Err(Status::unimplemented("Sample Settings Fixture: No Streams"))
    }
    async fn stop(&self, _: Request<InstanceStopRequest>) -> Result<Response<InstanceStopResponse>, Status> {
        Ok(Response::new(InstanceStopResponse {}))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn client_edits_settings() -> Result<(), Box<dyn Error>> {
    tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
        // Declare Typed Library Settings on a Real Plugin Carrier
        let declarations = SettingsDeclarations {
            values: vec![SettingDeclaration {
                name: "enabled".into(),
                title: "Enabled".into(),
                description: String::new(),
                default: Some(SettingValue { value: Some(SettingKind::Flag(true)) }),
            }],
        };
        let catalog = Plugin::new("settings_fixture", "0.1.0").setting(declarations.values[0].clone()).declarations()?;
        let provider = SettingsPlugin(InstanceGetResponse {
            id: "settings_fixture".into(),
            version: "0.1.0".into(),
            message_bytes: support::settings().placement.message_bytes.get() as u64,
            declarations: Some(catalog),
            revision: 1,
            started_at: Some(Timestamp::default()),
            ..Default::default()
        });
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let url = format!("http://{}", listener.local_addr()?);
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let provider = tokio::spawn(async move {
            Server::builder()
                .add_service(InstanceServer::new(provider))
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                    let _ = stopped.await;
                })
                .await
        });
        let root = support::tempdir()?;
        let credential_root = support::tempdir()?;
        let credentials = support::CredentialStore::start(credential_root.path()).await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = Client::from_channel(channel, support::client_settings())?;
        let placement = PluginPlacement {
            kind: Some(PlacementKind::Remote(PluginRemote { url })),
        };
        client.plugins.create("settings_fixture", placement, PluginValue { enabled: true }).await?;

        // Save a Revision-Checked Product Preference and Replay Its Receipt
        let schema = client.settings.schema().await?.schema.unwrap();
        assert_eq!(Path::new(&schema.bootstrap_paths.unwrap().storage), root.path());
        let before = client.settings.get().await?;
        let mut value = before.value.clone().unwrap();
        value.report.as_mut().unwrap().plot_points = 321;
        let options = SettingsSetOptions {
            request_id: Some("preferences".into()),
            ..Default::default()
        };
        let saved = client.settings.set_with(value.clone(), before.revision.clone(), options.clone()).await?;
        assert_eq!(saved.value, Some(value.clone()));
        assert_ne!(saved.revision, before.revision);
        assert_eq!(client.settings.set_with(value.clone(), before.revision.clone(), options).await?, saved);
        assert_eq!(client.settings.set(value, before.revision).await.unwrap_err().code(), Code::Aborted);
        let mut invalid = saved.value.clone().unwrap();
        invalid.client.as_mut().unwrap().connect_timeout_ms = 0;
        assert!(client.settings.set(invalid, saved.revision.clone()).await.is_err());
        assert_eq!(client.settings.get().await?.revision, saved.revision);

        // Complete Source Values and Plugin Settings Keep Independent Revisions
        let sources = client.storage.sources.get().await?;
        let saved_sources = client.storage.sources.set(sources.value.clone().unwrap(), sources.revision).await?;
        assert_eq!(saved_sources.value, sources.value);
        assert!(client.plugins.settings.list("missing").await.is_err());
        let before = client.plugins.settings.list("settings_fixture").await?;
        assert_eq!(before.value.as_ref().unwrap().values["enabled"].value, Some(SettingKind::Flag(true)));
        let value = SettingValues {
            values: HashMap::from([("enabled".into(), SettingValue { value: Some(SettingKind::Flag(false)) })]),
        };
        let options = PluginsSettingsSetOptions {
            request_id: Some("plugin-value".into()),
            ..Default::default()
        };
        let saved_plugin = client.plugins.settings.set_with("settings_fixture", value.clone(), before.revision, options).await?;
        assert_eq!(saved_plugin.value, Some(value));
        let invalid = SettingValues {
            values: HashMap::from([(
                "enabled".into(),
                SettingValue {
                    value: Some(SettingKind::Text("invalid".into())),
                },
            )]),
        };
        assert!(client.plugins.settings.set("settings_fixture", invalid, saved_plugin.revision.clone()).await.is_err());

        // Disconnected Entries Retain Editable Settings Across Host Restart
        stop.send(()).map_err(|_| "Stop Settings Fixture: Listener Already Closed")?;
        provider.await??;
        process.close().await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = Client::from_channel(channel, support::client_settings())?;
        let reopened = client.settings.get().await?;
        assert_eq!(reopened.value, saved.value);
        assert_eq!(reopened.revision, saved.revision);
        let reopened = client.plugins.settings.list("settings_fixture").await?;
        assert_eq!(reopened.value, saved_plugin.value);
        assert_eq!(reopened.revision, saved_plugin.revision);
        let value = reopened.value.unwrap();
        let updated = client.plugins.settings.set("settings_fixture", value.clone(), reopened.revision).await?;
        assert_eq!(updated.value, Some(value));
        process.close().await?;
        Ok::<(), Box<dyn Error>>(())
    })
    .await?
}
