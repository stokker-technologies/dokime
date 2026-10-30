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
use serde_json::{Value, json};
// Local Imports
use crate::support::bridge::{Harness, Local};
use dokime::execution::{CleanupContext, CriterionContext, InitializationContext, StepContext};
use dokime::{Runtime, RuntimeConfig};
use dokime_gui::connection::ConnectionSession;
use dokime_gui::connection::{ConnectRequest, ConnectionSecrets, ConnectionTarget};
use dokime_server::grpc::Config;

/* === Constants === */

const CONTRACT: &str = "crates/dokime-gui/src-tauri/tests/fixtures/execution.json";

static ROOT: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    root: PathBuf,
    writer: std::sync::Mutex<dokime::artifacts::ArtifactWriter>,
}

struct Provider;

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

#[dokime::catalog]
enum Suites {
    Lab(Lab),
}

impl dokime::harmos::builtins::lifecycle::Policy for Provider {
    const INITIALIZATION_POLICY: dokime::harmos::operations::OperationPolicy = dokime::harmos::builtins::lifecycle::POLICY;
    const SHUTDOWN_POLICY: dokime::harmos::operations::OperationPolicy = Self::INITIALIZATION_POLICY;
}

#[dokime::plugin(id = "matrix_rust", requires(), contributions(dokime::Suites<Suites>), restart = "never")]
impl dokime::Plugin for Provider {
    const CATALOGS: &'static [dokime::harmos::Catalog] = &[];
}

impl dokime::harmos::Lifecycle for Provider {
    type Config = dokime::plugin::Configuration;
    type Error = dokime::Refusal;
    type Context = dokime::Supervisor;

    async fn prepare(_: Self::Config, setup: &mut dokime::harmos::Setup<Self>) -> Result<Self, Self::Error> {
        setup.resource::<()>(Arc::new(()));
        dokime::execution::prepare::<Self, Suites>(setup)?;
        Ok(Self)
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
    let mut command = dokime::domain::process::command(root.join(interpreter));
    command.current_dir(root).arg(root.join("packages/dokime-client/tests/acceptance/client/execution_matrix.py"));
    command
}

async fn desktop(root: &Path, endpoint: &str, scenario: &str) -> Value {
    let session = ConnectionSession::default();
    session
        .connect(ConnectRequest {
            target: ConnectionTarget::Local { endpoint: endpoint.into() },
            secrets: ConnectionSecrets::None,
            expected_backend_id: None,
        })
        .await
        .unwrap();
    let harness = Harness::new(session.clone());
    let mut child = Process(
        dokime::domain::process::command("node")
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
    let root = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("nextest sets the manifest directory"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let contract: Value = serde_json::from_slice(&std::fs::read(root.join(CONTRACT)).unwrap()).unwrap();
    for scenario in contract["scenarios"].as_array().unwrap() {
        // Backend With The Matrix Plugin
        let name = scenario["name"].as_str().unwrap();
        let directory = tempfile::tempdir().unwrap();
        *ROOT.lock().unwrap() = Some(directory.path().into());
        let config = dokime_runtime::testing::plugin::<Provider>(
            RuntimeConfig::new(
                directory.path().join("backend"),
                dokime::Metadata::new(dokime::Identifier::new("matrix").unwrap(), "Matrix", None),
            ),
            &directory.path().join("plugins"),
        )
        .unwrap();
        let prepared_runtime = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
        let mut runtime = Runtime::start(prepared_runtime.configuration(config)).await.unwrap();
        let server = prepared_runtime.ready().await.unwrap().listen().await.unwrap();
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
