/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Interrupt Actual Remote Scripts Through Their Native Terminals

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
};

// Third Party Imports
use hyper::{Request, body::Incoming, service::service_fn};
use hyper_util::rt::{TokioExecutor, TokioIo};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::oneshot,
    task::{JoinHandle, JoinSet},
};
use tokio_util::sync::CancellationToken;

// Local Imports
use dokime::client::{Client, TestCycleHandle, TestExecutionHandle};
use dokime::domain::model::identity::wire::Metadata;
use dokime::domain::model::planning::wire::{RegisteredSelection, RegisteredSelectionParametersEntry, SelectionValues};
use dokime::domain::model::{
    execution::{ExecutionMode, result::CyclePhase, wire::ResultStatus},
    identity::{TestCycleId, TestExecutionId},
    values::Value,
};
use dokime::operations::{
    Endpoint,
    execution::{AppendExecutions, CreateCycle, ExecutionBinding, ListCycles, StartExecution},
    planning::MaterializeTemplate,
};

/* === Constants === */

// Every scenario's script waits at most 60 s on its own; past this the script is stuck.
const SCRIPT_DEADLINE: std::time::Duration = std::time::Duration::from_secs(300);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Process {
    child: Box<dyn portable_pty::Child + Send + Sync>,
    reaped: bool,
}

struct Outcome {
    success: bool,
    triggered: bool,
    text: String,
}

struct TerminalOutput {
    text: String,
    execution: Option<TestExecutionId>,
    native_input: bool,
    cancellation_requested: bool,
    batch_prompt: bool,
}

struct TerminalInput {
    writer: Box<dyn Write + Send>,
    native: bool,
    triggered: bool,
    batch_prompt: bool,
}

struct Proxy {
    endpoint: String,
    cancellation: CancellationToken,
    task: Option<JoinHandle<()>>,
}

#[derive(Clone)]
struct Gate {
    client: Client,
    label: String,
    boundary: Boundary,
    reached: CancellationToken,
    released: CancellationToken,
}

/* === Enums === */

enum Action {
    Interrupt(Client),
    InterruptOnSample(Client),
    Disconnect(Client, oneshot::Sender<()>),
    InterruptAt { reached: CancellationToken, released: CancellationToken },
}

enum TerminalKey {
    Interrupt,
    ContinueBatch,
}

#[derive(Clone, Copy)]
enum Boundary {
    Append,
    Start,
}

/* === Traits === */

/* === Implementations === */

impl Drop for Process {
    fn drop(&mut self) {
        if self.reaped {
            return;
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for Proxy {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

impl Proxy {
    async fn start(endpoint: &str) -> Self {
        Self::with_gate(endpoint, None).await
    }

    async fn with_gate(endpoint: &str, gate: Option<Gate>) -> Self {
        let target = endpoint.strip_prefix("http://").expect("the built host fixture uses a plaintext loopback endpoint").to_owned();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let cancellation = CancellationToken::new();
        let stopped = cancellation.clone();
        let task = tokio::spawn(async move {
            let mut connections = JoinSet::new();
            loop {
                tokio::select! {
                    biased;
                    _ = stopped.cancelled() => break,
                    connection = listener.accept() => {
                        let (incoming, _) = connection.unwrap();
                        let target = target.clone();
                        let cancelled = stopped.clone();
                        let gate = gate.clone();
                        connections.spawn(async move {
                            tokio::select! {
                                biased;
                                _ = cancelled.cancelled() => {},
                                _ = async {
                                    let outgoing = TcpStream::connect(target).await?;
                                    relay(incoming, outgoing, gate).await?;
                                    Ok::<(), std::io::Error>(())
                                } => {},
                            }
                        });
                    }
                }
            }
            drop(listener);
            connections.shutdown().await;
        });
        Self { endpoint, cancellation, task: Some(task) }
    }

    async fn close(mut self) {
        self.cancellation.cancel();
        self.task.take().unwrap().await.unwrap();
    }
}

impl Gate {
    fn selects(&self, path: &str) -> bool {
        let selected = match self.boundary {
            Boundary::Append => AppendExecutions::PATH,
            Boundary::Start => StartExecution::PATH,
        };
        path == selected
    }

    async fn hold(&self) -> std::io::Result<()> {
        if self.released.is_cancelled() {
            return Ok(());
        }
        if !self.reached.is_cancelled() && !self.committed().await? {
            return Ok(());
        }
        self.reached.cancel();
        let deadline = std::time::Duration::from_secs(120);
        let release = tokio::time::timeout(deadline, self.released.cancelled());
        release.await.map_err(|_| std::io::Error::other("native recovery did not release its held mutation response"))?;
        Ok(())
    }

    async fn committed(&self) -> std::io::Result<bool> {
        use dokime::domain::model::execution::wire::CycleFilter;
        use dokime::operations::execution::ListCycles;

        // Locate The Script's Cycle Without Depending On Its Pending Reply.
        let filter = CycleFilter {
            text: self.label.clone(),
            ..Default::default()
        };
        let request = ListCycles {
            limit: 100,
            filter: Some(filter),
            ..Default::default()
        };
        let reply = self.client.call(request).await.map_err(|error| std::io::Error::other(error.to_string()))?;
        let cycle = reply.cycles.into_iter().find(|cycle| cycle.metadata.as_ref().is_some_and(|metadata| metadata.label == self.label));
        let Some(cycle) = cycle else {
            return Ok(false);
        };
        if cycle.execution_count == 0 {
            return Ok(false);
        }
        if matches!(self.boundary, Boundary::Append) {
            return Ok(true);
        }

        // Start Is Confirmed By Admission, Including Queued Admission.
        let id = TestCycleId::from_uuid(&cycle.cycle_id).unwrap();
        let cycle = self.client.cycle(id);
        let members = cycle.executions().await.map_err(|error| std::io::Error::other(error.to_string()))?;
        for member in members {
            let snapshot = member.snapshot().await.map_err(|error| std::io::Error::other(error.to_string()))?;
            let request_id = snapshot.controls.and_then(|controls| controls.request_id);
            if request_id.is_some() {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

async fn relay(incoming: TcpStream, outgoing: TcpStream, gate: Option<Gate>) -> std::io::Result<()> {
    // Hold Only The Selected RPC Reply; The Drivers Continue Servicing HTTP/2 Control Frames.
    let (sender, upstream) = hyper::client::conn::http2::handshake::<_, _, Incoming>(TokioExecutor::new(), TokioIo::new(outgoing))
        .await
        .map_err(std::io::Error::other)?;
    let service = service_fn(move |request: Request<Incoming>| {
        let mut sender = sender.clone();
        let gate = gate.clone().filter(|gate| gate.selects(request.uri().path()));
        async move {
            let response = sender.send_request(request).await.map_err(std::io::Error::other)?;
            if let Some(gate) = gate {
                gate.hold().await?;
            }
            Ok::<_, std::io::Error>(response)
        }
    });
    let builder = hyper::server::conn::http2::Builder::new(TokioExecutor::new());
    let downstream = builder.serve_connection(TokioIo::new(incoming), service);
    tokio::select! {
        result = upstream => result.map_err(std::io::Error::other),
        result = downstream => result.map_err(std::io::Error::other),
    }
}

/* === Public Functions === */

pub async fn scripts(client: &Client, endpoint: &str, root: &Path, extension: &str) {
    std::fs::create_dir_all(root).unwrap();
    // Box Each Story And Keep Export Admission Available For Its Recovery.
    let (other_cycle, other_execution) = Box::pin(waiting_execution(client)).await;
    Box::pin(scenario(client, endpoint, root, extension, "test")).await;
    Box::pin(scenario(client, endpoint, root, extension, "prompt")).await;
    if extension != "py" {
        Box::pin(mutation_interruption(client, endpoint, root, extension, Boundary::Append)).await;
        Box::pin(mutation_interruption(client, endpoint, root, extension, Boundary::Start)).await;
    }
    let summary = other_execution.snapshot().await.unwrap();
    assert!(!summary.summary.unwrap().sealed, "another client's execution must survive every script interrupt");
    assert_eq!(other_cycle.snapshot().await.unwrap().phase, CyclePhase::Active);
    other_cycle.seal().await.unwrap();
    let saved = other_execution.wait(None, None).await.unwrap();
    assert_eq!(saved.document.unwrap().execution.unwrap().result.unwrap().status, ResultStatus::Cancelled);
    if extension == "py" {
        Box::pin(partial_capture(client, endpoint, root)).await;
    }
}

pub async fn disconnections(client: &Client, endpoint: &str, root: &Path) {
    std::fs::create_dir_all(root).unwrap();
    let (other_cycle, other_execution) = Box::pin(waiting_execution(client)).await;
    let extensions: &[&str] = if cfg!(windows) { &["ps1", "bat", "py"] } else { &["sh", "py"] };
    for extension in extensions {
        Box::pin(disconnected(client, endpoint, root, extension, "test")).await;
        Box::pin(disconnected(client, endpoint, root, extension, "prompt")).await;
        let summary = other_execution.snapshot().await.unwrap();
        assert!(!summary.summary.unwrap().sealed, "another client's execution must survive every script disconnect");
        assert_eq!(other_cycle.snapshot().await.unwrap().phase, CyclePhase::Active);
    }
    other_cycle.seal().await.unwrap();
    let saved = other_execution.wait(None, None).await.unwrap();
    assert_eq!(saved.document.unwrap().execution.unwrap().result.unwrap().status, ResultStatus::Cancelled);
}

/* === Private Functions === */

async fn waiting_execution(client: &Client) -> (TestCycleHandle, TestExecutionHandle) {
    let changes = client.stream(dokime::client::contracts::observation::WatchRuntime::default()).await.unwrap();
    let parameters = vec![RegisteredSelectionParametersEntry {
        key: "require_confirmation".into(),
        value: Some(SelectionValues {
            values: vec![Value::Bool(true).try_into().unwrap()],
        }),
    }];
    let (_, template) = client.template("obc.checks.pulse").await.unwrap();
    let selection = RegisteredSelection {
        suite_id: "obc.checks".into(),
        template_id: template.template_id,
        parameters,
        files: Vec::new(),
    };
    let materialized = client.materialize_template(MaterializeTemplate { selection: Some(selection) }).await.unwrap();
    assert_eq!(materialized.cases.len(), 1);
    let metadata = Metadata {
        identifier: String::new(),
        label: "Independent Waiting Operator".into(),
        description: None,
        tags: Vec::new(),
        attributes: Vec::new(),
        created_at: None,
        updated_at: None,
    };
    let request = CreateCycle {
        metadata: Some(metadata),
        continue_automatically: false,
        context: None,
    };
    let cycle = client.create_cycle(request).await.unwrap();
    let binding = ExecutionBinding {
        execution_id: uuid::Uuid::now_v7().to_string(),
        case_id: materialized.cases[0].case_id.clone(),
    };
    cycle.append(vec![binding]).await.unwrap();
    let mut members = cycle.executions().await.unwrap();
    let execution = members.pop().unwrap();
    execution.start(ExecutionMode::Automatic).await.unwrap();

    loop {
        let pending = execution.pending_prompts().await.unwrap();
        let prompt = pending.message().await.unwrap();
        pending.close().await;
        if prompt.is_some() {
            break;
        }
        changes.message().await.unwrap().expect("the runtime watch stays open");
    }
    changes.close().await;
    (cycle, execution)
}

async fn mutation_interruption(client: &Client, endpoint: &str, root: &Path, extension: &str, boundary: Boundary) {
    // Hold The Mutation Reply Only After Its Server Effect Is Observable.
    let (name, workflow) = match boundary {
        Boundary::Append => ("append confirmation", "test"),
        Boundary::Start => ("start confirmation", "prompt"),
    };
    eprintln!("Remote Interruption: {extension} {name}");
    let directory = root.join(format!("{extension} {name}"));
    let label = format!("Remote {extension} {name} {}", uuid::Uuid::now_v7());
    let gate = Gate {
        client: client.clone(),
        label: label.clone(),
        boundary,
        reached: CancellationToken::new(),
        released: CancellationToken::new(),
    };
    let proxy = Proxy::with_gate(endpoint, Some(gate.clone())).await;
    let parameters = [("EXECUTION_COUNT", "1"), ("CYCLE_LABEL", label.as_str())];
    let command = super::configured_command(extension, workflow, &proxy.endpoint, &directory, &parameters).await;
    let command = terminal_command(command.as_std(), extension == "bat");

    // Ctrl+C Arrives Before Confirmation; Cleanup Must Reconcile Exact Membership.
    let checked = CancellationToken::new();
    let action = Action::InterruptAt {
        reached: checked.clone(),
        released: gate.released.clone(),
    };
    let mut process = tokio::task::spawn_blocking(move || run(command, "Cycle: ", action));
    let mut probed = None;
    let outcome = tokio::select! {
        outcome = &mut process => outcome.unwrap(),
        _ = gate.reached.cancelled() => {
            // Another RPC Must Complete Through The Same Proxy While The Mutation Reply Is Held.
            let target = dokime::client::Target::local(&proxy.endpoint).unwrap();
            let reply = async {
                let probe = Client::connect(target, None, None).await?;
                probe.call(ListCycles {
                    limit: 100,
                    filter: Some(dokime::domain::model::execution::wire::CycleFilter {
                        text: label.clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }).await
            }.await;
            probed = Some((reply, !gate.released.is_cancelled()));
            checked.cancel();
            process.await.unwrap()
        }
    };
    proxy.close().await;
    assert!(outcome.triggered, "{name} was not interrupted:\n{}", outcome.text);
    let (reply, held) = probed.expect("the interruption follows its concurrent RPC probe");
    assert!(reply.unwrap().cycles.iter().any(|cycle| cycle.execution_count == 1));
    assert!(held, "the mutation reply stays held until interruption");
    assert_aborted(extension, &outcome);
    let outcome_text = outcome.text.clone();
    let cycles = super::identities(&outcome.text, "Cycle: ");
    assert_eq!(cycles.len(), 1, "{}", outcome.text);
    let cycle_id = TestCycleId::from_uuid(&cycles[0]).unwrap();
    let cycle = client.cycle(cycle_id);
    assert_eq!(cycle.snapshot().await.unwrap().phase, CyclePhase::Sealed);
    let members = cycle.executions().await.unwrap();
    assert_eq!(members.len(), 1, "interruption cannot retry append or Start");
    let execution = &members[0];
    let outcome = execution.wait(None, None).await.unwrap();
    let context = execution.context_at(outcome.checkpoint.as_ref().unwrap()).await.unwrap();
    let phase = context.cycle.unwrap().cycle.unwrap().phase;
    assert_eq!(outcome.document.unwrap().execution.unwrap().result.unwrap().status, ResultStatus::Cancelled);
    let execution_id = execution.id().to_string();
    let executions = vec![execution_id.clone()];
    let (execution_file, cycle_file) = recovered(extension, workflow, &directory, &cycles[0], &execution_id);
    recovery_saved(&execution_file, &outcome_text);
    let samples = super::archive(&execution_file, &cycles[0], &executions, phase, "cancelled", "obc.pulse");
    assert_eq!(samples[&execution_id], 0, "the independent execution keeps Start queued");
    super::archive(&cycle_file, &cycles[0], &executions, CyclePhase::Sealed, "cancelled", "obc.pulse");
}

async fn scenario(client: &Client, endpoint: &str, root: &Path, extension: &str, workflow: &str) {
    eprintln!("Remote Interruption: {extension} {workflow}");
    let directory = root.join(format!("{extension} {workflow}"));
    let command = super::configured_command(extension, workflow, endpoint, &directory, &[("EXECUTION_COUNT", "1")]).await;
    let command = command.as_std();
    let command = terminal_command(command, extension == "bat");
    // The CLI announces a wait once its interrupt handler is armed. A Ctrl+C sent
    // earlier can land between two commands or on one with no handler yet, so
    // nothing cancels the execution; Python handles it anywhere in its process.
    let armed = match (extension, workflow) {
        ("py", _) => "Execution: ",
        (_, "prompt") => "Waiting For Prompt: ",
        _ => "Waiting For Saved Outcome: ",
    };
    let selected = client.clone();
    let outcome = tokio::task::spawn_blocking(move || run(command, armed, Action::Interrupt(selected))).await.unwrap();
    assert!(outcome.triggered, "the script exited before the selected interrupt boundary:\n{}", outcome.text);
    assert_aborted(extension, &outcome);

    let cycles = super::identities(&outcome.text, "Cycle: ");
    let executions = super::identities(&outcome.text, "Execution: ");
    assert_eq!(cycles.len(), 1, "{}", outcome.text);
    assert_eq!(executions.len(), 1, "{}", outcome.text);
    let id = TestExecutionId::from_uuid(&executions[0]).unwrap();
    let execution = client.execution(id);
    let saved = execution.wait(None, None).await.unwrap();
    let context = execution.context_at(saved.checkpoint.as_ref().unwrap()).await.unwrap();
    let phase = context.cycle.unwrap().cycle.unwrap().phase;
    assert_eq!(saved.document.unwrap().execution.unwrap().result.unwrap().status, ResultStatus::Cancelled);

    let (execution_file, cycle_file) = recovered(extension, workflow, &directory, &cycles[0], &executions[0]);
    recovery_saved(&execution_file, &outcome.text);
    recovery_saved(&cycle_file, &outcome.text);
    let samples = super::archive(&execution_file, &cycles[0], &executions, phase, "cancelled", "obc.pulse");
    assert_eq!(samples[&executions[0]], 0, "the independent prompt kept this execution queued");
    super::archive(&cycle_file, &cycles[0], &executions, CyclePhase::Sealed, "cancelled", "obc.pulse");
    assert!(!outcome.text.contains("Drained Evidence:"), "a cancelled queued capture cannot claim final observation");
}

async fn disconnected(client: &Client, endpoint: &str, root: &Path, extension: &str, workflow: &str) {
    eprintln!("Remote Disconnection: {extension} {workflow}");
    let proxy = Proxy::start(endpoint).await;
    let directory = root.join(format!("disconnected {extension} {workflow}"));
    let command = super::configured_command(extension, workflow, &proxy.endpoint, &directory, &[("EXECUTION_COUNT", "1")]).await;
    let command = terminal_command(command.as_std(), extension == "bat");
    let (disconnect, ready) = oneshot::channel();
    let selected = client.clone();
    let process = tokio::task::spawn_blocking(move || run(command, "Execution: ", Action::Disconnect(selected, disconnect)));
    let triggered = ready.await;
    proxy.close().await;
    let outcome = process.await.unwrap();
    assert!(triggered.is_ok() && outcome.triggered, "the disconnect boundary was not reached:\n{}", outcome.text);
    assert_aborted(extension, &outcome);

    let cycles = super::identities(&outcome.text, "Cycle: ");
    let executions = super::identities(&outcome.text, "Execution: ");
    assert_eq!(cycles.len(), 1, "{}", outcome.text);
    assert_eq!(executions.len(), 1, "{}", outcome.text);
    let id = TestExecutionId::from_uuid(&executions[0]).unwrap();
    let execution = client.execution(id);
    let snapshot = execution.snapshot().await.unwrap();
    assert!(!snapshot.summary.unwrap().sealed, "a disconnected script must leave its admitted execution alive");
    let id = TestCycleId::from_uuid(&cycles[0]).unwrap();
    let cycle = client.cycle(id);
    assert_eq!(cycle.snapshot().await.unwrap().phase, CyclePhase::Active);
    let members = cycle.executions().await.unwrap();
    assert_eq!(members.len(), 1, "disconnect cannot retry admission");
    assert_eq!(members[0].id(), execution.id());
    assert!(!outcome.text.contains("Evidence Saved To"));
    assert!(!outcome.text.contains("Cycle Evidence:"));
    assert!(!directory.join("cycle.zip").exists());

    // Reconnection Uses The Retained Identities, Without Resubmission.
    cycle.seal().await.unwrap();
    let saved = execution.wait(None, None).await.unwrap();
    assert_eq!(saved.document.unwrap().execution.unwrap().result.unwrap().status, ResultStatus::Cancelled);
}

async fn partial_capture(client: &Client, endpoint: &str, root: &Path) {
    eprintln!("Remote Interruption: Partial Capture");
    let directory = root.join("partial Python capture");
    // The Operator Prompt Holds The Second Half Until This Terminal Is Interrupted.
    let command = super::configured_command(
        "py",
        "test",
        endpoint,
        &directory,
        &[
            ("TEMPLATE", "obc.charger.startup.sequence"),
            ("PARAMETERS", r#"{"duration_s":0.064,"sample_interval_s":0.001}"#),
            ("STREAMS", r#"["obc.charger.startup.samples"]"#),
            ("EXECUTION_COUNT", "1"),
        ],
    )
    .await;
    let command = terminal_command(command.as_std(), false);
    let selected = client.clone();
    let outcome = tokio::task::spawn_blocking(move || run(command, "Execution: ", Action::InterruptOnSample(selected))).await.unwrap();
    assert!(outcome.triggered && !outcome.success, "partial capture was not interrupted:\n{}", outcome.text);
    let cycles = super::identities(&outcome.text, "Cycle: ");
    let executions = super::identities(&outcome.text, "Execution: ");
    assert_eq!(cycles.len(), 1);
    assert_eq!(executions.len(), 1);
    let id = TestExecutionId::from_uuid(&executions[0]).unwrap();
    let execution = client.execution(id);
    let saved = execution.wait(None, None).await.unwrap();
    let context = execution.context_at(saved.checkpoint.as_ref().unwrap()).await.unwrap();
    let phase = context.cycle.unwrap().cycle.unwrap().phase;
    assert_eq!(saved.document.unwrap().execution.unwrap().result.unwrap().status, ResultStatus::Cancelled);

    let execution_file = directory.join("execution-1.zip");
    recovery_saved(&execution_file, &outcome.text);
    let samples = super::archive(&execution_file, &cycles[0], &executions, phase, "cancelled", "obc.charger.startup.samples");
    assert!(samples[&executions[0]] > 0 && samples[&executions[0]] < 64, "the saved bundle must retain a partial capture");
    let cycle_file = directory.join("cycle.zip");
    let cycle_samples =
        super::archive(&cycle_file, &cycles[0], &executions, CyclePhase::Sealed, "cancelled", "obc.charger.startup.samples");
    assert_eq!(samples, cycle_samples);
    assert!(!outcome.text.contains("Drained Evidence:"), "an interrupted listener cannot claim a completed final drain");
}

fn assert_aborted(extension: &str, outcome: &Outcome) {
    // cmd.exe Continues After The Native Command: Its Final Exit Is Not The
    // Operation Verdict. The Saved State And Evidence Checks Remain Required.
    if extension == "bat" {
        return;
    }
    assert!(!outcome.success, "the script must abort after interruption or disconnect:\n{}", outcome.text);
}

// The Script Output Explains A Recovery That Saved No Execution Evidence.
fn recovery_saved(file: &Path, output: &str) {
    assert!(file.is_file(), "the interrupted script saved no {}:\n{output}", file.display());
}

fn recovered(extension: &str, workflow: &str, directory: &Path, cycle: &str, execution: &str) -> (PathBuf, PathBuf) {
    let native = directory.join("recovery").join(format!("execution-{execution}.zip"));
    if extension == "sh" && !native.exists() {
        let filename = if workflow == "test" { "execution-1.zip" } else { "execution.zip" };
        return (directory.join(filename), directory.join("cycle.zip"));
    }
    if extension == "py" {
        let filename = if workflow == "test" { "execution-1.zip" } else { "execution.zip" };
        return (directory.join(filename), directory.join("cycle.zip"));
    }
    let root = directory.join("recovery");
    (root.join(format!("execution-{execution}.zip")), root.join(format!("cycle-{cycle}.zip")))
}

fn terminal_command(command: &Command, batch: bool) -> CommandBuilder {
    let program = Path::new(command.get_program());
    let mut terminal = if batch {
        let mut terminal = CommandBuilder::new("cmd.exe");
        terminal.args(["/D", "/C"]);
        // An Explicit Relative Path: cmd.exe Skips Its Current Directory When NoDefaultCurrentDirectoryInExePath Is Set.
        terminal.arg(Path::new(".").join(program.file_name().unwrap()));
        terminal.cwd(program.parent().unwrap());
        terminal
    } else {
        let mut terminal = CommandBuilder::new(command.get_program());
        terminal.cwd(command.get_current_dir().unwrap());
        terminal
    };
    // Preserve The Calling Task's Resolved Environment Instead Of Rebuilding It From The Registry.
    terminal.env_clear();
    for (name, value) in std::env::vars_os() {
        terminal.env(name, value);
    }
    terminal.args(command.get_args());
    for (name, value) in command.get_envs() {
        match value {
            Some(value) => terminal.env(name, value),
            None => terminal.env_remove(name),
        }
    }
    terminal
}

fn run(command: CommandBuilder, marker: &str, action: Action) -> Outcome {
    let runtime = tokio::runtime::Handle::current();
    let size = PtySize {
        rows: 40,
        cols: 240,
        pixel_width: 0,
        pixel_height: 0,
    };
    let pair = native_pty_system().openpty(size).unwrap();
    // An Operator's Terminal Starts Its Shell With Ctrl+C Enabled. A Runner Started In Another
    // Process Group Ignores Ctrl+C, And The Shell Would Inherit That Through The Pseudo-Terminal.
    // SAFETY: A Null Handler Only Clears This Process's Ignore-Ctrl+C Attribute.
    #[cfg(windows)]
    assert_ne!(unsafe { windows_sys::Win32::System::Console::SetConsoleCtrlHandler(None, 0) }, 0, "Ctrl+C processing must be restorable");
    let child = pair.slave.spawn_command(command).unwrap();
    let mut process = Process { child, reaped: false };
    eprintln!("Remote Terminal Started: {:?}", process.child.process_id());
    drop(pair.slave);

    let reader = pair.master.try_clone_reader().unwrap();
    let writer = pair.master.take_writer().unwrap();
    let marker = marker.to_owned();
    let transcript = Arc::new(Mutex::new(Vec::new()));
    let captured = transcript.clone();
    let mut killer = process.child.clone_killer();
    let root = process.child.process_id();
    let read = std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            read_terminal(reader, writer, &marker, action, runtime, captured.clone(), root)
        }))
        .unwrap_or_else(|failure| {
            let message = failure
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| failure.downcast_ref::<&str>().copied())
                .unwrap_or("unknown terminal reader panic");
            let output = terminal_output(&captured.lock().unwrap());
            Err(std::io::Error::other(format!("{message}\nTerminal Transcript:\n{}", output.text)))
        });
        if result.is_err() {
            if let Some(root) = root {
                kill_tree(root);
            }
            let _ = killer.kill();
        }
        result
    });
    // A Stuck Script Fails With Its Terminal Text Instead Of Hanging The Lane.
    let (finished, exited) = std::sync::mpsc::channel::<()>();
    let watchdog = std::thread::spawn(move || {
        let expired = exited.recv_timeout(SCRIPT_DEADLINE).is_err();
        if expired && let Some(root) = root {
            kill_tree(root);
        }
        expired
    });
    let status = process.child.wait().unwrap();
    process.reaped = true;
    let _ = finished.send(());
    let expired = watchdog.join().unwrap();
    drop(pair.master);

    // Surviving Descendants Keep The Terminal Open, So The Reader Join Is Bounded.
    let (delivered, result) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = delivered.send(read.join());
    });
    let joined = result.recv_timeout(std::time::Duration::from_secs(30));
    let (text, triggered) = match joined {
        Ok(joined) => joined.unwrap().unwrap(),
        Err(_) => {
            let transcript = transcript.lock().unwrap();
            let output = terminal_output(&transcript);
            panic!("the terminal reader did not finish after script exit; watchdog expired: {expired}\n{}", output.text);
        }
    };
    assert!(
        !expired,
        "the remote script exceeded {SCRIPT_DEADLINE:?} and its process tree was killed:
{text}"
    );
    Outcome {
        success: status.success(),
        triggered,
        text,
    }
}

// Kill The Script And Every Descendant, Such As Waiting CLI Processes.
fn kill_tree(root: u32) {
    let root = root.to_string();
    #[cfg(windows)]
    let _ = dokime::domain::process::command("taskkill").args(["/T", "/F", "/PID", &root]).output();
    // The pseudo terminal makes the script a session and group leader.
    #[cfg(not(windows))]
    let _ = dokime::domain::process::command("kill").args(["-KILL", "--", &format!("-{root}")]).output();
}

fn read_terminal(
    mut reader: Box<dyn Read + Send>, writer: Box<dyn Write + Send>, marker: &str, action: Action, runtime: tokio::runtime::Handle,
    transcript: Arc<Mutex<Vec<u8>>>, process: Option<u32>,
) -> std::io::Result<(String, bool)> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 4096];
    let mut cursors = 0;
    let input = Arc::new(Mutex::new(TerminalInput {
        writer,
        native: false,
        triggered: false,
        batch_prompt: false,
    }));
    let mut control = None;
    let mut batch = None;
    let mut action = Some(action);
    let gate = match &action {
        Some(Action::InterruptAt { released, .. }) => Some(released.clone()),
        _ => None,
    };
    let mut diagnostics = std::env::var_os("DOKIME_TEST_TRACE_DIR")
        .map(|root| {
            let name = format!("terminal-{}-{}.log", std::process::id(), uuid::Uuid::now_v7());
            std::fs::File::create(PathBuf::from(root).join(name))
        })
        .transpose()?;
    loop {
        let count = match reader.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(count) => count,
        };
        bytes.extend_from_slice(&buffer[..count]);
        transcript.lock().unwrap().extend_from_slice(&buffer[..count]);
        if let Some(diagnostics) = diagnostics.as_mut() {
            diagnostics.write_all(&buffer[..count])?;
            diagnostics.flush()?;
        }
        if bytes.len() > 4 * 1024 * 1024 {
            return Err(std::io::Error::other("remote script terminal output exceeded its budget"));
        }
        let queries = bytes.windows(4).filter(|window| window == b"\x1b[6n").count();
        while cursors < queries {
            let mut input = input.lock().unwrap();
            input.writer.write_all(b"\x1b[1;1R")?;
            input.writer.flush()?;
            cursors += 1;
        }
        let output = terminal_output(&bytes);
        {
            let mut input = input.lock().unwrap();
            input.native = output.native_input;
            input.batch_prompt = output.batch_prompt;
        }
        let text = &output.text;
        let execution = output.execution;
        let selected = execution.is_some() || gate.is_some();
        if action.is_some() && text.contains(marker) && selected {
            eprintln!("Remote Terminal Reached: {marker}; execution: {execution:?}; native input: {}", output.native_input);
            let action = action.take().unwrap();
            let input = input.clone();
            // Readiness Can Require More Terminal Queries Before The Script Can Make Progress.
            control = Some(runtime.spawn(async move {
                let result = async {
                    let disconnect = match action {
                        Action::Interrupt(client) => {
                            admitted(&client, execution.unwrap()).await?;
                            None
                        }
                        Action::InterruptOnSample(client) => {
                            let id = execution.unwrap();
                            admitted(&client, id.clone()).await?;
                            first_sample(&client, id).await?;
                            None
                        }
                        Action::Disconnect(client, disconnect) => {
                            admitted(&client, execution.unwrap()).await?;
                            Some(disconnect)
                        }
                        Action::InterruptAt { reached, .. } => {
                            tokio::time::timeout(std::time::Duration::from_secs(120), reached.cancelled())
                                .await
                                .map_err(|_| std::io::Error::other("mutation reply gate was not reached"))?;
                            None
                        }
                    };
                    let mut input = input.lock().unwrap();
                    input.triggered = true;
                    if let Some(disconnect) = disconnect {
                        let _ = disconnect.send(());
                    } else {
                        let native = input.native;
                        terminal_key(&mut input.writer, native, TerminalKey::Interrupt)?;
                    }
                    eprintln!("Remote Terminal Action Delivered");
                    Ok::<_, std::io::Error>(())
                }
                .await;
                if result.is_err()
                    && let Some(process) = process
                {
                    kill_tree(process);
                }
                result
            }));
        }
        let cancellation_requested = input.lock().unwrap().triggered && output.cancellation_requested;
        if cancellation_requested && let Some(gate) = &gate {
            gate.cancel();
        }
        if batch.is_none() && output.batch_prompt {
            let input = input.clone();
            batch = Some(runtime.spawn(async move {
                // A Written Reply Is Not An Acknowledgement; Keep Reading And Retry Until It Is Echoed.
                let mut retry = tokio::time::interval(std::time::Duration::from_millis(100));
                retry.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                loop {
                    retry.tick().await;
                    let mut input = input.lock().unwrap();
                    if !input.batch_prompt {
                        eprintln!("Batch Termination Prompt Acknowledged");
                        return Ok::<_, std::io::Error>(());
                    }
                    let native = input.native;
                    terminal_key(&mut input.writer, native, TerminalKey::ContinueBatch)?;
                }
            }));
        }
    }
    let triggered = input.lock().unwrap().triggered;
    if let Some(control) = control {
        if triggered || control.is_finished() {
            runtime.block_on(control).map_err(|error| std::io::Error::other(format!("terminal action failed: {error:?}")))??;
        } else {
            control.abort();
        }
    }
    if let Some(batch) = batch {
        if batch.is_finished() {
            runtime.block_on(batch).map_err(|error| std::io::Error::other(format!("batch reply failed: {error:?}")))??;
        } else {
            batch.abort();
        }
    }
    Ok((terminal_output(&bytes).text, triggered))
}

fn terminal_key(writer: &mut dyn Write, native_input: bool, key: TerminalKey) -> std::io::Result<()> {
    let native = cfg!(windows) && native_input;
    let bytes: &[u8] = match (key, native) {
        (TerminalKey::Interrupt, false) => b"\x03",
        (TerminalKey::ContinueBatch, false) => b"N\r",
        // Native Key Events Carry Virtual Key, Scan Code, Character, Down/Up, Modifiers And Repeat Count.
        (TerminalKey::Interrupt, true) => b"\x1b[17;29;0;1;8;1_\x1b[67;46;3;1;8;1_\x1b[67;46;3;0;8;1_\x1b[17;29;0;0;0;1_",
        (TerminalKey::ContinueBatch, true) => b"\x1b[78;49;110;1;0;1_\x1b[78;49;110;0;0;1_\x1b[13;28;13;1;0;1_\x1b[13;28;13;0;0;1_",
    };
    writer.write_all(bytes)?;
    writer.flush()
}

fn terminal_output(bytes: &[u8]) -> TerminalOutput {
    let raw = String::from_utf8_lossy(bytes);
    let plain = anstream::adapter::strip_str(&raw).to_string();
    let text = plain.replace("\r\n", "\n");
    let enabled = raw.rfind("\x1b[?9001h");
    let disabled = raw.rfind("\x1b[?9001l");
    let execution = text
        .lines()
        .filter_map(|line| line.strip_prefix("Execution: "))
        .filter_map(|line| line.split_whitespace().next())
        .find_map(|id| TestExecutionId::from_uuid(id).ok());
    TerminalOutput {
        execution,
        native_input: enabled.is_some_and(|enabled| disabled.is_none_or(|disabled| enabled > disabled)),
        cancellation_requested: text.contains("Cancellation Requested"),
        batch_prompt: text.rsplit_once("Terminate batch job (Y/N)?").is_some_and(|(_, response)| response.trim().is_empty()),
        text,
    }
}

async fn first_sample(client: &Client, id: TestExecutionId) -> std::io::Result<()> {
    // The Tutorial Prints Only After Draining; Observe Live Evidence Before Interrupting It.
    let execution = client.execution(id);
    let samples = execution
        .samples(vec!["obc.charger.startup.samples".into()], true, 64)
        .map_err(|error| std::io::Error::other(format!("sample listener failed: {error:?}")))?;
    let waiting = async {
        while let Some(page) = samples.message().await.map_err(|error| std::io::Error::other(format!("sample read failed: {error:?}")))? {
            if !page.samples.is_empty() {
                return Ok(());
            }
        }
        Err(std::io::Error::other("the execution must publish a sample before finishing"))
    };
    let result = tokio::time::timeout(std::time::Duration::from_secs(120), waiting)
        .await
        .map_err(|_| std::io::Error::other("the execution must begin its capture"));
    samples.close().await;
    result?
}

async fn admitted(client: &Client, id: TestExecutionId) -> std::io::Result<()> {
    let changes = client
        .stream(dokime::client::contracts::observation::WatchRuntime::default())
        .await
        .map_err(|error| std::io::Error::other(format!("admission observation failed: {error:?}")))?;
    let execution = client.execution(id);
    let waiting = async {
        loop {
            // Python Prints Constructed Identities Before Append Is Confirmed.
            let request_id = match execution.snapshot().await {
                Ok(snapshot) => snapshot.controls.and_then(|controls| controls.request_id),
                Err(dokime::client::Error::Rpc(status)) if status.code() == dokime::client::RpcCode::NotFound => None,
                Err(error) => return Err(std::io::Error::other(format!("execution admission probe failed: {error:?}"))),
            };
            if request_id.is_some() {
                return Ok(());
            }
            changes
                .message()
                .await
                .map_err(|error| std::io::Error::other(format!("admission observation failed: {error:?}")))?
                .ok_or_else(|| std::io::Error::other("runtime observation closed before admission"))?;
        }
    };
    let result = tokio::time::timeout(std::time::Duration::from_secs(120), waiting)
        .await
        .map_err(|_| std::io::Error::other("script must admit its printed execution"));
    changes.close().await;
    result?
}

/* === Tests === */

#[tokio::test]
async fn terminal_reader_services_queries_before_action_readiness() {
    struct Reader {
        stage: usize,
        interrupted: std::sync::mpsc::Receiver<()>,
    }
    impl Read for Reader {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            let bytes: &[u8] = match self.stage {
                0 => b"Cycle: fixture\r\n",
                1 => b"\x1b[6n",
                2 => {
                    self.interrupted.recv().map_err(std::io::Error::other)?;
                    b"Cancellation Requested\r\n"
                }
                _ => return Ok(0),
            };
            buffer[..bytes.len()].copy_from_slice(bytes);
            self.stage += 1;
            Ok(bytes.len())
        }
    }
    struct Writer {
        cursor: Option<oneshot::Sender<()>>,
        interrupted: std::sync::mpsc::Sender<()>,
    }
    impl Write for Writer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes == b"\x1b[1;1R" {
                if let Some(cursor) = self.cursor.take() {
                    let _ = cursor.send(());
                }
            } else if bytes == b"\x03" {
                self.interrupted.send(()).map_err(std::io::Error::other)?;
            }
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let (cursor, response) = oneshot::channel();
    let (interrupt, interrupted) = std::sync::mpsc::channel();
    let reader = Reader { stage: 0, interrupted };
    let writer = Writer {
        cursor: Some(cursor),
        interrupted: interrupt,
    };
    let reached = CancellationToken::new();
    let released = CancellationToken::new();
    let action = Action::InterruptAt {
        reached: reached.clone(),
        released: released.clone(),
    };
    let runtime = tokio::runtime::Handle::current();
    let reading = std::thread::spawn(move || {
        read_terminal(Box::new(reader), Box::new(writer), "Cycle: ", action, runtime, Arc::new(Mutex::new(Vec::new())), None)
    });

    // The Producer Cannot Reach Its Mutation Boundary Until Its Cursor Query Is Answered.
    let serviced = tokio::time::timeout(std::time::Duration::from_secs(3), response).await;
    reached.cancel();
    let outcome = tokio::task::spawn_blocking(move || reading.join()).await.unwrap().unwrap().unwrap();
    assert!(matches!(serviced, Ok(Ok(()))), "terminal queries must progress before action readiness");
    assert!(outcome.1);
    assert!(released.is_cancelled());
}

#[test]
fn terminal_protocol_preserves_identity_and_input_negotiation() {
    let id = "1b7847e4-3351-4ea9-9f77-59c549da8de9";
    let mut bytes = b"\x1b[?9001h\x1b[32mExecution: \x1b[0m".to_vec();
    bytes.extend_from_slice(&id.as_bytes()[..12]);
    assert!(terminal_output(&bytes).execution.is_none());
    bytes.extend_from_slice(&id.as_bytes()[12..]);
    bytes.extend_from_slice(b"\r\n\x1b[33mCancellation Requested\x1b[0m\r\nTerminate batch job (Y/N)?");
    let output = terminal_output(&bytes);
    assert_eq!(output.execution, Some(TestExecutionId::from_uuid(id).unwrap()));
    assert!(output.native_input);
    assert!(output.cancellation_requested);
    assert!(output.batch_prompt);
    bytes.extend_from_slice(b"\x1b[?9001l");
    assert!(!terminal_output(&bytes).native_input);
    bytes.extend_from_slice(b"n\r\n");
    assert!(!terminal_output(&bytes).batch_prompt);
}

#[tokio::test]
async fn batch_reply_recovery() {
    struct Reader {
        stage: usize,
        acknowledged: std::sync::mpsc::Receiver<()>,
    }
    impl Read for Reader {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            let bytes: &[u8] = match self.stage {
                0 => b"Terminate batch job (Y/N)? ",
                1 => {
                    self.acknowledged.recv_timeout(std::time::Duration::from_secs(3)).map_err(std::io::Error::other)?;
                    b"n\r\n"
                }
                _ => return Ok(0),
            };
            buffer[..bytes.len()].copy_from_slice(bytes);
            self.stage += 1;
            Ok(bytes.len())
        }
    }
    struct Writer {
        replies: Arc<std::sync::atomic::AtomicUsize>,
        acknowledged: std::sync::mpsc::Sender<()>,
    }
    impl Write for Writer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes == b"N\r" && self.replies.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 1 {
                self.acknowledged.send(()).map_err(std::io::Error::other)?;
            }
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let (acknowledged, received) = std::sync::mpsc::channel();
    let replies = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let reader = Reader { stage: 0, acknowledged: received };
    let writer = Writer { replies: replies.clone(), acknowledged };
    let action = Action::InterruptAt {
        reached: CancellationToken::new(),
        released: CancellationToken::new(),
    };
    let runtime = tokio::runtime::Handle::current();
    let reading = std::thread::spawn(move || {
        read_terminal(Box::new(reader), Box::new(writer), "Cycle: ", action, runtime, Arc::new(Mutex::new(Vec::new())), None)
    });
    let outcome = tokio::task::spawn_blocking(move || reading.join()).await.unwrap().unwrap().unwrap();
    assert!(!outcome.1);
    assert!(replies.load(std::sync::atomic::Ordering::SeqCst) >= 2, "a dropped reply must be delivered again before terminal EOF");
}
