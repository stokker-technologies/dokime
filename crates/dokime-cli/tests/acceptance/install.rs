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
use dokime::harmos::Context;
use dokime::harmos::operations::{OperationDeclaration, Response};
use dokime::operations::hosting::{Serve, Served};
use harmos_packaging::bundle::Bundle;
use sha2::{Digest, Sha256};

// Local Imports
use dokime::client::contracts::{self, Endpoint, backend, plugins as v1, runtime};
use dokime::domain::model;

/* === Constants === */

const ID: &str = "01900000-0000-7000-8000-000000000001";
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
    install_code: Arc<Mutex<tonic::Code>>,
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
    let code = *host.install_code.lock().unwrap();
    if code != tonic::Code::Ok {
        return Err(tonic::Status::new(code, "installation acknowledgement unavailable"));
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

#[derive(Default, serde::Deserialize)]
struct FixtureConfig {
    #[serde(skip)]
    host: Option<Arc<Host>>,
}

#[harmos_sdk::catalog(harmos_path = dokime::harmos)]
enum HostOperations {
    Operation0(v1::Install),
    Operation1(backend::GetIdentity),
    Operation2(backend::GetHealth),
    Operation3(backend::GetAvailability),
    Operation4(runtime::GetIdentity),
    Operation5(runtime::GetHealth),
}

impl dokime::harmos::Lifecycle for Fixture {
    type Config = FixtureConfig;
    type Error = dokime::harmos::Error;
    type Context = dokime::harmos::Startup<Self>;
    async fn prepare(config: Self::Config, setup: &mut dokime::harmos::Setup<Self>) -> dokime::harmos::Result<Self> {
        let host = config.host.expect("fixture hosting resources");
        setup.resource::<dokime::operations::hosting::Host<v1::Install>>(dokime::operations::hosting::Host::<v1::Install>::new(
            host.clone(),
        ));
        setup.resource::<dokime::operations::hosting::Host<backend::GetIdentity>>(
            dokime::operations::hosting::Host::<backend::GetIdentity>::new(host.clone()),
        );
        setup.resource::<dokime::operations::hosting::Host<backend::GetHealth>>(
            dokime::operations::hosting::Host::<backend::GetHealth>::new(host.clone()),
        );
        setup.resource::<dokime::operations::hosting::Host<backend::GetAvailability>>(dokime::operations::hosting::Host::<
            backend::GetAvailability,
        >::new(host.clone()));
        setup.resource::<dokime::operations::hosting::Host<runtime::GetIdentity>>(
            dokime::operations::hosting::Host::<runtime::GetIdentity>::new(host.clone()),
        );
        setup.resource::<dokime::operations::hosting::Host<runtime::GetHealth>>(
            dokime::operations::hosting::Host::<runtime::GetHealth>::new(host.clone()),
        );
        setup.operations::<HostOperations>(Lanes::Open);
        setup.operations::<Discovery>(Lanes::Open);
        Ok(Self)
    }
    async fn initialize(&mut self, _: &Self::Context) -> dokime::harmos::Result<()> {
        Ok(())
    }
    async fn stop(&mut self, _: &Self::Context) -> dokime::harmos::Result<()> {
        Ok(())
    }
    async fn cleanup(&mut self) -> dokime::harmos::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn install_bundle(schema: u32) -> Bundle {
    use harmos_packaging::bundle::{Executable, Facility, Identity, Limits, Source};
    let owner = "fixture_worker";
    let declaration = serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":"1.0.0","description":"Install fixture","config":"()"});
    let policy = dokime::harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(120),
        attempt_timeout: std::time::Duration::from_secs(120),
        cancellation_timeout: std::time::Duration::ZERO,
        retries: dokime::harmos::operations::RetryPolicy::Never,
    };
    let lifecycle = [dokime::harmos::builtins::lifecycle::Action::Initialize, dokime::harmos::builtins::lifecycle::Action::Shutdown]
        .map(|action| serde_json::json!({"action":action,"identity":"harmos.operation.v1","policy":policy}));
    let contracts =
        serde_json::json!({"kind":"contracts","facility":"sidecar","owner":owner,"entry":{"direction":"lifecycle","lifecycle":lifecycle}});
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
                    { "kind": "secret", "id": "token", "title": "Token", "required": true },
                    { "kind": "secret", "id": "optional", "title": "Optional", "required": false },
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
    let output = dokime::domain::process::command(cli_executable())
        .args(["--color", "never", "plugin", "install"])
        .arg(bundle)
        .args(["--endpoint", endpoint])
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

#[tokio::test]
async fn install() {
    let install_code = Arc::new(Mutex::new(tonic::Code::Ok));
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
                owners: vec!["fixture_worker".into()],
                size_bytes: bundle.bytes().len() as u64,
            }),
            error: None,
            active: false,
            restart_required: false,
            boot_failed: false,
        },
        bytes: bundle.bytes().to_vec(),
        calls: calls.clone(),
        install_code: Arc::clone(&install_code),
        descriptor_sha256: {
            let catalog = dokime::harmos::operations::Contracts::of::<contracts::Operations>().unwrap();
            let artifacts = harmos_packaging::proto::Artifacts::new(&catalog).unwrap();
            Sha256::digest(artifacts.descriptors()).to_vec()
        },
    };
    let runtime = dokime::harmos::Runtime::<Fixture>::launch((), FixtureConfig { host: Some(Arc::new(host)) }).await.unwrap();
    let attachment = runtime
        .grpc()
        .attach(dokime::harmos::grpc::Settings {
            address: "127.0.0.1:0".parse().unwrap(),
            ..Default::default()
        })
        .await
        .unwrap();
    let endpoint = format!("http://{}", attachment.local_addr());
    // One Provider And Bundle Exercise Success Followed By An Uncertain Acknowledgement.
    for (index, code) in [tonic::Code::Ok, tonic::Code::Unavailable].into_iter().enumerate() {
        *install_code.lock().unwrap() = code;
        let path = file.path().to_owned();
        let endpoint = endpoint.clone();
        let outcome = tokio::task::spawn_blocking(move || run_cli(&path, &endpoint)).await.unwrap().unwrap();
        assert_eq!(outcome.success, code == tonic::Code::Ok, "{}", outcome.output);
        if outcome.success {
            assert!(outcome.output.contains(ID));
        }
        assert_eq!(*calls.lock().unwrap(), vec!["install"; index + 1]);
    }
    attachment.detach().await.unwrap();
    runtime.shutdown().await.unwrap();
}

// The built executable: nextest names it at run time, so an archive extracted on another runner finds it.
fn cli_executable() -> String {
    std::env::var("NEXTEST_BIN_EXE_dokime").ok().or_else(|| std::env::var("CARGO_BIN_EXE_dokime").ok()).expect("workspace executable")
}

/* === Tests === */

#[harmos_sdk::catalog(harmos_path = dokime::harmos)]
enum Discovery {
    Catalog(dokime::harmos::builtins::host::Catalog),
}
