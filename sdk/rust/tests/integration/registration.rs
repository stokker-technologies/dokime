/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// A Plugin Registers Itself With a Paired Host and Keeps Its Entry When It Stops

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::HashMap,
    fs,
    pin::Pin,
    process::Stdio,
    sync::{Arc, Mutex},
};

// Third Party Imports
use harmos_protocol::model::{Classification, Failure};
use harmos_protocol::plugin::{InstanceStopRequest, instance_client::InstanceClient};
use prost::Message;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpListener,
    process::Command,
    sync::Notify,
};
use tokio_stream::{Stream, wrappers::TcpListenerStream};
use tonic::{Code, Request, Response, Status, transport::Server};

// Local Imports
use super::support::{TestResult, plugin::binary};
use dokime::Plugin;
use dokime::client::{ConnectionSettings, PluginRegisterOptions, connect_with};
use dokime_protocol::host::plugins_server::{Plugins, PluginsServer};
use dokime_protocol::host::{
    PluginEntry, PluginState, PluginsCreateRequest, PluginsCreateResponse, PluginsDeleteRequest, PluginsDeleteResponse, PluginsGetRequest,
    PluginsGetResponse, PluginsListRequest, PluginsListResponse, PluginsReplaceRequest, PluginsReplaceResponse, PluginsRestartRequest,
    PluginsRestartResponse, PluginsSetRequest, PluginsSetResponse, PluginsWatchRequest, PluginsWatchResponse,
};

/* === Constants === */

const REFUSAL: &str = "Fixture Host Refuses SDK";
const REFUSAL_ACTION: &str = "Rebuild the Fixture Plugin";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Default)]
struct Host {
    state: Arc<Mutex<Registry>>,
    created: Arc<Notify>,
}

#[derive(Default)]
struct Registry {
    requests: Vec<PluginsCreateRequest>,
    entries: HashMap<String, PluginEntry>,
    deletes: usize,
    refuse: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl Plugins for Host {
    type WatchStream = Pin<Box<dyn Stream<Item = Result<PluginsWatchResponse, Status>> + Send>>;

    async fn create(&self, request: Request<PluginsCreateRequest>) -> Result<Response<PluginsCreateResponse>, Status> {
        let request = request.into_inner();
        let mut state = self.state.lock().expect("Fixture Host Poisoned");
        state.requests.push(request.clone());
        self.created.notify_one();
        if state.refuse {
            let failure = Failure {
                classification: Classification::Permanent as i32,
                code: "plugin_incompatible".into(),
                message: REFUSAL.into(),
                action: REFUSAL_ACTION.into(),
                ..Default::default()
            };
            return Err(Status::with_details(Code::FailedPrecondition, REFUSAL, failure.encode_to_vec().into()));
        }
        // Like the Product Host, an Existing Name Reuses Its Entry.
        let entry = state.entries.entry(request.name.clone()).or_insert_with(|| PluginEntry {
            name: request.name.clone(),
            revision: b"created".to_vec(),
            ..Default::default()
        });
        entry.placement = request.placement;
        entry.value = request.value;
        entry.state = PluginState::Ready as i32;
        Ok(Response::new(PluginsCreateResponse {
            entry: Some(entry.clone()),
            receipt: None,
        }))
    }
    async fn delete(&self, _: Request<PluginsDeleteRequest>) -> Result<Response<PluginsDeleteResponse>, Status> {
        self.state.lock().expect("Fixture Host Poisoned").deletes += 1;
        Err(Status::unimplemented("Fixture Host Keeps Entries"))
    }
    async fn get(&self, _: Request<PluginsGetRequest>) -> Result<Response<PluginsGetResponse>, Status> {
        Err(Status::unimplemented("Fixture Host Answers Create Only"))
    }
    async fn list(&self, _: Request<PluginsListRequest>) -> Result<Response<PluginsListResponse>, Status> {
        Err(Status::unimplemented("Fixture Host Answers Create Only"))
    }
    async fn watch(&self, _: Request<PluginsWatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        Err(Status::unimplemented("Fixture Host Answers Create Only"))
    }
    async fn set(&self, _: Request<PluginsSetRequest>) -> Result<Response<PluginsSetResponse>, Status> {
        Err(Status::unimplemented("Fixture Host Answers Create Only"))
    }
    async fn restart(&self, _: Request<PluginsRestartRequest>) -> Result<Response<PluginsRestartResponse>, Status> {
        Err(Status::unimplemented("Fixture Host Answers Create Only"))
    }
    async fn replace(&self, _: Request<PluginsReplaceRequest>) -> Result<Response<PluginsReplaceResponse>, Status> {
        Err(Status::unimplemented("Fixture Host Answers Create Only"))
    }
}

impl Host {
    async fn start() -> TestResult<(Self, String)> {
        let host = Self::default();
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let endpoint = format!("http://{}", listener.local_addr()?);
        let service = PluginsServer::new(host.clone());
        tokio::spawn(Server::builder().add_service(service).serve_with_incoming(TcpListenerStream::new(listener)));
        Ok((host, endpoint))
    }
    fn requests(&self) -> Vec<PluginsCreateRequest> {
        self.state.lock().expect("Fixture Host Poisoned").requests.clone()
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn sdk_version() -> TestResult<String> {
    // The SDK Reports Its Own Package Version, Which the Workspace Manifest Owns.
    let manifest = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../Cargo.toml"))?;
    let package = manifest.split("[workspace.package]").nth(1).ok_or("Missing Workspace Package")?;
    let line = package.lines().find(|line| line.starts_with("version = ")).ok_or("Missing Workspace Version")?;
    Ok(line.trim_start_matches("version = ").trim_matches('"').into())
}

fn plugin() -> Plugin {
    Plugin::of::<()>("1.0.0").identifier("registered")
}

/* === Tests === */

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_managed_plugin_delegates_shutdown_and_ignores_ambient_pairing() -> TestResult {
    let (host, endpoint) = Host::start().await?;
    let mut command = Command::from(crate::process::command(binary("dokime-rust-plugin")));
    command.env("HARMOS_ADDRESS", "127.0.0.1:0").env("DOKIME_REMOTE", endpoint);
    let mut child = command.stdout(Stdio::piped()).stderr(Stdio::inherit()).kill_on_drop(true).spawn()?;
    let mut lines = BufReader::new(child.stdout.take().ok_or("Missing Plugin Output")?).lines();
    let line = lines.next_line().await?.ok_or("Missing Managed Announcement")?;
    let address = line.strip_prefix("harmos listening ").ok_or("Invalid Managed Announcement")?;
    let mut client = InstanceClient::connect(format!("http://{address}")).await?;
    client.stop(InstanceStopRequest { reason: "fixture-complete".into() }).await?;
    assert!(child.wait().await?.success());
    assert!(lines.next_line().await?.is_none());
    assert!(host.requests().is_empty());
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registration_sends_the_sdk_version_and_close_keeps_the_entry() -> TestResult {
    let (host, endpoint) = Host::start().await?;
    for _ in 0..2 {
        let client = connect_with(ConnectionSettings {
            target: Some(endpoint.clone()),
            ..Default::default()
        })
        .await?;
        let options = PluginRegisterOptions {
            address: Some("127.0.0.1:0".into()),
            ..Default::default()
        };
        let registered = client.plugins().register_with(plugin(), options).await?;
        assert_eq!(registered.identifier(), "registered");
        client.close().await?;
    }
    let requests = host.requests();
    let names = requests.iter().map(|request| request.name.as_str()).collect::<Vec<_>>();
    assert_eq!(names, ["registered", "registered"]);
    let version = sdk_version()?;
    assert!(requests.iter().all(|request| request.sdk_version.as_deref() == Some(version.as_str())));
    let state = host.state.lock().expect("Fixture Host Poisoned");
    assert_eq!((state.entries.len(), state.deletes), (1, 0));
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_refused_registration_reports_the_host_message() -> TestResult {
    let (host, endpoint) = Host::start().await?;
    host.state.lock().expect("Fixture Host Poisoned").refuse = true;
    let client = connect_with(ConnectionSettings {
        target: Some(endpoint),
        ..Default::default()
    })
    .await?;
    let options = PluginRegisterOptions {
        address: Some("127.0.0.1:0".into()),
        ..Default::default()
    };
    let Err(error) = client.plugins().register_with(plugin(), options).await else {
        return Err("The Fixture Host Refusal Must Fail Registration".into());
    };
    assert!(error.to_string().contains(REFUSAL), "{error}");
    assert!(format!("{error:?}").contains(REFUSAL_ACTION), "{error:?}");
    client.close().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_paired_plugin_process_registers_itself_and_exits_nonzero_on_refusal() -> TestResult {
    let (host, endpoint) = Host::start().await?;
    let command = |remote: &str| {
        let mut command = Command::from(crate::process::command(binary("dokime-rust-plugin")));
        command.env_remove("HARMOS_ADDRESS").env("DOKIME_REMOTE", remote).env("DOKIME_PLUGIN_ADDRESS", "127.0.0.1:0");
        command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
        command
    };

    // Self-Registration Happens Once per Start Under the Plugin's Stable Name.
    let created = host.created.notified();
    let mut child = command(&endpoint).spawn()?;
    created.await;
    child.kill().await?;
    let requests = host.requests();
    assert_eq!(requests.len(), 1);
    assert!(!requests[0].name.is_empty());
    assert_eq!(requests[0].sdk_version.as_deref(), Some(sdk_version()?.as_str()));

    host.state.lock().expect("Fixture Host Poisoned").refuse = true;
    let output = command(&endpoint).output().await?;
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains(REFUSAL));
    assert!(String::from_utf8_lossy(&output.stderr).contains(REFUSAL_ACTION));
    assert_eq!(host.requests()[1].name, requests[0].name);
    Ok(())
}
