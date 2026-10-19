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
    collections::BTreeMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

// Third Party Imports
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::oneshot,
    task::{JoinHandle, JoinSet},
};
use tokio_util::sync::CancellationToken;

// Local Imports
use dokime_client::{Client, TestCycleHandle, TestExecutionHandle};
use dokime_domain::model::{
    execution::{ExecutionMode, result::CyclePhase, wire::ResultStatus},
    identity::{TestCycleId, TestExecutionId},
    values::Value,
};

/* === Constants === */

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

/* === Enums === */
enum Action {
    Interrupt,
    Disconnect(oneshot::Sender<()>),
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
                        let (mut incoming, _) = connection.unwrap();
                        let target = target.clone();
                        let cancelled = stopped.clone();
                        connections.spawn(async move {
                            tokio::select! {
                                biased;
                                _ = cancelled.cancelled() => {},
                                _ = async {
                                    let mut outgoing = TcpStream::connect(target).await?;
                                    tokio::io::copy_bidirectional(&mut incoming, &mut outgoing).await?;
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

/* === Public Functions === */
pub async fn scripts(client: &Client, endpoint: &str, root: &Path) {
    std::fs::create_dir_all(root).unwrap();
    let (other_cycle, other_execution) = waiting_execution(client).await;
    let extensions: &[&str] = if cfg!(windows) { &["ps1", "bat", "py"] } else { &["sh", "py"] };
    for extension in extensions {
        scenario(client, endpoint, root, extension, "test").await;
        scenario(client, endpoint, root, extension, "prompt").await;
        disconnected(client, endpoint, root, extension, "test").await;
        disconnected(client, endpoint, root, extension, "prompt").await;
        let summary = other_execution.snapshot().await.unwrap();
        assert!(!summary.summary.unwrap().sealed, "another client's execution must survive every script interrupt");
        assert_eq!(other_cycle.snapshot().await.unwrap().phase, CyclePhase::Active);
    }
    other_cycle.seal().await.unwrap();
    let saved = other_execution.wait(None, None).await.unwrap();
    assert_eq!(saved.document.unwrap().execution.unwrap().result.unwrap().status, ResultStatus::Cancelled);
    partial_capture(client, endpoint, root).await;
}

/* === Private Functions === */
async fn waiting_execution(client: &Client) -> (TestCycleHandle, TestExecutionHandle) {
    let changes = client.observation_watch_runtime(Default::default()).await.unwrap();
    let parameters = BTreeMap::from([("require_confirmation".to_owned(), Value::Bool(true))]);
    let cycle = client.plan_cycle("obc.checks", "obc.checks.pulse", parameters, Some("Independent Waiting Operator".into())).await.unwrap();
    let mut members = cycle.executions().await.unwrap();
    let execution = members.pop().unwrap();
    execution.start("independent-waiting-operator", ExecutionMode::Automatic).await.unwrap();

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

async fn scenario(client: &Client, endpoint: &str, root: &Path, extension: &str, workflow: &str) {
    let directory = root.join(format!("{extension} {workflow}"));
    let command = super::command(extension, workflow, endpoint, &directory);
    let command = command.as_std();
    let command = terminal_command(command, extension == "bat");
    let marker = if extension == "py" && workflow == "test" {
        "Current Status:"
    } else if extension == "py" {
        "Execution:"
    } else {
        "Waiting For "
    };
    let outcome = tokio::task::spawn_blocking(move || run(command, marker, Action::Interrupt)).await.unwrap();
    assert!(outcome.triggered, "the script exited before the selected interrupt boundary:\n{}", outcome.text);
    assert!(!outcome.success, "interruption must return nonzero:\n{}", outcome.text);

    let cycles = super::identities(&outcome.text, "Cycle: ");
    let executions = super::identities(&outcome.text, "Execution: ");
    assert_eq!(cycles.len(), 1, "{}", outcome.text);
    assert_eq!(executions.len(), 1, "{}", outcome.text);
    let id = TestExecutionId::from_uuid(&executions[0]).unwrap();
    let execution = client.execution(id);
    let saved = execution.wait(None, None).await.unwrap();
    assert_eq!(saved.document.unwrap().execution.unwrap().result.unwrap().status, ResultStatus::Cancelled);

    let (execution_file, cycle_file) = recovered(extension, workflow, &directory, &cycles[0], &executions[0]);
    let samples = super::archive(&execution_file, &cycles[0], &executions, true, "cancelled");
    assert_eq!(samples[&executions[0]], 0, "the independent prompt kept this execution queued");
    super::archive(&cycle_file, &cycles[0], &executions, true, "cancelled");
    assert!(!outcome.text.contains("Drained Evidence:"), "a cancelled queued capture cannot claim final observation");
}

async fn disconnected(client: &Client, endpoint: &str, root: &Path, extension: &str, workflow: &str) {
    let proxy = Proxy::start(endpoint).await;
    let directory = root.join(format!("disconnected {extension} {workflow}"));
    let command = super::command(extension, workflow, &proxy.endpoint, &directory);
    let command = terminal_command(command.as_std(), extension == "bat");
    let marker = if extension == "py" && workflow == "test" {
        "Current Status:"
    } else if extension == "py" {
        "Request Accepted For Execution "
    } else {
        "Waiting For "
    };
    let (disconnect, ready) = oneshot::channel();
    let process = tokio::task::spawn_blocking(move || run(command, marker, Action::Disconnect(disconnect)));
    let triggered = ready.await;
    proxy.close().await;
    let outcome = process.await.unwrap();
    assert!(triggered.is_ok() && outcome.triggered, "the disconnect boundary was not reached:\n{}", outcome.text);
    assert!(!outcome.success, "a lost connection cannot claim completion:\n{}", outcome.text);

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
    let mut command = super::command("py", "test", endpoint, &directory);
    command.args(["--sample-count", "64"]);
    let command = terminal_command(command.as_std(), false);
    let outcome = tokio::task::spawn_blocking(move || run(command, "Sample: obc.pulse #", Action::Interrupt)).await.unwrap();
    assert!(outcome.triggered && !outcome.success, "partial capture was not interrupted:\n{}", outcome.text);
    let cycles = super::identities(&outcome.text, "Cycle: ");
    let executions = super::identities(&outcome.text, "Execution: ");
    assert_eq!(cycles.len(), 1);
    assert_eq!(executions.len(), 1);
    let id = TestExecutionId::from_uuid(&executions[0]).unwrap();
    let saved = client.execution(id).wait(None, None).await.unwrap();
    assert_eq!(saved.document.unwrap().execution.unwrap().result.unwrap().status, ResultStatus::Cancelled);

    let execution_file = directory.join("execution-1.zip");
    let samples = super::archive(&execution_file, &cycles[0], &executions, true, "cancelled");
    assert!(samples[&executions[0]] > 0 && samples[&executions[0]] < 64, "the saved bundle must retain a partial capture");
    let cycle_file = directory.join("cycle.zip");
    let cycle_samples = super::archive(&cycle_file, &cycles[0], &executions, true, "cancelled");
    assert_eq!(samples, cycle_samples);
    assert!(!outcome.text.contains("Drained Evidence:"), "an interrupted listener cannot claim a completed final drain");
}

fn recovered(extension: &str, workflow: &str, directory: &Path, cycle: &str, execution: &str) -> (PathBuf, PathBuf) {
    if extension == "sh" {
        return (directory.join("interrupted-execution.zip"), directory.join("interrupted-cycle.zip"));
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
        terminal.arg(program.file_name().unwrap());
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
    let size = PtySize { rows: 40, cols: 240, pixel_width: 0, pixel_height: 0 };
    let pair = native_pty_system().openpty(size).unwrap();
    let child = pair.slave.spawn_command(command).unwrap();
    let mut process = Process { child, reaped: false };
    drop(pair.slave);

    let reader = pair.master.try_clone_reader().unwrap();
    let writer = pair.master.take_writer().unwrap();
    let marker = marker.to_owned();
    let mut killer = process.child.clone_killer();
    let read = std::thread::spawn(move || {
        let result = read_terminal(reader, writer, &marker, action);
        if result.is_err() {
            let _ = killer.kill();
        }
        result
    });
    let status = process.child.wait().unwrap();
    process.reaped = true;
    drop(pair.master);

    let (text, triggered) = read.join().unwrap().unwrap();
    Outcome { success: status.success(), triggered, text }
}

fn read_terminal(
    mut reader: Box<dyn Read + Send>, mut writer: Box<dyn Write + Send>, marker: &str, action: Action,
) -> std::io::Result<(String, bool)> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 4096];
    let mut cursors = 0;
    let mut action = Some(action);
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
        if !triggered && text.contains(marker) {
            match action.take().unwrap() {
                Action::Interrupt => {
                    writer.write_all(b"\x03")?;
                    writer.flush()?;
                }
                Action::Disconnect(disconnect) => {
                    let _ = disconnect.send(());
                }
            }
            triggered = true;
        }
        if !continued && text.contains("Terminate batch job") {
            writer.write_all(b"N\r")?;
            writer.flush()?;
            continued = true;
        }
    }
    let text = String::from_utf8_lossy(&bytes);
    let plain = anstream::adapter::strip_str(&text).to_string();
    let text = plain.replace("\r\n", "\n");
    Ok((text, triggered))
}

/* === Tests === */
