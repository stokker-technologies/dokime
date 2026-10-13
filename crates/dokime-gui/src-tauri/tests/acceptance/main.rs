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
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Arc,
};
// Third Party Imports
use async_trait::async_trait;
use harmos::plugin::sidecar::StreamDeclaration;
use serde_json::{Value, json};
// Local Imports
use dokime::{
    execution::{
        CleanupContext, CriterionContext, InitializationContext, Runnable, StepContext, Workflow,
    },
    schema::Described,
};
use dokime_app::{InstalledPlugin, Runtime, RuntimeConfig, runtime::EmbeddedProvider};
use dokime_gui::connection::{ConnectRequest, ConnectionCredentials, ConnectionTarget};
use dokime_gui::{
    connection::ConnectionSession,
    uploads::{Bridge, artifact_upload_handler},
};
use dokime_server::grpc::{Config, Server};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Constants === */
static ROOT: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

/* === Structs === */
#[derive(Clone)]
struct State {
    root: PathBuf,
}

struct Resources {
    root: PathBuf,
    writer: std::sync::Mutex<dokime::artifacts::ArtifactWriter>,
}

struct Provider(State);

struct Lab;

struct Probe;

struct Process(Child);

#[dokime::record(id = "matrix.phase", label = "Phase")]
struct Phase {
    phase: String,
}

#[dokime::stream(id = "matrix.reading", window = 32, label = "Reading")]
struct Reading {
    value: i64,
}

#[dokime::prompt(id = "matrix.continue", label = "Continue?")]
struct Continue;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

impl Drop for Resources {
    fn drop(&mut self) {
        trace(&self.root, "close");
    }
}

#[dokime::catalog]
enum Templates {
    Probe(Probe),
}

#[dokime::catalog]
enum Streams {
    Reading(Reading),
}

#[dokime::catalog]
enum Records {
    Phase(Phase),
}

#[dokime::catalog]
enum Prompts {
    Continue(Continue),
}

#[dokime::suite(id = "matrix.lab", label = "Lab", templates = Templates)]
impl Lab {}

#[dokime::template(
    id = "matrix.probe",
    label = "Probe",
    records = Records,
    streams = Streams,
    prompts = Prompts
)]
impl Probe {
    async fn initialize(ctx: &mut InitializationContext<'_>) -> dokime::Result<(Self, Resources)> {
        let template = Self;

        let root = root();
        trace(&root, "open");
        let mut writer = ctx.artifacts().create("matrix.txt")?;
        writer.write_all(b"open\n")?;
        let shared = Resources {
            root,
            writer: std::sync::Mutex::new(writer),
        };
        Ok((template, shared))
    }

    #[dokime::step(label = "Measure")]
    async fn measure(&mut self, ctx: &mut StepContext<'_>) -> dokime::Result<()> {
        for value in ctx.progress("Measuring", [1, 2]) {
            ctx.publish(Reading { value }).await;
            if value == 1 {
                ctx.publish(Phase {
                    phase: "ready".into(),
                })
                .await;
            }
            if !ctx.ask(Continue).await? {
                return Err(dokime::Error::PromptRefused("operator refused".into()));
            }
        }
        ctx.publish(Phase {
            phase: "done".into(),
        })
        .await;
        Ok(())
    }

    async fn cleanup(&mut self, ctx: &mut CleanupContext<'_>) -> dokime::Result<()> {
        ctx.resources()
            .writer
            .lock()
            .unwrap()
            .write_all(b"cleanup\n")?;
        trace(&ctx.resources().root, "cleanup");
        Ok(())
    }

    #[dokime::criterion(id = "matrix.accepted", label = "Accepted")]
    async fn accepted(ctx: &CriterionContext<'_>) -> dokime::Result<bool> {
        Ok(ctx.samples::<Reading>().len() == 2)
    }
}

#[async_trait]
impl EmbeddedProvider for Provider {
    fn catalog(&self) -> InstalledPlugin {
        InstalledPlugin {
            source: "matrix.rust".into(),
            version: "0.1.0".into(),
            enabled: true,
            suites: dokime_app::definition::SuiteSet::new([Lab::definition()]),
            ..Default::default()
        }
    }
    fn streams(&self) -> Vec<StreamDeclaration> {
        vec![StreamDeclaration {
            id: Reading::ID.into(),
            schema: Reading::VERSION,
            window: Reading::WINDOW,
            metadata: <Reading as harmos::StreamSpec>::metadata(),
            row_metadata: <Reading as harmos::RowMetadata>::row_metadata(),
            row: harmos::schema::graph::Schema::from_declaration(
                <Reading as Described>::DEFINITION,
                &harmos::config::SchemaSettings::default(),
            )
            .unwrap(),
        }]
    }
    async fn start(&self) -> dokime_app::Result<Vec<Arc<dyn Runnable>>> {
        *ROOT.lock().unwrap() = Some(self.0.root.clone());
        let lab = Lab::definition();
        Ok(vec![Arc::new(
            Workflow::<Probe>::new(&lab.suite_id, "matrix.rust")
                .with_records(&lab.record_contracts),
        ) as Arc<dyn Runnable>])
    }
    async fn stop(&self) -> dokime_app::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn invoke(
    window: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    cmd: &str,
    bytes: Vec<u8>,
) -> Result<tauri::ipc::InvokeResponseBody, serde_json::Value> {
    tauri::test::get_ipc_response(
        window,
        tauri::webview::InvokeRequest {
            cmd: cmd.into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url: if cfg!(windows) {
                "http://tauri.localhost"
            } else {
                "tauri://localhost"
            }
            .parse()
            .unwrap(),
            body: tauri::ipc::InvokeBody::Raw(bytes),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.into(),
        },
    )
}

fn root() -> PathBuf {
    ROOT.lock()
        .unwrap()
        .clone()
        .expect("a case names the directory its execution traces into")
}

fn trace(root: &Path, stage: &str) {
    writeln!(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(root.join("lifecycle.txt"))
            .unwrap(),
        "{stage}"
    )
    .unwrap();
}

fn python(root: &Path) -> Command {
    let interpreter = if cfg!(windows) {
        ".venv/Scripts/python.exe"
    } else {
        ".venv/bin/python"
    };
    let mut command = Command::new(root.join(interpreter));
    command
        .current_dir(root)
        .arg(root.join("packages/dokime/tests/acceptance/client/execution_matrix.py"));
    command
}

async fn desktop(root: &Path, endpoint: &str, scenario: &str) -> Value {
    let session = ConnectionSession::default();
    session
        .connect(ConnectRequest {
            target: ConnectionTarget::Local {
                endpoint: endpoint.into(),
            },
            credentials: ConnectionCredentials::None,
            expected_backend_id: None,
        })
        .await
        .unwrap();
    let app = tauri::test::mock_builder()
        .manage(session.clone())
        .manage(Bridge::default())
        .invoke_handler(artifact_upload_handler())
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .unwrap();
    let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let snapshot = session.status();
    let backend = snapshot
        .backend
        .as_ref()
        .expect("the desktop fixture has negotiated a backend");
    let browser_access = json!({
        "generation": snapshot.generation,
        "endpoint": snapshot.endpoint,
        "authorization": null,
        "backendId": backend.backend_id,
        "epoch": backend.epoch,
        "limits": backend.limits,
        "methods": backend.methods,
    });
    let mut child =
        Process(
            Command::new("node")
                .env("DOKIME_TEST_BROWSER_ACCESS", browser_access.to_string())
                .arg(root.join(
                    "crates/dokime-gui/src-tauri/tests/rpc/fixtures/test-execution-matrix.mjs",
                ))
                .arg(root.join("crates/dokime-gui/src-tauri/tests/rpc/fixtures/execution.json"))
                .arg(scenario)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap(),
        );
    let mut input = child.0.stdin.take().unwrap();
    let output = BufReader::new(child.0.stdout.take().unwrap());
    let mut result = None;
    for line in output.lines() {
        let request: Value = serde_json::from_str(&line.unwrap()).unwrap();
        if let Some(value) = request.get("result") {
            result = Some(value.clone());
            continue;
        }
        let bytes = serde_json::from_value(request["bytes"].clone()).unwrap();
        let response = match invoke(&window, request["cmd"].as_str().unwrap(), bytes) {
            Ok(tauri::ipc::InvokeResponseBody::Raw(bytes)) => json!({"bytes":bytes}),
            Ok(tauri::ipc::InvokeResponseBody::Json(value)) => {
                json!({"value":serde_json::from_str::<Value>(&value).unwrap()})
            }
            Err(error) => json!({"error":error}),
        };
        writeln!(input, "{response}").unwrap();
        input.flush().unwrap();
    }
    assert!(
        child.0.wait().unwrap().success(),
        "desktop scenario {scenario} failed"
    );
    session.disconnect().unwrap();
    result.expect("desktop returned its observations")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn shared_execution_scenarios() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let contract: Value = serde_json::from_slice(
        &std::fs::read(root.join("crates/dokime-gui/src-tauri/tests/rpc/fixtures/execution.json"))
            .unwrap(),
    )
    .unwrap();
    for interface in ["python", "desktop"] {
        for scenario in contract["scenarios"].as_array().unwrap() {
            let name = scenario["name"].as_str().unwrap();
            eprintln!("execution matrix: Rust Backend / {interface} / {name}");
            let directory = tempfile::tempdir().unwrap();
            let config = RuntimeConfig::new(
                directory.path().join("backend"),
                dokime::core::Metadata::new(
                    dokime::core::Identifier::new("matrix").unwrap(),
                    "Matrix",
                    None,
                ),
            )
            .register(Arc::new(Provider(State {
                root: directory.path().into(),
            })));
            let mut runtime = Runtime::start(config).await.unwrap();
            let server = Server::start(runtime.handle(), Config::local())
                .await
                .unwrap();
            let endpoint = server.local_endpoint();
            let observed = if interface == "desktop" {
                desktop(&root, &endpoint, name).await
            } else {
                let output = python(&root)
                    .arg(&endpoint)
                    .arg(root.join("crates/dokime-gui/src-tauri/tests/rpc/fixtures/execution.json"))
                    .arg(name)
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                serde_json::from_slice(&output.stdout).unwrap()
            };
            assert_eq!(
                observed,
                json!({
                    "status":scenario["status"], "phase":scenario["phase"],
                    "values":scenario["values"], "records":scenario["records"],
                    "cases":1,"executions":1,"filtered":1,"live":1,
                    "artifact":"open\ncleanup\n", "progress":{"completed":if name == "complete" {1} else {0},"total":2},
                }),
                "Rust Backend/{interface}/{name}"
            );
            assert_eq!(
                std::fs::read_to_string(directory.path().join("lifecycle.txt")).unwrap(),
                "open\ncleanup\nclose\n"
            );
            server.shutdown().await.unwrap();
            runtime.shutdown().await.unwrap();
        }
    }
}

/* === Tests === */
