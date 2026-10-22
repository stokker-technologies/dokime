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
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Arc,
};
// Third Party Imports
use async_trait::async_trait;
use dokime::harmos::sidecar::StreamDeclaration;
use serde_json::{Value, json};
// Local Imports
use crate::support::bridge::{Harness, Local};
use dokime::{InstalledPlugin, Runtime, RuntimeConfig, runtime::EmbeddedProvider};
use dokime::{
    execution::{CleanupContext, CriterionContext, InitializationContext, Runnable, StepContext, Workflow},
    schema::Described,
};
use dokime_gui::connection::ConnectionSession;
use dokime_gui::connection::{ConnectRequest, ConnectionCredentials, ConnectionTarget};
use dokime_server::grpc::{Config, Server};

/* === Constants === */

const CONTRACT: &str = "crates/dokime-gui/src-tauri/tests/fixtures/execution.json";

static ROOT: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

/* === Macros === */

/* === Type Aliases === */

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
    id = "matrix.lab.probe",
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
        let shared = Resources { root, writer: std::sync::Mutex::new(writer) };
        Ok((template, shared))
    }

    #[dokime::step(label = "Measure")]
    async fn measure(&mut self, ctx: &mut StepContext<'_>) -> dokime::Result<()> {
        for value in ctx.progress("Measuring", [1, 2]) {
            ctx.publish(Reading { value }).await;
            if value == 1 {
                ctx.publish(Phase { phase: "ready".into() }).await;
            }
            if !ctx.ask(Continue).await? {
                return Err(dokime::Error::PromptRefused("operator refused".into()));
            }
        }
        ctx.publish(Phase { phase: "done".into() }).await;
        Ok(())
    }

    async fn cleanup(&mut self, ctx: &mut CleanupContext<'_>) -> dokime::Result<()> {
        ctx.resources().writer.lock().unwrap().write_all(b"cleanup\n")?;
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
            suites: dokime::definition::SuiteSet::new([Lab::definition()]),
            ..Default::default()
        }
    }
    fn streams(&self) -> Vec<StreamDeclaration> {
        vec![StreamDeclaration {
            id: Reading::ID.into(),
            schema: Reading::VERSION,
            window: Reading::WINDOW,
            metadata: <Reading as dokime::harmos::StreamSpec>::metadata(),
            row_metadata: <Reading as dokime::harmos::RowMetadata>::row_metadata(),
            row: dokime::harmos::schema::graph::Schema::from_declaration(
                <Reading as Described>::DEFINITION,
                &dokime::harmos::config::SchemaSettings::default(),
            )
            .unwrap(),
        }]
    }
    async fn start(&self) -> dokime::runtime::Result<Vec<Arc<dyn Runnable>>> {
        *ROOT.lock().unwrap() = Some(self.0.root.clone());
        let lab = Lab::definition();
        Ok(vec![Arc::new(Workflow::<Probe>::new(&lab.suite_id, "matrix.rust").with_records(&lab.record_contracts)) as Arc<dyn Runnable>])
    }
    async fn stop(&self) -> dokime::runtime::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn root() -> PathBuf {
    ROOT.lock().unwrap().clone().expect("a case names the directory its execution traces into")
}

fn trace(root: &Path, stage: &str) {
    writeln!(OpenOptions::new().create(true).append(true).open(root.join("lifecycle.txt")).unwrap(), "{stage}").unwrap();
}

fn python(root: &Path) -> Command {
    let interpreter = if cfg!(windows) { ".venv/Scripts/python.exe" } else { ".venv/bin/python" };
    let mut command = Command::new(root.join(interpreter));
    command.current_dir(root).arg(root.join("packages/dokime-client/tests/acceptance/client/execution_matrix.py"));
    command
}

async fn desktop(root: &Path, endpoint: &str, scenario: &str) -> Value {
    let session = ConnectionSession::default();
    session
        .connect(ConnectRequest {
            target: ConnectionTarget::Local { endpoint: endpoint.into() },
            credentials: ConnectionCredentials::None,
            expected_backend_id: None,
        })
        .await
        .unwrap();
    let harness = Harness::new(session.clone());
    let mut child = Process(
        Command::new("node")
            .env("DOKIME_TEST_GENERATION", session.status().generation)
            .arg(root.join("crates/dokime-gui/src-tauri/tests/fixtures/test-execution-matrix.mjs"))
            .arg(root.join(CONTRACT))
            .arg(scenario)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    let mut result = None;
    let status = harness.drive(&mut child.0, |_, _| Local::Dispatch, |line| result = line.get("result").cloned());
    assert!(status.success(), "desktop scenario {scenario} failed");
    session.disconnect().unwrap();
    result.expect("desktop returned its observations")
}

async fn run_matrix(interface: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().unwrap();
    let contract: Value = serde_json::from_slice(&std::fs::read(root.join(CONTRACT)).unwrap()).unwrap();
    for scenario in contract["scenarios"].as_array().unwrap() {
        // Backend With The Matrix Provider
        let name = scenario["name"].as_str().unwrap();
        let directory = tempfile::tempdir().unwrap();
        let config = RuntimeConfig::new(
            directory.path().join("backend"),
            dokime::Metadata::new(dokime::Identifier::new("matrix").unwrap(), "Matrix", None),
        )
        .register(Arc::new(Provider(State { root: directory.path().into() })));
        let mut runtime = Runtime::start(config).await.unwrap();
        let server = Server::start(runtime.handle(), Config::local()).await.unwrap();
        let endpoint = server.local_endpoint();

        // Client Observation
        let observed = if interface == "desktop" {
            desktop(&root, &endpoint, name).await
        } else {
            let output = python(&root).arg(&endpoint).arg(root.join(CONTRACT)).arg(name).output().unwrap();
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
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
        assert_eq!(std::fs::read_to_string(directory.path().join("lifecycle.txt")).unwrap(), "open\ncleanup\nclose\n");

        server.shutdown().await.unwrap();
        runtime.shutdown().await.unwrap();
    }
}

/* === Tests === */

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn python_runs_matrix() {
    run_matrix("python").await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn desktop_runs_matrix() {
    run_matrix("desktop").await;
}
