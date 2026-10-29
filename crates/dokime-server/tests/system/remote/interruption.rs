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
};

// Third Party Imports
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
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
    execution::{CreateCycle, ExecutionBinding},
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
    InterruptAt(Gate),
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
        let reply = self.client.execution_list_cycles(request).await.map_err(|error| std::io::Error::other(error.to_string()))?;
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

async fn relay(mut incoming: TcpStream, mut outgoing: TcpStream, gate: Option<Gate>) -> std::io::Result<()> {
    let Some(gate) = gate else {
        tokio::io::copy_bidirectional(&mut incoming, &mut outgoing).await?;
        return Ok(());
    };
    let (mut inbound, mut output) = incoming.into_split();
    let (mut response, mut outbound) = outgoing.into_split();
    let requests = tokio::io::copy(&mut inbound, &mut outbound);
    let responses = async {
        let mut buffer = [0_u8; 65_536];
        loop {
            let count = response.read(&mut buffer).await?;
            if count == 0 {
                break;
            }
            gate.hold().await?;
            output.write_all(&buffer[..count]).await?;
        }
        output.shutdown().await
    };
    tokio::try_join!(requests, responses)?;
    Ok(())
}

/* === Public Functions === */

pub async fn scripts(client: &Client, endpoint: &str, root: &Path) {
    std::fs::create_dir_all(root).unwrap();
    // Each Interruption Story Is Boxed So Only The Running One's Await Temporaries Occupy The Stack.
    let (other_cycle, other_execution) = Box::pin(waiting_execution(client)).await;
    let extensions: &[&str] = if cfg!(windows) { &["ps1", "bat", "py"] } else { &["sh", "py"] };
    for extension in extensions {
        Box::pin(scenario(client, endpoint, root, extension, "test")).await;
        Box::pin(scenario(client, endpoint, root, extension, "prompt")).await;
        Box::pin(disconnected(client, endpoint, root, extension, "test")).await;
        Box::pin(disconnected(client, endpoint, root, extension, "prompt")).await;
        if *extension != "py" {
            Box::pin(mutation_interruption(client, endpoint, root, extension, Boundary::Append)).await;
            Box::pin(mutation_interruption(client, endpoint, root, extension, Boundary::Start)).await;
        }
        let summary = other_execution.snapshot().await.unwrap();
        assert!(!summary.summary.unwrap().sealed, "another client's execution must survive every script interrupt");
        assert_eq!(other_cycle.snapshot().await.unwrap().phase, CyclePhase::Active);
    }
    other_cycle.seal().await.unwrap();
    let saved = other_execution.wait(None, None).await.unwrap();
    assert_eq!(saved.document.unwrap().execution.unwrap().result.unwrap().status, ResultStatus::Cancelled);
    Box::pin(partial_capture(client, endpoint, root)).await;
}

/* === Private Functions === */

async fn waiting_execution(client: &Client) -> (TestCycleHandle, TestExecutionHandle) {
    let changes = client.observation_watch_runtime(Default::default()).await.unwrap();
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
    let command = super::configured_command(extension, workflow, &proxy.endpoint, &directory, &parameters);
    let command = terminal_command(command.as_std(), extension == "bat");

    // Ctrl+C Arrives Before Confirmation; Cleanup Must Reconcile Exact Membership.
    let process = tokio::task::spawn_blocking(move || run(command, "Cycle: ", Action::InterruptAt(gate)));
    let outcome = process.await.unwrap();
    proxy.close().await;
    assert!(outcome.triggered, "{name} was not interrupted:\n{}", outcome.text);
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
    let samples = super::archive(&execution_file, &cycles[0], &executions, phase, "cancelled");
    assert_eq!(samples[&execution_id], 0, "the independent execution keeps Start queued");
    super::archive(&cycle_file, &cycles[0], &executions, CyclePhase::Sealed, "cancelled");
}

async fn scenario(client: &Client, endpoint: &str, root: &Path, extension: &str, workflow: &str) {
    let directory = root.join(format!("{extension} {workflow}"));
    let command = super::configured_command(extension, workflow, endpoint, &directory, &[("EXECUTION_COUNT", "1")]);
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
    let samples = super::archive(&execution_file, &cycles[0], &executions, phase, "cancelled");
    assert_eq!(samples[&executions[0]], 0, "the independent prompt kept this execution queued");
    super::archive(&cycle_file, &cycles[0], &executions, CyclePhase::Sealed, "cancelled");
    assert!(!outcome.text.contains("Drained Evidence:"), "a cancelled queued capture cannot claim final observation");
}

async fn disconnected(client: &Client, endpoint: &str, root: &Path, extension: &str, workflow: &str) {
    let proxy = Proxy::start(endpoint).await;
    let directory = root.join(format!("disconnected {extension} {workflow}"));
    let command = super::configured_command(extension, workflow, &proxy.endpoint, &directory, &[("EXECUTION_COUNT", "1")]);
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
    let directory = root.join("partial Python capture");
    let command = super::configured_command("py", "test", endpoint, &directory, &[("SAMPLE_COUNT", "64"), ("EXECUTION_COUNT", "1")]);
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
    let samples = super::archive(&execution_file, &cycles[0], &executions, phase, "cancelled");
    assert!(samples[&executions[0]] > 0 && samples[&executions[0]] < 64, "the saved bundle must retain a partial capture");
    let cycle_file = directory.join("cycle.zip");
    let cycle_samples = super::archive(&cycle_file, &cycles[0], &executions, CyclePhase::Sealed, "cancelled");
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
    drop(pair.slave);

    let reader = pair.master.try_clone_reader().unwrap();
    let writer = pair.master.take_writer().unwrap();
    let marker = marker.to_owned();
    let mut killer = process.child.clone_killer();
    let read = std::thread::spawn(move || {
        let result = read_terminal(reader, writer, &marker, action, runtime);
        if result.is_err() {
            let _ = killer.kill();
        }
        result
    });
    // A Stuck Script Fails With Its Terminal Text Instead Of Hanging The Lane.
    let (finished, exited) = std::sync::mpsc::channel::<()>();
    let root = process.child.process_id();
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
        Err(_) => panic!("the remote script exceeded {SCRIPT_DEADLINE:?} and its terminal stayed open after the process tree was killed"),
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
    let _ = Command::new("taskkill").args(["/T", "/F", "/PID", &root]).output();
    // The pseudo terminal makes the script a session and group leader.
    #[cfg(not(windows))]
    let _ = Command::new("kill").args(["-KILL", "--", &format!("-{root}")]).output();
}

fn read_terminal(
    mut reader: Box<dyn Read + Send>, mut writer: Box<dyn Write + Send>, marker: &str, action: Action, runtime: tokio::runtime::Handle,
) -> std::io::Result<(String, bool)> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 4096];
    let mut cursors = 0;
    let mut action = Some(action);
    let gate = match &action {
        Some(Action::InterruptAt(gate)) => Some(gate.clone()),
        _ => None,
    };
    let mut triggered = false;
    let mut continued = false;
    loop {
        let count = match reader.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(count) => count,
        };
        bytes.extend_from_slice(&buffer[..count]);
        if bytes.len() > 4 * 1024 * 1024 {
            return Err(std::io::Error::other("remote script terminal output exceeded its budget"));
        }
        let queries = bytes.windows(4).filter(|window| window == b"\x1b[6n").count();
        while cursors < queries {
            writer.write_all(b"\x1b[1;1R")?;
            writer.flush()?;
            cursors += 1;
        }
        let text = String::from_utf8_lossy(&bytes);
        let execution = text
            .lines()
            .filter_map(|line| line.strip_prefix("Execution: "))
            .filter_map(|line| line.split_whitespace().next())
            .find_map(|id| TestExecutionId::from_uuid(id).ok());
        let selected = execution.is_some() || gate.is_some();
        if !triggered && text.contains(marker) && selected {
            match action.take().unwrap() {
                Action::Interrupt(client) => {
                    runtime.block_on(admitted(&client, execution.unwrap()));
                    writer.write_all(b"\x03")?;
                    writer.flush()?;
                }
                Action::InterruptOnSample(client) => {
                    let id = execution.unwrap();
                    runtime.block_on(admitted(&client, id.clone()));
                    runtime.block_on(first_sample(&client, id));
                    writer.write_all(b"\x03")?;
                    writer.flush()?;
                }
                Action::Disconnect(client, disconnect) => {
                    runtime.block_on(admitted(&client, execution.unwrap()));
                    let _ = disconnect.send(());
                }
                Action::InterruptAt(gate) => {
                    let waiting = async { tokio::time::timeout(std::time::Duration::from_secs(120), gate.reached.cancelled()).await };
                    runtime.block_on(waiting).map_err(|_| std::io::Error::other("mutation reply gate was not reached"))?;
                    writer.write_all(b"\x03")?;
                    writer.flush()?;
                }
            }
            triggered = true;
        }
        let cancellation_requested = triggered && text.contains("Cancellation Requested");
        if cancellation_requested && let Some(gate) = &gate {
            gate.released.cancel();
        }
        if !continued && text.contains("Terminate batch job (Y/N)? ") {
            writer.write_all(b"N\r")?;
            writer.flush()?;
            eprintln!("Batch Termination Prompt Acknowledged");
            continued = true;
        }
    }
    let text = String::from_utf8_lossy(&bytes);
    let plain = anstream::adapter::strip_str(&text).to_string();
    let text = plain.replace("\r\n", "\n");
    Ok((text, triggered))
}

async fn first_sample(client: &Client, id: TestExecutionId) {
    // The Tutorial Prints Only After Draining; Observe Live Evidence Before Interrupting It.
    let execution = client.execution(id);
    let samples = execution.samples(vec!["obc.pulse".into()], true, 64).unwrap();
    let waiting = async {
        while let Some(page) = samples.message().await.unwrap() {
            if !page.samples.is_empty() {
                return;
            }
        }
        panic!("the execution must publish a sample before finishing");
    };
    tokio::time::timeout(std::time::Duration::from_secs(120), waiting).await.expect("the execution must begin its capture");
    samples.close().await;
}

async fn admitted(client: &Client, id: TestExecutionId) {
    let changes = client.observation_watch_runtime(Default::default()).await.unwrap();
    let execution = client.execution(id);
    let waiting = async {
        loop {
            // Python Prints Constructed Identities Before Append Is Confirmed.
            let request_id = match execution.snapshot().await {
                Ok(snapshot) => snapshot.controls.and_then(|controls| controls.request_id),
                Err(dokime::client::Error::Rpc(status)) if status.code() == dokime::client::RpcCode::NotFound => None,
                Err(error) => panic!("execution admission probe failed: {error}"),
            };
            if request_id.is_some() {
                break;
            }
            changes.message().await.unwrap().expect("runtime observation remains open until admission");
        }
    };
    tokio::time::timeout(std::time::Duration::from_secs(120), waiting).await.expect("script must admit its printed execution");
    changes.close().await;
}

/* === Tests === */
