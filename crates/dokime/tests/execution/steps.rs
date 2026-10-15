/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The step built-ins through `ctx.host()` with no host behind them: a process
// run once with its output in a produced artifact, a process spawned and
// cancelled, and a file watch a step leaves running for the step-end sweep.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::io::Write;
use std::time::Duration;

// Third Party Imports
use dokime::harmos::operations::Host;

// Local Imports
use dokime::execution::log::Level;
use dokime::host::{ProcessExited, StepHandleCancelled};
use dokime::operations::files::{
    FileWatchPolicy, FileWatchSource, FileWatchSourcePayload, LineMarker, LineMatcher,
    LocalWatchSource, WatchFile, WatchFileRequest,
};
use dokime::operations::process::{Output, RunProcess, RunProcessRequest};
use dokime::plugin::testing::Harness;
use dokime::prelude::*;

/* === Constants === */
const SUITE: &str = "steps.checks";
const RUN: &str = "steps.checks.run";
const CANCEL: &str = "steps.checks.cancel";
const TAIL: &str = "steps.checks.tail";

const BEFORE: &[u8] = b"before the watch\n";
const APPENDED: &[u8] = b"temp=41.5 state=charging\nFAULT E7\n";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Checks;

struct RunOnce;

struct SpawnCancel;

struct LeaveWatch;

#[dokime::stream(
    id = "steps.checks.device",
    window = 64,
    label = "Device Log",
    description = "Lines the device logged, read by a matcher."
)]
struct DeviceLog {
    temperature_c: f64,
    state: String,
}

#[dokime::record(
    id = "steps.checks.fault",
    label = "Device Fault",
    description = "The device logged a fault."
)]
#[derive(Clone, Debug)]
struct Fault {
    #[record(description = "The fault code the device logged.")]
    code: String,
}

#[dokime::parameters]
struct TailParameters {
    #[parameter(label = "Watched File", default = "")]
    path: String,
}

/* === Enums === */
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("dokime sdk: {0}")]
    Sdk(#[from] dokime::Error),
    #[error("host: {0}")]
    Host(#[from] dokime::harmos::sidecar::CallError),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Unexpected(String),
}

#[dokime::catalog]
enum Suites {
    Checks(Checks),
}

#[dokime::catalog]
enum Templates {
    RunOnce(RunOnce),
    SpawnCancel(SpawnCancel),
    LeaveWatch(LeaveWatch),
}

#[dokime::catalog]
enum Streams {
    DeviceLog(DeviceLog),
}

#[dokime::catalog]
enum Records {
    Fault(Fault),
}

/* === Traits === */

/* === Implementations === */
#[dokime::suite(id = SUITE, label = "Step Built-Ins", templates = Templates)]
impl Checks {}

#[dokime::template(id = RUN, label = "Run Once")]
impl RunOnce {
    async fn initialize(_: &(), _ctx: &mut InitializationContext<'_>) -> Result<Self, Error> {
        Ok(Self)
    }

    #[dokime::step(label = "Run the Tool")]
    async fn run(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let exit = ctx
            .host()
            .call_as::<RunProcess>(
                shell("printf 'alpha\\nbeta\\n'; echo \"template=$DOKIME_TEMPLATE_ID\" >&2")
                    .stdout(Output::Artifact("logs/tool.log".into())),
            )
            .await?
            .into_owned();
        if !exit.success() || exit.cancelled {
            return Err(Error::Unexpected(format!(
                "the tool exited {:?}",
                exit.exit_code
            )));
        }
        Ok(())
    }
}

#[dokime::template(id = CANCEL, label = "Spawn And Cancel")]
impl SpawnCancel {
    async fn initialize(_: &(), _ctx: &mut InitializationContext<'_>) -> Result<Self, Error> {
        Ok(Self)
    }

    #[dokime::step(label = "Spawn the Tool")]
    async fn spawn(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let running = ctx
            .host()
            .submit_as::<RunProcess>(shell("echo started; exec sleep 30"))
            .await?;
        tokio::time::sleep(Duration::from_millis(300)).await;
        let exit = running.cancel_and_join().await?.into_owned();
        if !exit.cancelled || exit.duration() >= Duration::from_secs(30) {
            return Err(Error::Unexpected("the tool was not cancelled".into()));
        }
        Ok(())
    }
}

#[dokime::template(
    id = TAIL, label = "Leave a Watch Running",
    parameters = TailParameters, streams = Streams, records = Records)]
impl LeaveWatch {
    async fn initialize(
        _: &TailParameters,
        _ctx: &mut InitializationContext<'_>,
    ) -> Result<Self, Error> {
        Ok(Self)
    }

    #[dokime::step(label = "Tail the Device Log")]
    async fn tail(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let path = ctx.parameters().path.clone();
        let watch = ctx
            .host()
            .submit_as::<WatchFile>(WatchFileRequest {
                execution_id: None,
                source: Some(FileWatchSource {
                    payload: Some(FileWatchSourcePayload::Local(LocalWatchSource {
                        root: path.clone(),
                    })),
                }),
                output: "logs".into(),
                artifact: Some("logs/device.log".into()),
                policy: FileWatchPolicy::Stream,
                poll_interval_ms: Some(50),
                maximum_seconds: None,
                maximum_bytes: None,
                matchers: vec![LineMatcher::publishing::<DeviceLog>(
                    r"temp=(?P<temperature_c>[\d.]+) state=(?P<state>\w+)",
                )],
                markers: vec![LineMarker::new(
                    r"FAULT (?P<code>\w+)",
                    "steps.checks.fault",
                )],
            })
            .await?;
        // The device writes once the watch has taken its baseline.
        tokio::time::sleep(Duration::from_millis(150)).await;
        std::fs::OpenOptions::new()
            .append(true)
            .open(&path)?
            .write_all(APPENDED)?;
        tokio::time::sleep(Duration::from_millis(150)).await;
        // Left running: the step-end sweep cancels it.
        watch.detach();
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
#[cfg(unix)]
fn shell(script: &str) -> RunProcessRequest {
    RunProcessRequest::new("sh").args(["-c", script])
}

#[cfg(windows)]
fn shell(script: &str) -> RunProcessRequest {
    RunProcessRequest::new("cmd").args(["/C", script])
}

fn logged(report: &dokime::plugin::testing::Report, level: Level) -> Vec<String> {
    report
        .log()
        .iter()
        .filter(|entry| entry.level == level as i32)
        .map(|entry| entry.message.clone())
        .collect()
}

/* === Tests === */
// One shot: stdout streams into the produced artifact, stderr reaches the log
// at warning with the execution's identity, and the exit is recorded.
#[cfg(unix)]
pub(crate) async fn process_output_artifact() {
    let report = Harness::template(RUN)
        .run::<Suites>()
        .await
        .expect("the run completes");

    assert!(report.passed(), "{}", report.diagnostics());
    assert_eq!(report.artifact("logs/tool.log").bytes, b"alpha\nbeta\n");
    assert_eq!(
        logged(&report, Level::Warn),
        vec![format!("template={RUN}")],
        "{}",
        report.diagnostics()
    );
    assert!(
        logged(&report, Level::Info)
            .iter()
            .all(|line| !line.contains("alpha")),
        "redirected output stays out of the log"
    );
    let exited = report.records::<ProcessExited>();
    assert_eq!(exited.len(), 1);
    assert_eq!(exited[0].program, "sh");
    assert_eq!(exited[0].exit_code, Some(0));
    assert!(!exited[0].cancelled);
}

// Submitted and cancelled: SIGTERM ends it, and the exit says so.
#[cfg(unix)]
pub(crate) async fn process_cancel_exit() {
    let report = Harness::template(CANCEL)
        .run::<Suites>()
        .await
        .expect("the run completes");

    assert!(report.passed(), "{}", report.diagnostics());
    assert!(
        logged(&report, Level::Info).contains(&"started".to_owned()),
        "{}",
        report.diagnostics()
    );
    let exited = report.records::<ProcessExited>();
    assert_eq!(exited.len(), 1);
    assert!(exited[0].cancelled);
    assert_eq!(exited[0].exit_code, None, "a signal ended it");
    assert!(exited[0].duration_ms < 30_000);
    assert!(report.records::<StepHandleCancelled>().is_empty());
}

// A watch left running past its step is cancelled by the sweep with a
// warning; until then its matcher and marker turned the lines it captured
// into a row and a record, and its artifact holds only what came after it.
pub(crate) async fn swept_watch_warning() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("device.log");
    std::fs::write(&path, BEFORE).unwrap();
    let report = Harness::template(TAIL)
        .parameter("path", path.to_str().unwrap())
        .run::<Suites>()
        .await
        .expect("the run completes");

    assert!(report.passed(), "{}", report.diagnostics());
    assert_eq!(
        report.records::<StepHandleCancelled>(),
        vec![StepHandleCancelled {
            step: "tail".into(),
            kind: "watch".into(),
            name: "logs/device.log".into(),
        }]
    );
    let rows = report.samples::<DeviceLog>();
    assert_eq!(rows.len(), 1, "{}", report.diagnostics());
    assert_eq!(rows[0].temperature_c, 41.5);
    assert_eq!(rows[0].state, "charging");
    let faults = report.records::<Fault>();
    assert_eq!(faults.len(), 1);
    assert_eq!(faults[0].code, "E7");
    assert_eq!(report.artifact("logs/device.log").bytes, APPENDED);
}
