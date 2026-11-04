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
use harmos_packaging::bundle::Bundle;
use sha2::{Digest, Sha256};

// Local Imports
use crate::backend::{Fixture, FixtureConfig, Host, ID};
use dokime::client::contracts;
use dokime::domain::model;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Outcome {
    success: bool,
    output: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn install_bundle(schema: u32) -> Bundle {
    use harmos_packaging::bundle::{Executable, Facility, Identity, Limits, Source};
    let owner = "fixture_worker";
    let declaration = serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":"1.0.0","description":"Install fixture","config":"()"});
    let policy = dokime::harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(240),
        attempt_timeout: std::time::Duration::from_secs(240),
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

#[dokime_tracing::trace_test]
#[tokio::test]
async fn install() {
    let install_code = Arc::new(Mutex::new(tonic::Code::Ok));
    let bundle = install_bundle(3);
    let digest = Sha256::digest(bundle.bytes()).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), bundle.bytes()).unwrap();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let host = Host {
        report: None,
        entry: Some(model::plugins::wire::PluginInstallation {
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
            restart: model::settings::wire::PluginRestart::Idle,
            boot_failed: false,
        }),
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
