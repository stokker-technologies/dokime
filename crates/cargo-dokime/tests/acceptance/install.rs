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
use harmos::bundle::Bundle;
use harmos::operations::{Context, Operation, OperationDeclaration, Response, StreamInput};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use sha2::{Digest, Sha256};
// Local Imports
use dokime_client::contracts::{self, Endpoint, backend, plugins as v1, runtime};

/* === Constants === */
const ID: &str = "01900000-0000-7000-8000-000000000001";
const CHILD_TIMEOUT: Duration = Duration::from_secs(15);
const SECRET_VALUE: &str = "fixture-secret-must-not-be-disclosed";

/* === Macros === */
macro_rules! operation {
    ($name:ident, $declaration:ty, $request:ty, $endpoints:ty, $host:ident, $message:ident, $input:ident, $body:block) => {
        struct $name {
            host: Arc<Host>,
        }
        impl Operation for $name {
            type Declaration = $declaration;
            type Resources = Host;
            type Error = tonic::Status;
            type Context<'a> = Context<'a>;
            type Endpoints = $endpoints;
            async fn initialize<'ctx>(
                host: Arc<Host>,
                _: Self::Context<'ctx>,
            ) -> Result<Self, Self::Error> {
                Ok(Self { host })
            }
            async fn run<'ctx>(
                &mut self,
                _: Self::Context<'ctx>,
                $message: &mut $request,
                $input: &mut Self::Endpoints,
            ) -> Result<Response<<$declaration as OperationDeclaration>::Response>, Self::Error>
            {
                let $host = &self.host;
                $body
            }
        }
    };
}

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone)]
struct Host {
    entry: v1::PluginInstallation,
    bytes: Vec<u8>,
    calls: Arc<Mutex<Vec<&'static str>>>,
    consent_code: tonic::Code,
    descriptor_sha256: Vec<u8>,
}

struct Fixture;

struct ChildGuard {
    child: Box<dyn portable_pty::Child + Send + Sync>,
    reaped: bool,
}

struct Outcome {
    success: bool,
    output: String,
}

/* === Enums === */
#[harmos::lanes]
enum Lanes {
    #[default]
    Open,
}

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

impl harmos::Application for Fixture {
    type State = ();
    type Transaction = harmos::state::Never;
    type Record = ();
    type Streams = ();
    type Resources = ();
}

operation!(
    Stage,
    v1::Stage,
    (),
    StreamInput<v1::StagePluginFrame>,
    host,
    _request,
    input,
    {
        host.calls.lock().unwrap().push("stage");
        let header = input.recv().await.unwrap();
        let Some(v1::StagePluginFrameFrame::Header(header)) = &header.frame else {
            panic!("header first")
        };
        assert_eq!(header.size_bytes, host.bytes.len() as u64);
        assert_eq!(
            header.sha256,
            Sha256::digest(&host.bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        let mut bytes = Vec::new();
        while let Some(frame) = input.recv().await {
            let Some(v1::StagePluginFrameFrame::Chunk(chunk)) = &frame.frame else {
                panic!("chunk required")
            };
            assert_eq!(chunk.offset, bytes.len() as u64);
            assert!(chunk.data.len() <= 256 * 1024);
            bytes.extend_from_slice(&chunk.data);
        }
        assert_eq!(bytes, host.bytes);
        Ok(host.entry.clone().into())
    }
);

operation!(
    Consent,
    v1::Consent,
    v1::ConsentPluginRequest,
    (),
    host,
    request,
    _input,
    {
        host.calls.lock().unwrap().push("consent");
        assert_eq!(request.install_id, ID);
        assert_eq!(
            request.expected_sha256,
            Sha256::digest(&host.bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        Err(tonic::Status::new(
            host.consent_code,
            "fixture refusal or lost acknowledgement",
        ))
    }
);

operation!(
    Remove,
    v1::Remove,
    v1::RemovePluginRequest,
    (),
    host,
    request,
    _input,
    {
        host.calls.lock().unwrap().push("remove");
        assert_eq!(request.install_id, ID);
        Ok(v1::RemovePluginReply { installation: None }.into())
    }
);

operation!(
    Identity,
    backend::GetIdentity,
    backend::GetIdentityRequest,
    (),
    host,
    _request,
    _input,
    {
        Ok(backend::BackendIdentity {
            backend_id: ID.into(),
            epoch: ID.into(),
            api: Some(backend::ApiIdentity {
                package: backend::API_PACKAGE.into(),
                version: backend::API_VERSION.into(),
                descriptor_sha256: host.descriptor_sha256.clone(),
            }),
            product_version: env!("CARGO_PKG_VERSION").into(),
            display_name: "Installation Fixture".into(),
            ..Default::default()
        }
        .into())
    }
);

operation!(
    Health,
    backend::GetHealth,
    backend::GetHealthRequest,
    (),
    _host,
    _request,
    _input,
    {
        Ok(backend::BackendHealth {
            backend_id: ID.into(),
            epoch: ID.into(),
            readiness: backend::Readiness::Ready,
            observed_at: None,
            message: String::new(),
        }
        .into())
    }
);

operation!(
    Capabilities,
    backend::GetCapabilities,
    backend::GetCapabilitiesRequest,
    (),
    _host,
    _request,
    _input,
    {
        Ok(backend::BackendCapabilities {
            backend_id: ID.into(),
            epoch: ID.into(),
            methods: [
                backend::GetIdentity::PATH,
                backend::GetHealth::PATH,
                backend::GetCapabilities::PATH,
                runtime::GetIdentity::PATH,
                runtime::GetHealth::PATH,
                v1::Stage::PATH,
                v1::Consent::PATH,
                v1::Remove::PATH,
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            limits: Some(backend::BackendLimits {
                max_request_bytes: 16 * 1024 * 1024,
                max_response_bytes: 16 * 1024 * 1024,
                max_content_bytes: Some(272 * 1024 * 1024),
                max_chunk_bytes: Some(256 * 1024),
                ..Default::default()
            }),
        }
        .into())
    }
);

operation!(
    RuntimeIdentity,
    runtime::GetIdentity,
    runtime::Request,
    (),
    _host,
    _request,
    _input,
    {
        Ok(runtime::Identity {
            backend_id: ID.into(),
            epoch: ID.into(),
            display_name: "Installation Fixture".into(),
            product_version: env!("CARGO_PKG_VERSION").into(),
        }
        .into())
    }
);

operation!(
    RuntimeHealth,
    runtime::GetHealth,
    runtime::Request,
    (),
    _host,
    _request,
    _input,
    {
        Ok(runtime::Health {
            backend_id: ID.into(),
            epoch: ID.into(),
            readiness: runtime::Readiness::Ready,
            observed_at_seconds: 0,
            observed_at_nanoseconds: 0,
            message: String::new(),
        }
        .into())
    }
);

/* === Public Functions === */

/* === Private Functions === */
fn review_bundle(schema: u32) -> Bundle {
    use harmos::bundle::{Executable, Facility, Identity, Limits, Source};
    let owner = "fixture.worker";
    let declaration = serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":"1.0.0","description":"Review fixture","config":"()"});
    let policy = harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(5),
        attempt_timeout: std::time::Duration::from_secs(5),
        retries: harmos::operations::RetryPolicy::Never,
        restart: harmos::operations::RestartPolicy::Never,
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
        prefix = std::str::from_utf8(harmos::artifacts::PREFIX).unwrap()
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
            staged_at: None,
            consented_at: None,
            state: v1::PluginInstallationState::Staged,
            last_boot: None,
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
            error: None,
            active: false,
            restart_required: false,
            boot_error: None,
        },
        bytes: bundle.bytes().to_vec(),
        calls: calls.clone(),
        consent_code,
        descriptor_sha256: {
            let catalog = harmos::operations::Contracts::of::<contracts::Operations>().unwrap();
            let artifacts = harmos::grpc::Artifacts::new(&catalog).unwrap();
            Sha256::digest(artifacts.descriptors()).to_vec()
        },
    };
    let runtime = harmos::Runtime::<Fixture>::builder((), ())
        .register(Lanes::catalog())
        .runtime()
        .await
        .unwrap();
    let host = Arc::new(host);
    runtime
        .operations
        .register_wire_operation::<Stage>("dokime", Lanes::Open, host.clone())
        .unwrap();
    runtime
        .operations
        .register_wire_operation::<Consent>("dokime", Lanes::Open, host.clone())
        .unwrap();
    runtime
        .operations
        .register_wire_operation::<Remove>("dokime", Lanes::Open, host.clone())
        .unwrap();
    runtime
        .operations
        .register_wire_operation::<Identity>("dokime", Lanes::Open, host.clone())
        .unwrap();
    runtime
        .operations
        .register_wire_operation::<Health>("dokime", Lanes::Open, host.clone())
        .unwrap();
    runtime
        .operations
        .register_wire_operation::<Capabilities>("dokime", Lanes::Open, host.clone())
        .unwrap();
    runtime
        .operations
        .register_wire_operation::<RuntimeIdentity>("dokime", Lanes::Open, host.clone())
        .unwrap();
    runtime
        .operations
        .register_wire_operation::<RuntimeHealth>("dokime", Lanes::Open, host)
        .unwrap();
    let attachment = runtime
        .grpc()
        .attach(harmos::grpc::Settings {
            address: "127.0.0.1:0".parse().unwrap(),
            ..Default::default()
        })
        .await
        .unwrap();
    let endpoint = format!("http://{}", attachment.local_addr());
    let output =
        tokio::task::spawn_blocking(move || run_cli(file.path(), &endpoint, consent)).await;
    tokio::time::timeout(CHILD_TIMEOUT, attachment.detach())
        .await
        .expect("install fixture host exceeded shutdown deadline")
        .unwrap();
    runtime.stop_with_timeout(CHILD_TIMEOUT).await.unwrap();
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
