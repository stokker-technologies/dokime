/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{
    io::{Read, Write},
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
// Third Party Imports
use harmos::plugin::Bundle;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use sha2::{Digest, Sha256};
// Local Imports
use dokime_protocol::v1;

/* === Constants === */
const ID: &str = "01900000-0000-7000-8000-000000000001";
const CHILD_TIMEOUT: Duration = Duration::from_secs(15);
const SECRET_VALUE: &str = "fixture-secret-must-not-be-disclosed";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone)]
struct Host {
    entry: v1::PluginInstallation,
    bytes: Vec<u8>,
    calls: Arc<Mutex<Vec<&'static str>>>,
    consent_code: tonic::Code,
}

struct ChildGuard {
    child: Box<dyn portable_pty::Child + Send + Sync>,
    reaped: bool,
}

struct Outcome {
    success: bool,
    output: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Drop for ChildGuard {
    fn drop(&mut self) {
        if !self.reaped {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

#[tonic::async_trait]
impl v1::plugins_server::Plugins for Host {
    async fn stage(
        &self,
        request: tonic::Request<tonic::Streaming<v1::StagePluginFrame>>,
    ) -> Result<tonic::Response<v1::PluginInstallation>, tonic::Status> {
        self.calls.lock().unwrap().push("stage");
        let mut stream = request.into_inner();
        let header = stream.message().await?.unwrap();
        let Some(v1::stage_plugin_frame::Frame::Header(header)) = header.frame else {
            panic!("header first")
        };
        assert_eq!(header.size_bytes, self.bytes.len() as u64);
        assert_eq!(
            header.sha256,
            Sha256::digest(&self.bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        let mut bytes = Vec::new();
        while let Some(frame) = stream.message().await? {
            let Some(v1::stage_plugin_frame::Frame::Chunk(chunk)) = frame.frame else {
                panic!("chunk required")
            };
            assert_eq!(chunk.offset, bytes.len() as u64);
            assert!(chunk.data.len() <= 256 * 1024);
            bytes.extend(chunk.data);
        }
        assert_eq!(bytes, self.bytes);
        Ok(tonic::Response::new(self.entry.clone()))
    }
    async fn consent(
        &self,
        request: tonic::Request<v1::ConsentPluginRequest>,
    ) -> Result<tonic::Response<v1::PluginInstallation>, tonic::Status> {
        self.calls.lock().unwrap().push("consent");
        assert_eq!(request.get_ref().install_id, ID);
        assert_eq!(
            request.get_ref().expected_sha256,
            Sha256::digest(&self.bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        Err(tonic::Status::new(
            self.consent_code,
            "fixture refusal or lost acknowledgement",
        ))
    }
    async fn remove(
        &self,
        request: tonic::Request<v1::RemovePluginRequest>,
    ) -> Result<tonic::Response<v1::RemovePluginReply>, tonic::Status> {
        self.calls.lock().unwrap().push("remove");
        assert_eq!(request.get_ref().install_id, ID);
        Ok(tonic::Response::new(v1::RemovePluginReply {
            installation: None,
        }))
    }
    async fn list_installations(
        &self,
        _: tonic::Request<v1::ListInstallationsRequest>,
    ) -> Result<tonic::Response<v1::InstallationPage>, tonic::Status> {
        Err(tonic::Status::unimplemented("fixture"))
    }
    async fn get_installation(
        &self,
        _: tonic::Request<v1::GetInstallationRequest>,
    ) -> Result<tonic::Response<v1::PluginInstallation>, tonic::Status> {
        Err(tonic::Status::unimplemented("fixture"))
    }
    async fn set_enabled(
        &self,
        _: tonic::Request<v1::SetPluginEnabledRequest>,
    ) -> Result<tonic::Response<v1::PluginInstallation>, tonic::Status> {
        Err(tonic::Status::unimplemented("fixture"))
    }
    async fn list_admitted_plugins(
        &self,
        _: tonic::Request<v1::ListAdmittedPluginsRequest>,
    ) -> Result<tonic::Response<v1::AdmittedPluginPage>, tonic::Status> {
        Err(tonic::Status::unimplemented("fixture"))
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn review_bundle(schema: u32) -> Bundle {
    use harmos::plugin::{Executable, Facility, Identity, Limits, Source};
    let owner = "fixture.worker";
    let declaration = serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":"1.0.0","description":"Review fixture","config":"()"});
    let policy = harmos::OperationPolicy {
        timeout: std::time::Duration::from_secs(5),
        attempt_timeout: std::time::Duration::from_secs(5),
        retries: harmos::RetryPolicy::Never,
    };
    let lifecycle = [
        harmos::operations::lifecycle::Descriptor::new(
            harmos::operations::lifecycle::Action::Initialize,
            policy,
        )
        .unwrap(),
        harmos::operations::lifecycle::Descriptor::new(
            harmos::operations::lifecycle::Action::Shutdown,
            policy,
        )
        .unwrap(),
    ];
    let contracts = serde_json::json!({"kind":"contracts","facility":"sidecar","owner":owner,"provides":[],"requires":[],"lifecycle":lifecycle});
    let restart = serde_json::json!({"kind":"restart","facility":"sidecar","owner":owner,"policy":{"credit":0,"sustained_health_ms":30000,"backoff_base_ms":250,"backoff_cap_ms":10000}});
    let publications =
        serde_json::json!({"kind":"publications","facility":"sidecar","owner":owner,"streams":[]});
    let observers =
        serde_json::json!({"kind":"observers","facility":"sidecar","owner":owner,"observers":[]});
    let executable = format!(
        "inert{prefix}{declaration}\0{prefix}{contracts}\0{prefix}{restart}\0{prefix}{publications}\0{prefix}{observers}\0",
        prefix = std::str::from_utf8(harmos::artifact::PREFIX).unwrap()
    );
    let mut registrations = harmos::sidecar::Registrar::new();
    registrations
        .register(
            "dokime.settings",
            schema,
            &serde_json::json!({
                "catalog": { "contributions": [
                    { "kind": "secret_reference", "id": "token", "default": { "kind": "environment", "variable": "PLUGIN_API_TOKEN" } },
                    { "kind": "secret_reference", "id": "optional", "default": null },
                    { "kind": "text_area", "id": "notes", "default": "not-for-consent-output" }
                ] }
            }),
        )
        .unwrap();
    Bundle::build(
        Identity {
            name: "review-fixture".into(),
            version: "1.0.0".parse().unwrap(),
            description: "Review fixture".into(),
            package: Default::default(),
        },
        &[Source {
            path: "bin/worker",
            bytes: executable.as_bytes(),
            executable: Some(Executable {
                facility: Facility::Sidecar,
                target: "fixture-target".into(),
                registrations: registrations.finish(),
            }),
        }],
        Limits::default(),
    )
    .unwrap()
}

fn run_cli(bundle: &Path, endpoint: &str, consent: bool) -> Result<Outcome, String> {
    let pair = native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|error| error.to_string())?;
    let mut command = CommandBuilder::new(super::support::tool());
    command.args(["--color", "never", "install"]);
    command.arg(bundle);
    command.args(["--endpoint", endpoint]);
    command.env("PLUGIN_API_TOKEN", SECRET_VALUE);
    if consent {
        command.arg("--yes");
    }
    let mut child = ChildGuard {
        child: pair
            .slave
            .spawn_command(command)
            .map_err(|error| error.to_string())?,
        reaped: false,
    };
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|error| error.to_string())?;
    let mut writer = pair
        .master
        .take_writer()
        .map_err(|error| error.to_string())?;
    let (sent, received) = std::sync::mpsc::sync_channel(1);
    let read_task = std::thread::spawn(move || {
        let mut output = Vec::new();
        let mut buffer = [0_u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(count) => {
                    output.extend_from_slice(&buffer[..count]);
                    if output.len() > 64 * 1024 {
                        break;
                    }
                }
            }
        }
        let _ = sent.send(output);
    });
    if !consent {
        writer
            .write_all(b"n\r")
            .map_err(|error| error.to_string())?;
        writer.flush().map_err(|error| error.to_string())?;
    }
    let deadline = Instant::now() + CHILD_TIMEOUT;
    let status = loop {
        if let Some(status) = child.child.try_wait().map_err(|error| error.to_string())? {
            child.reaped = true;
            break status;
        }
        if Instant::now() >= deadline {
            child.child.kill().map_err(|error| error.to_string())?;
            child.child.wait().map_err(|error| error.to_string())?;
            child.reaped = true;
            drop(writer);
            drop(pair.master);
            let _ = received.recv_timeout(CHILD_TIMEOUT);
            return Err("install CLI exceeded its exit deadline".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    drop(writer);
    drop(pair.master);
    let output = received
        .recv_timeout(CHILD_TIMEOUT)
        .map_err(|_| "install PTY output did not close")?;
    read_task
        .join()
        .map_err(|_| "install PTY reader panicked")?;
    if output.len() > 64 * 1024 {
        return Err("install CLI exceeded output budget".into());
    }
    Ok(Outcome {
        success: status.success(),
        output: String::from_utf8_lossy(&output).replace("\r\n", "\n"),
    })
}

async fn scenario(
    hide_environment: bool,
    consent: bool,
    consent_code: tonic::Code,
) -> (Outcome, Vec<&'static str>) {
    let bundle = review_bundle(3);
    let digest = Sha256::digest(bundle.bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), bundle.bytes()).unwrap();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let host = Host {
        entry: v1::PluginInstallation {
            install_id: ID.into(),
            sha256: digest,
            state: v1::PluginInstallationState::Staged as i32,
            metadata: Some(v1::PluginMetadata {
                identity: Some(v1::PluginIdentity {
                    name: bundle.manifest().identity.name.clone(),
                    version: bundle.manifest().identity.version.to_string(),
                    description: bundle.manifest().identity.description.clone(),
                }),
                owners: vec!["fixture.worker".into()],
                environment_references: if hide_environment {
                    vec![]
                } else {
                    vec![v1::PluginEnvironmentReference {
                        owner: "fixture.worker".into(),
                        setting: "token".into(),
                        variable: "PLUGIN_API_TOKEN".into(),
                    }]
                },
                size_bytes: bundle.bytes().len() as u64,
            }),
            ..Default::default()
        },
        bytes: bundle.bytes().to_vec(),
        calls: calls.clone(),
        consent_code,
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let mut server = tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(v1::plugins_server::PluginsServer::new(host))
            .serve_with_incoming_shutdown(
                tokio_stream::wrappers::TcpListenerStream::new(listener),
                async {
                    let _ = stopped.await;
                },
            )
            .await
            .unwrap();
    });
    let output =
        tokio::task::spawn_blocking(move || run_cli(file.path(), &endpoint, consent)).await;
    let _ = stop.send(());
    if let Ok(result) = tokio::time::timeout(CHILD_TIMEOUT, &mut server).await {
        result.unwrap();
    } else {
        server.abort();
        let _ = server.await;
        panic!("install fixture host exceeded shutdown deadline");
    }
    let calls = calls.lock().unwrap().clone();
    (output.unwrap().unwrap(), calls)
}

async fn hidden_environment_access_prevents_automatic_consent() {
    let (outcome, calls) = scenario(true, true, tonic::Code::Unavailable).await;
    assert!(!outcome.success);
    assert!(
        outcome
            .output
            .contains("did not match the verified bundle; no consent sent"),
        "{}",
        outcome.output
    );
    assert_eq!(calls, ["stage"]);
    assert!(!outcome.output.contains(SECRET_VALUE));
}

async fn interactive_cancellation_removes_only_the_staged_bundle() {
    let (outcome, calls) = scenario(false, false, tonic::Code::Ok).await;
    assert!(outcome.success, "{}", outcome.output);
    assert!(outcome.output.contains("Discarded staged installation"));
    assert!(outcome.output.contains(ID));
    assert!(outcome.output.contains("PLUGIN_API_TOKEN"));
    assert!(!outcome.output.contains(SECRET_VALUE));
    assert_eq!(calls, ["stage", "remove"]);
}

async fn refused_or_uncertain_consent_never_removes_or_retries() {
    for code in [tonic::Code::FailedPrecondition, tonic::Code::Unavailable] {
        let (outcome, calls) = scenario(false, true, code).await;
        assert!(!outcome.success);
        assert!(outcome.output.contains(ID));
        assert!(outcome.output.contains("No removal or retry"));
        assert!(!outcome.output.contains(SECRET_VALUE));
        assert_eq!(calls, ["stage", "consent"]);
    }
}

/* === Tests === */

pub(super) async fn consent_contract() {
    hidden_environment_access_prevents_automatic_consent().await;
    interactive_cancellation_removes_only_the_staged_bundle().await;
    refused_or_uncertain_consent_never_removes_or_retries().await;
}
