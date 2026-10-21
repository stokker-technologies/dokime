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
};
// Third Party Imports
use harmos::operations::{Context, Operation, OperationDeclaration, Response, StreamInput};
use harmos_packaging::bundle::Bundle;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use sha2::{Digest, Sha256};
// Local Imports
use dokime::client::contracts::{self, Endpoint, backend, plugins as v1, runtime};
use dokime::domain::model;

/* === Constants === */

const ID: &str = "01900000-0000-7000-8000-000000000001";
const SECRET_VALUE: &str = "fixture-secret-must-not-be-disclosed";
const CHUNK_LIMIT_BYTES: u32 = 256 * 1024;

/* === Macros === */

macro_rules! operation {
    ($name:ident, $declaration:ty, $endpoints:ty, $host:ident, $message:ident, $input:ident, $body:block) => {
        struct $name {
            host: Arc<Host>,
        }
        impl Operation for $name {
            type Declaration = $declaration;
            type Resources = Host;
            type Error = tonic::Status;
            type Context<'a> = Context<'a>;
            type Endpoints = $endpoints;
            async fn initialize<'ctx>(host: Arc<Host>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
                Ok(Self { host })
            }
            async fn run<'ctx>(
                &mut self, _: Self::Context<'ctx>, $message: &mut $declaration, $input: &mut Self::Endpoints,
            ) -> Result<Response<<$declaration as OperationDeclaration>::Response>, Self::Error> {
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
    entry: model::plugins::wire::PluginInstallation,
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

operation!(Stage, v1::Stage, StreamInput<model::plugins::wire::StagePluginFrame>, host, _request, input, {
    host.calls.lock().unwrap().push("stage");
    let header = input.recv().await.unwrap();
    let Some(model::plugins::wire::StagePluginFrameFrame::Header(header)) = &header.frame else { panic!("header first") };
    assert_eq!(header.size_bytes, host.bytes.len() as u64);
    assert_eq!(header.sha256, Sha256::digest(&host.bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>());
    let mut bytes = Vec::new();
    while let Some(frame) = input.recv().await {
        let Some(model::plugins::wire::StagePluginFrameFrame::Chunk(chunk)) = &frame.frame else { panic!("chunk required") };
        assert_eq!(chunk.offset, bytes.len() as u64);
        assert!(chunk.data.len() <= CHUNK_LIMIT_BYTES as usize, "every chunk fits the advertised chunk limit");
        bytes.extend_from_slice(&chunk.data);
    }
    assert_eq!(bytes, host.bytes);
    Ok(host.entry.clone().into())
});

operation!(Consent, v1::Consent, (), host, request, _input, {
    host.calls.lock().unwrap().push("consent");
    assert_eq!(request.install_id, ID);
    assert_eq!(request.expected_sha256, Sha256::digest(&host.bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>());
    Err(tonic::Status::new(host.consent_code, "fixture refusal or lost acknowledgement"))
});

operation!(Remove, v1::Remove, (), host, request, _input, {
    host.calls.lock().unwrap().push("remove");
    assert_eq!(request.install_id, ID);
    Ok(model::plugins::wire::RemovePluginReply { installation: None }.into())
});

operation!(Identity, backend::GetIdentity, (), host, _request, _input, {
    Ok(model::backend::wire::BackendIdentity {
        backend_id: ID.into(),
        epoch: ID.into(),
        api: Some(model::backend::wire::ApiIdentity {
            package: model::backend::wire::API_PACKAGE.into(),
            version: model::backend::wire::API_VERSION.into(),
            descriptor_sha256: host.descriptor_sha256.clone(),
        }),
        product_version: env!("CARGO_PKG_VERSION").into(),
        display_name: "Installation Fixture".into(),
        ..Default::default()
    }
    .into())
});

operation!(Health, backend::GetHealth, (), _host, _request, _input, {
    Ok(model::backend::wire::BackendHealth {
        backend_id: ID.into(),
        epoch: ID.into(),
        readiness: model::backend::wire::Readiness::Ready,
        observed_at: None,
        message: String::new(),
    }
    .into())
});

operation!(Availability, backend::GetAvailability, (), _host, _request, _input, {
    Ok(model::backend::wire::BackendAvailability {
        backend_id: ID.into(),
        epoch: ID.into(),
        methods: [
            backend::GetIdentity::PATH,
            backend::GetHealth::PATH,
            backend::GetAvailability::PATH,
            runtime::GetIdentity::PATH,
            runtime::GetHealth::PATH,
            v1::Stage::PATH,
            v1::Consent::PATH,
            v1::Remove::PATH,
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
        limits: Some(model::backend::wire::BackendLimits {
            max_request_bytes: 16 * 1024 * 1024,
            max_response_bytes: 16 * 1024 * 1024,
            max_content_bytes: Some(272 * 1024 * 1024),
            max_chunk_bytes: Some(CHUNK_LIMIT_BYTES),
            ..Default::default()
        }),
    }
    .into())
});

operation!(RuntimeIdentity, runtime::GetIdentity, (), _host, _request, _input, {
    Ok(model::runtime::wire::Identity {
        backend_id: ID.into(),
        epoch: ID.into(),
        display_name: "Installation Fixture".into(),
        product_version: env!("CARGO_PKG_VERSION").into(),
    }
    .into())
});

operation!(RuntimeHealth, runtime::GetHealth, (), _host, _request, _input, {
    Ok(model::runtime::wire::Health {
        backend_id: ID.into(),
        epoch: ID.into(),
        readiness: model::runtime::wire::Readiness::Ready,
        observed_at_seconds: 0,
        observed_at_nanoseconds: 0,
        message: String::new(),
    }
    .into())
});

/* === Public Functions === */

/* === Private Functions === */

fn review_bundle(schema: u32) -> Bundle {
    use harmos_packaging::bundle::{Executable, Facility, Identity, Limits, Source};
    let owner = "fixture.worker";
    let declaration =
        serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":"1.0.0","description":"Review fixture","config":"()"});
    let policy = harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(5),
        attempt_timeout: std::time::Duration::from_secs(5),
        retries: harmos::operations::RetryPolicy::Never,
        restart: harmos::operations::RestartPolicy::Never,
    };
    let lifecycle = [
        harmos::operations::lifecycle::Descriptor::new(harmos::operations::lifecycle::Action::Initialize, policy).unwrap(),
        harmos::operations::lifecycle::Descriptor::new(harmos::operations::lifecycle::Action::Shutdown, policy).unwrap(),
    ];
    let contracts =
        serde_json::json!({"kind":"contracts","facility":"sidecar","owner":owner,"provides":[],"requires":[],"lifecycle":lifecycle});
    let restart = serde_json::json!({"kind":"restart","facility":"sidecar","owner":owner,"policy":{"credit":0,"sustained_health_ms":30000,"backoff_base_ms":250,"backoff_cap_ms":10000}});
    let publications = serde_json::json!({"kind":"publications","facility":"sidecar","owner":owner,"streams":[]});
    let observers = serde_json::json!({"kind":"observers","facility":"sidecar","owner":owner,"observers":[]});
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
    let terminal = native_pty_system();
    let size = PtySize { rows: 24, cols: 120, pixel_width: 0, pixel_height: 0 };
    let pair = terminal.openpty(size).map_err(|error| error.to_string())?;
    let executable = cli_executable();
    let mut command = CommandBuilder::new(executable);
    command.args(["--color", "never", "plugin", "install"]);
    command.arg(bundle);
    command.args(["--endpoint", endpoint]);
    command.env("PLUGIN_API_TOKEN", SECRET_VALUE);
    if consent {
        command.arg("--yes");
    }
    let process = pair.slave.spawn_command(command).map_err(|error| error.to_string())?;
    let mut child = ChildGuard { child: process, reaped: false };
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().map_err(|error| error.to_string())?;
    let mut writer = pair.master.take_writer().map_err(|error| error.to_string())?;
    let read_task = std::thread::spawn(move || -> Result<Vec<u8>, String> {
        let mut output = Vec::new();
        let mut buffer = [0_u8; 4096];
        let mut answered = 0;
        let mut declined = consent;

        loop {
            let count = match reader.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(count) => count,
            };
            output.extend_from_slice(&buffer[..count]);

            // Answer Cursor Queries Even During Pseudoconsole Teardown
            let asked = output.windows(4).filter(|window| window == b"\x1b[6n").count();
            while answered < asked {
                writer.write_all(b"\x1b[1;1R").map_err(|error| error.to_string())?;
                writer.flush().map_err(|error| error.to_string())?;
                answered += 1;
            }

            // Decline Only After The CLI Asks For Consent
            let prompted = output.windows(5).any(|window| window == b"[y/N]");
            if !declined && prompted {
                writer.write_all(b"n\r").map_err(|error| error.to_string())?;
                writer.flush().map_err(|error| error.to_string())?;
                declined = true;
            }
            if output.len() > 64 * 1024 {
                break;
            }
        }
        Ok(output)
    });

    // The lane deadline bounds a CLI that never exits; the guard reaps it.
    let status = child.child.wait().map_err(|error| error.to_string())?;
    child.reaped = true;
    drop(pair.master);
    let output = read_task.join().map_err(|_| "install PTY reader panicked")??;
    if output.len() > 64 * 1024 {
        return Err("install CLI exceeded output budget".into());
    }
    let output = String::from_utf8_lossy(&output).replace("\r\n", "\n");
    Ok(Outcome { success: status.success(), output })
}

async fn scenario(hide_environment: bool, consent: bool, consent_code: tonic::Code) -> (Outcome, Vec<&'static str>) {
    let bundle = review_bundle(3);
    let digest = Sha256::digest(bundle.bytes()).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), bundle.bytes()).unwrap();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let host = Host {
        entry: model::plugins::wire::PluginInstallation {
            install_id: ID.into(),
            sha256: digest,
            staged_at: None,
            consented_at: None,
            state: model::plugins::wire::PluginInstallationState::Staged,
            last_boot: None,
            metadata: Some(model::plugins::wire::PluginMetadata {
                identity: Some(model::plugins::wire::PluginIdentity {
                    name: bundle.manifest().identity.name.clone(),
                    version: bundle.manifest().identity.version.to_string(),
                    description: bundle.manifest().identity.description.clone(),
                }),
                owners: vec!["fixture.worker".into()],
                environment_references: if hide_environment {
                    vec![]
                } else {
                    vec![model::plugins::wire::PluginEnvironmentReference {
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
            let artifacts = harmos_packaging::proto::Artifacts::new(&catalog).unwrap();
            Sha256::digest(artifacts.descriptors()).to_vec()
        },
    };
    let runtime = harmos::Runtime::<Fixture>::builder((), ()).register(Lanes::catalog()).runtime().await.unwrap();
    let host = Arc::new(host);
    runtime.operations.register_wire_operation::<Stage>("dokime", Lanes::Open, host.clone()).unwrap();
    runtime.operations.register_wire_operation::<Consent>("dokime", Lanes::Open, host.clone()).unwrap();
    runtime.operations.register_wire_operation::<Remove>("dokime", Lanes::Open, host.clone()).unwrap();
    runtime.operations.register_wire_operation::<Identity>("dokime", Lanes::Open, host.clone()).unwrap();
    runtime.operations.register_wire_operation::<Health>("dokime", Lanes::Open, host.clone()).unwrap();
    runtime.operations.register_wire_operation::<Availability>("dokime", Lanes::Open, host.clone()).unwrap();
    runtime.operations.register_wire_operation::<RuntimeIdentity>("dokime", Lanes::Open, host.clone()).unwrap();
    runtime.operations.register_wire_operation::<RuntimeHealth>("dokime", Lanes::Open, host).unwrap();
    let attachment =
        runtime.grpc().attach(harmos::grpc::Settings { address: "127.0.0.1:0".parse().unwrap(), ..Default::default() }).await.unwrap();
    let endpoint = format!("http://{}", attachment.local_addr());
    let output = tokio::task::spawn_blocking(move || run_cli(file.path(), &endpoint, consent)).await;
    attachment.detach().await.unwrap();
    runtime.stop().await.unwrap();
    let calls = calls.lock().unwrap().clone();
    (output.unwrap().unwrap(), calls)
}

#[tokio::test]
async fn hidden_access_refused() {
    let (outcome, calls) = scenario(true, true, tonic::Code::Unavailable).await;
    assert!(!outcome.success);
    assert!(outcome.output.contains("did not match the verified bundle; no consent sent"), "{}", outcome.output);
    assert_eq!(calls, ["stage"]);
    assert!(!outcome.output.contains(SECRET_VALUE));
}

#[tokio::test]
async fn cancel_removes_staged() {
    let (outcome, calls) = scenario(false, false, tonic::Code::Ok).await;
    assert!(outcome.success, "{}", outcome.output);
    assert!(outcome.output.contains("Discarded staged installation"));
    assert!(outcome.output.contains(ID));
    assert!(outcome.output.contains("PLUGIN_API_TOKEN"));
    assert!(!outcome.output.contains(SECRET_VALUE));
    assert_eq!(calls, ["stage", "remove"]);
}

#[tokio::test]
async fn refusal_never_retries() {
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

// The built executable: nextest names it at run time, so an archive extracted on another runner finds it.
fn cli_executable() -> String {
    std::env::var("NEXTEST_BIN_EXE_dokime")
        .ok()
        .or_else(|| option_env!("CARGO_BIN_EXE_dokime").map(str::to_owned))
        .expect("workspace executable")
}
