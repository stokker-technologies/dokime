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
    path::Path,
    sync::{Arc, Mutex},
};

// Third Party Imports
use dokime::harmos::operations::{OperationDeclaration, Response};
use dokime::harmos::{Context, RegistrationExt};
use dokime::operations::hosting::{Serve, Served};
use harmos_packaging::bundle::Bundle;
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
    ($declaration:ty, $host:ident, $message:ident, $input:ident, $body:block) => {
        impl Serve<$declaration> for Host {
            fn serve<'a>(
                &'a self, $message: $declaration, ctx: Context<'a, $declaration>,
            ) -> Served<'a, <$declaration as OperationDeclaration>::Response> {
                Box::pin(async move {
                    let $host = self;
                    let $input = ctx.streams();
                    let served: Result<Response<<$declaration as OperationDeclaration>::Response>, tonic::Status> =
                        async move { $body }.await;
                    served
                })
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
    install_code: tonic::Code,
    descriptor_sha256: Vec<u8>,
}

struct Fixture;

struct Outcome {
    success: bool,
    output: String,
}

/* === Enums === */

#[dokime::harmos::lanes(harmos_path = dokime::harmos)]
enum Lanes {
    #[default]
    Open,
}

/* === Traits === */

/* === Implementations === */

impl dokime::harmos::Application for Fixture {
    type Lanes = Lanes;
    type State = ();
    type Transaction = dokime::harmos::state::Never;
    type Record = ();
    type Streams = ();
    type Resources = ();
}

operation!(v1::Install, host, _request, input, {
    host.calls.lock().unwrap().push("install");
    let header = input.recv().await.unwrap();
    let Some(model::plugins::wire::InstallPluginFrameFrame::Header(header)) = &header.frame else { panic!("header first") };
    assert_eq!(header.size_bytes, host.bytes.len() as u64);
    assert_eq!(header.sha256, Sha256::digest(&host.bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>());
    let mut bytes = Vec::new();
    while let Some(frame) = input.recv().await {
        let Some(model::plugins::wire::InstallPluginFrameFrame::Chunk(chunk)) = &frame.frame else { panic!("chunk required") };
        assert_eq!(chunk.offset, bytes.len() as u64);
        assert!(chunk.data.len() <= CHUNK_LIMIT_BYTES as usize, "every chunk fits the advertised chunk limit");
        bytes.extend_from_slice(&chunk.data);
    }
    assert_eq!(bytes, host.bytes);
    if host.install_code != tonic::Code::Ok {
        return Err(tonic::Status::new(host.install_code, "installation acknowledgement unavailable"));
    }
    Ok(host.entry.clone().into())
});

operation!(backend::GetIdentity, host, _request, _input, {
    Ok(model::backend::wire::BackendIdentity {
        backend_id: ID.into(),
        epoch: ID.into(),
        api: Some(model::backend::wire::ApiIdentity {
            package: model::backend::wire::API_PACKAGE.into(),
            version: model::backend::wire::API_VERSION.into(),
            descriptor_sha256: host.descriptor_sha256.clone(),
        }),
        product_version: env!("CARGO_PKG_VERSION").into(),
        target: "x86_64-unknown-linux-gnu".into(),
        display_name: "Installation Fixture".into(),
        ..Default::default()
    }
    .into())
});

operation!(backend::GetHealth, _host, _request, _input, {
    Ok(model::backend::wire::BackendHealth {
        backend_id: ID.into(),
        epoch: ID.into(),
        readiness: model::backend::wire::Readiness::Ready,
        observed_at: None,
        message: String::new(),
    }
    .into())
});

operation!(backend::GetAvailability, _host, _request, _input, {
    Ok(model::backend::wire::BackendAvailability {
        backend_id: ID.into(),
        epoch: ID.into(),
        methods: [
            backend::GetIdentity::PATH,
            backend::GetHealth::PATH,
            backend::GetAvailability::PATH,
            runtime::GetIdentity::PATH,
            runtime::GetHealth::PATH,
            v1::Install::PATH,
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

operation!(runtime::GetIdentity, _host, _request, _input, {
    Ok(model::runtime::wire::Identity {
        backend_id: ID.into(),
        epoch: ID.into(),
        display_name: "Installation Fixture".into(),
        product_version: env!("CARGO_PKG_VERSION").into(),
    }
    .into())
});

operation!(runtime::GetHealth, _host, _request, _input, {
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

fn install_bundle(schema: u32) -> Bundle {
    use harmos_packaging::bundle::{Executable, Facility, Identity, Limits, Source};
    let owner = "fixture.worker";
    let declaration = serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":"1.0.0","description":"Install fixture","config":"()"});
    let policy = dokime::harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(120),
        attempt_timeout: std::time::Duration::from_secs(120),
        retries: dokime::harmos::operations::RetryPolicy::Never,
    };
    let lifecycle = [
        dokime::harmos::builtins::lifecycle::Descriptor::new(dokime::harmos::builtins::lifecycle::Action::Initialize, policy).unwrap(),
        dokime::harmos::builtins::lifecycle::Descriptor::new(dokime::harmos::builtins::lifecycle::Action::Shutdown, policy).unwrap(),
    ];
    let contracts =
        serde_json::json!({"kind":"contracts","facility":"sidecar","owner":owner,"provides":[],"requires":[],"lifecycle":lifecycle});
    let restart = serde_json::json!({"kind":"restart","facility":"sidecar","owner":owner,"policy":{"credit":0,"sustained_health_ms":30000,"backoff_base_ms":250,"backoff_cap_ms":10000}});
    let publications = serde_json::json!({"kind":"publications","facility":"sidecar","owner":owner,"streams":[]});
    let observers = serde_json::json!({"kind":"observers","facility":"sidecar","owner":owner,"observers":[]});
    let executable = format!(
        "inert{prefix}{declaration}\0{prefix}{contracts}\0{prefix}{restart}\0{prefix}{publications}\0{prefix}{observers}\0",
        prefix = std::str::from_utf8(dokime::harmos::artifacts::PREFIX).unwrap()
    );
    let mut registrations = dokime::harmos::sidecar::Registrar::new();
    registrations
        .register(
            "dokime.settings",
            schema,
            &serde_json::json!({
                "catalog": { "contributions": [
                    { "kind": "secret_reference", "id": "token", "default": { "kind": "environment", "variable": "PLUGIN_API_TOKEN" } },
                    { "kind": "secret_reference", "id": "optional", "default": null },
                    { "kind": "text_area", "id": "notes", "default": "fixture-notes" }
                ] }
            }),
        )
        .unwrap();
    Bundle::build(
        Identity {
            name: "install-fixture".into(),
            version: "1.0.0".parse().unwrap(),
            description: "Install fixture".into(),
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

fn run_cli(bundle: &Path, endpoint: &str) -> Result<Outcome, String> {
    let output = std::process::Command::new(cli_executable())
        .args(["--color", "never", "plugin", "install"])
        .arg(bundle)
        .args(["--endpoint", endpoint])
        .env("PLUGIN_API_TOKEN", SECRET_VALUE)
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|error| error.to_string())?;
    let mut bytes = output.stdout;
    bytes.extend_from_slice(&output.stderr);
    Ok(Outcome {
        success: output.status.success(),
        output: String::from_utf8_lossy(&bytes).into_owned(),
    })
}

async fn scenario(install_code: tonic::Code) -> (Outcome, Vec<&'static str>) {
    let bundle = install_bundle(3);
    let digest = Sha256::digest(bundle.bytes()).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), bundle.bytes()).unwrap();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let host = Host {
        entry: model::plugins::wire::PluginInstallation {
            install_id: ID.into(),
            sha256: digest,
            installed_at: None,
            state: model::plugins::wire::PluginInstallationState::Installed,
            last_boot: None,
            metadata: Some(model::plugins::wire::PluginMetadata {
                identity: Some(model::plugins::wire::PluginIdentity {
                    name: bundle.manifest().identity.name.clone(),
                    version: bundle.manifest().identity.version.to_string(),
                    description: bundle.manifest().identity.description.clone(),
                }),
                owners: vec!["fixture.worker".into()],
                size_bytes: bundle.bytes().len() as u64,
            }),
            error: None,
            active: false,
            restart_required: false,
            boot_error: None,
        },
        bytes: bundle.bytes().to_vec(),
        calls: calls.clone(),
        install_code,
        descriptor_sha256: {
            let catalog = dokime::harmos::operations::Contracts::of::<contracts::Operations>().unwrap();
            let artifacts = harmos_packaging::proto::Artifacts::new(&catalog).unwrap();
            Sha256::digest(artifacts.descriptors()).to_vec()
        },
    };
    let runtime = dokime::harmos::Runtime::<Fixture>::builder((), ()).register(Lanes::catalog()).runtime().await.unwrap();
    let host = Arc::new(host);
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(dokime::operations::hosting::Host::<v1::Install>::new(host.clone())).unwrap();
    dependencies.insert(dokime::operations::hosting::Host::<backend::GetIdentity>::new(host.clone())).unwrap();
    dependencies.insert(dokime::operations::hosting::Host::<backend::GetHealth>::new(host.clone())).unwrap();
    dependencies.insert(dokime::operations::hosting::Host::<backend::GetAvailability>::new(host.clone())).unwrap();
    dependencies.insert(dokime::operations::hosting::Host::<runtime::GetIdentity>::new(host.clone())).unwrap();
    dependencies.insert(dokime::operations::hosting::Host::<runtime::GetHealth>::new(host)).unwrap();
    let batch = dokime::harmos::RegistrationBatch::new(dependencies)
        .operation::<v1::Install>()
        .operation::<backend::GetIdentity>()
        .operation::<backend::GetHealth>()
        .operation::<backend::GetAvailability>()
        .operation::<runtime::GetIdentity>()
        .operation::<runtime::GetHealth>();
    runtime.register(batch.lane(Lanes::Open)).await.unwrap();
    let attachment = runtime
        .grpc()
        .attach(dokime::harmos::grpc::Settings {
            address: "127.0.0.1:0".parse().unwrap(),
            ..Default::default()
        })
        .await
        .unwrap();
    let endpoint = format!("http://{}", attachment.local_addr());
    let output = tokio::task::spawn_blocking(move || run_cli(file.path(), &endpoint)).await;
    attachment.detach().await.unwrap();
    runtime.stop().await.unwrap();
    let calls = calls.lock().unwrap().clone();
    (output.unwrap().unwrap(), calls)
}

#[tokio::test]
async fn direct_noninteractive_install() {
    let (outcome, calls) = scenario(tonic::Code::Ok).await;
    assert!(outcome.success, "{}", outcome.output);
    assert!(outcome.output.contains(ID));
    assert!(!outcome.output.contains(SECRET_VALUE));
    assert_eq!(calls, ["install"]);
}

#[tokio::test]
async fn uncertain_install_retained() {
    let (outcome, calls) = scenario(tonic::Code::Unavailable).await;
    assert!(!outcome.success);
    assert!(!outcome.output.contains(SECRET_VALUE));
    assert_eq!(calls, ["install"]);
}

// The built executable: nextest names it at run time, so an archive extracted on another runner finds it.
fn cli_executable() -> String {
    std::env::var("NEXTEST_BIN_EXE_dokime").ok().or_else(|| std::env::var("CARGO_BIN_EXE_dokime").ok()).expect("workspace executable")
}

/* === Tests === */
