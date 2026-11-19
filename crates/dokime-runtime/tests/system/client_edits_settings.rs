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

// Standard Library Imports
use std::{collections::HashMap, error::Error, path::Path};

// Third Party Imports
use harmos_protocol::host::plugin_placement;
use harmos_protocol::host::{PluginPlacement, PluginRemote, PluginValue};
use harmos_protocol::model::setting_value;
use harmos_protocol::model::{SettingDeclaration, SettingValue, SettingValues, SettingsDeclarations};
use harmos_protocol::plugin::instance_server::{Instance, InstanceServer};
use harmos_protocol::plugin::{
    InstanceGetRequest, InstanceGetResponse, InstanceInvokeRequest, InstanceInvokeResponse, InstanceSamplesRequest,
    InstanceSamplesResponse, InstanceStopRequest, InstanceStopResponse, instance_invoke_request, instance_invoke_response, response,
};
use prost_types::Timestamp;
use tokio::net::TcpListener;
use tokio_stream::{Empty, wrappers::TcpListenerStream};
use tonic::{
    Code, Request, Response, Status, Streaming,
    transport::{Endpoint, Server},
};

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use super::support;
use dokime::plugin::Plugin;
use dokime_client_internal::Client;
use dokime_client_internal::PluginsSettingsSetOptions;
use dokime_client_internal::SettingsSetOptions;
use dokime_client_internal::{PluginState, PluginsGetOptions, PluginsWatchOptions};

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
    type InvokeStream = tokio_stream::Once<Result<InstanceInvokeResponse, Status>>;
    async fn invoke(&self, request: Request<Streaming<InstanceInvokeRequest>>) -> Result<Response<Self::InvokeStream>, Status> {
        let Some(instance_invoke_request::Frame::Request(request)) = request.into_inner().message().await?.and_then(|frame| frame.frame)
        else {
            return Err(Status::invalid_argument("Invoke Settings Fixture: Supply an Invocation"));
        };
        if !matches!(request.operation.as_str(), "settings_fixture.initialize" | "settings_fixture.finalize") {
            return Err(Status::unimplemented("Invoke Settings Fixture: Call Initialize or Finalize"));
        }
        Ok(Response::new(tokio_stream::once(Ok(InstanceInvokeResponse {
            frame: Some(instance_invoke_response::Frame::Response(harmos_protocol::plugin::Response {
                result: Some(response::Result::Payload(Vec::new())),
            })),
        }))))
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

#[test]
fn client_edits_settings() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Declare Typed Library Settings on a Real Plugin Carrier
            let declarations = SettingsDeclarations {
                values: vec![SettingDeclaration {
                    name: "enabled".into(),
                    title: "Enabled".into(),
                    description: String::new(),
                    default: Some(SettingValue {
                        value: Some(setting_value::Value::Flag(true)),
                    }),
                }],
            };
            let catalog =
                Plugin::of::<()>("0.1.0").identifier("settings_fixture").setting(declarations.values[0].clone()).declarations()?;
            let provider = SettingsPlugin(InstanceGetResponse {
                id: "settings_fixture".into(),
                version: "0.1.0".into(),
                sdk: "rust".into(),
                sdk_version: env!("CARGO_PKG_VERSION").into(),
                message_bytes: support::settings().placement.message_bytes.get() as u64,
                declarations: Some(catalog),
                revision: 1,
                started_at: Some(Timestamp::default()),
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
                kind: Some(plugin_placement::Kind::Remote(PluginRemote { url })),
            };
            let created = client.plugins.create("settings_fixture", placement, PluginValue { enabled: true }).await?;
            let mut entry = created.entry.ok_or("Create Settings Fixture: Missing Entry")?;
            let mut watching = client
                .plugins
                .watch_with(PluginsWatchOptions {
                    name: Some(entry.name.clone()),
                    ..Default::default()
                })
                .await?;
            while entry.state != PluginState::Ready as i32 {
                if entry.failure.is_some() || entry.state == PluginState::Stopped as i32 {
                    return Err(format!("Activate Settings Fixture: {:?}", entry.failure).into());
                }
                watching.next().await?.ok_or("Activate Settings Fixture: Watch Closed")?;
                entry = client
                    .plugins
                    .get_with(entry.name, PluginsGetOptions::default())
                    .await?
                    .entry
                    .ok_or("Activate Settings Fixture: Entry Removed")?;
            }
            drop(watching);

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
            let saved = client
                .settings
                .set_with(
                    value.clone(),
                    before.revision.clone(),
                    SettingsSetOptions {
                        request_id: options.request_id.clone(),
                        ..Default::default()
                    },
                )
                .await?;
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
            assert_eq!(before.value.as_ref().unwrap().values["enabled"].value, Some(setting_value::Value::Flag(true)));
            let value = SettingValues {
                values: HashMap::from([(
                    "enabled".into(),
                    SettingValue {
                        value: Some(setting_value::Value::Flag(false)),
                    },
                )]),
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
                        value: Some(setting_value::Value::Text("invalid".into())),
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
    }))
}
