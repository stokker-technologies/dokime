/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The file watch built-in through `ctx.host()` with no host behind it: a
// watch a step leaves running for the step-end sweep.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::io::Write;
use std::time::Duration;

// Third Party Imports
use dokime_runtime::harmos::operations::Host;

// Local Imports
use dokime_domain::model;
use dokime_runtime::host::StepHandleCancelled;
use dokime_runtime::plugin::testing::Harness;
use dokime_runtime::prelude::*;
use {
    dokime_runtime::operations::files::WatchFile, model::files::wire::FileWatchPolicy, model::files::wire::FileWatchSource,
    model::files::wire::FileWatchSourcePayload, model::files::wire::LineMarker, model::files::wire::LineMatcher,
    model::files::wire::LocalWatchSource,
};

/* === Constants === */
const SUITE: &str = "steps.checks";
const TAIL: &str = "steps.checks.tail";

const BEFORE: &[u8] = b"before the watch\n";
const APPENDED: &[u8] = b"temp=41.5 state=charging\nFAULT E7\n";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Checks;

struct LeaveWatch;

#[dokime_runtime::stream(
    id = "steps.checks.device",
    window = 64,
    label = "Device Log",
    description = "Lines the device logged, read by a matcher."
)]
struct DeviceLog {
    temperature_c: f64,
    state: String,
}

#[dokime_runtime::record(id = "steps.checks.fault", label = "Device Fault", description = "The device logged a fault.")]
#[derive(Clone, Debug)]
struct Fault {
    #[record(description = "The fault code the device logged.")]
    code: String,
}

#[dokime_runtime::parameters]
struct TailParameters {
    #[parameter(label = "Watched File", default = "")]
    path: String,

    #[parameter(label = "Mirror And Rotate", default = false)]
    mirror: bool,

    #[parameter(label = "Copy Truncate", default = false)]
    copy_truncate: bool,
}

/* === Enums === */
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("dokime sdk: {0}")]
    Sdk(#[from] dokime_runtime::Error),
    #[error("host: {0}")]
    Host(#[from] dokime_runtime::harmos::sidecar::CallError),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

#[dokime_runtime::catalog]
enum Suites {
    Checks(Checks),
}

#[dokime_runtime::catalog]
enum Templates {
    LeaveWatch(LeaveWatch),
}

#[dokime_runtime::catalog]
enum Streams {
    DeviceLog(DeviceLog),
}

#[dokime_runtime::catalog]
enum Records {
    Fault(Fault),
}

/* === Traits === */

/* === Implementations === */
#[dokime_runtime::suite(id = SUITE, label = "Step Built-Ins", templates = Templates)]
impl Checks {}

#[dokime_runtime::template(
    id = TAIL, label = "Leave a Watch Running",
    parameters = TailParameters, streams = Streams, records = Records)]
impl LeaveWatch {
    async fn initialize(_: &TailParameters, _ctx: &mut InitializationContext<'_>) -> Result<Self, Error> {
        Ok(Self)
    }

    #[dokime_runtime::step(label = "Tail the Device Log")]
    async fn tail(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let path = ctx.parameters().path.clone();
        let mirror = ctx.parameters().mirror;
        let matchers = if mirror {
            Vec::new()
        } else {
            vec![LineMatcher::publishing::<DeviceLog>(r"temp=(?P<temperature_c>[\d.]+) state=(?P<state>\w+)")]
        };
        let markers = if mirror { Vec::new() } else { vec![LineMarker::new(r"FAULT (?P<code>\w+)", "steps.checks.fault")] };
        let watch = ctx
            .host()
            .submit(WatchFile {
                execution_id: None,
                source: Some(FileWatchSource {
                    payload: Some(FileWatchSourcePayload::Local(LocalWatchSource { root: path.clone() })),
                }),
                output: "logs".into(),
                artifact: Some("logs/device.log".into()),
                policy: if mirror { FileWatchPolicy::Mirror } else { FileWatchPolicy::Stream },
                selection: dokime_domain::model::files::wire::FileWatchSelection::All,
                wait_for_source_seconds: None,
                poll_interval_ms: Some(50),
                maximum_seconds: mirror.then_some(1),
                maximum_bytes: None,
                matchers,
                markers,
            })
            .await?;
        // The device writes once the watch has taken its baseline.
        watch.armed().await?;
        if mirror {
            if ctx.parameters().copy_truncate {
                std::fs::write(&path, b"x\n")?;
            } else {
                std::fs::rename(&path, format!("{path}.old"))?;
                std::fs::write(&path, APPENDED)?;
            }
            let captured = watch.join().await?.into_owned();
            assert_eq!(captured.artifacts.len(), 2);
            return Ok(());
        }
        std::fs::OpenOptions::new().append(true).open(&path)?.write_all(APPENDED)?;
        tokio::time::sleep(Duration::from_millis(150)).await;
        // Left running: the step-end sweep cancels it.
        watch.detach();
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn harness_mirror_arms_before_rotation_and_retains_both_segments() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("device.log");
    std::fs::write(&path, BEFORE).unwrap();
    let report = Harness::template(TAIL)
        .parameter("path", path.to_str().unwrap())
        .parameter("mirror", true)
        .run::<Suites>()
        .await
        .expect("the run completes");
    assert!(report.passed(), "{}", report.diagnostics());
    assert_eq!(report.artifact("logs/device.log").bytes, BEFORE);
    assert_eq!(report.artifact("logs/device.segment2.log").bytes, APPENDED);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn harness_copy_truncate_retains_the_previous_segment() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("device.log");
    std::fs::write(&path, BEFORE).unwrap();
    let report = Harness::template(TAIL)
        .parameter("path", path.to_str().unwrap())
        .parameter("mirror", true)
        .parameter("copy_truncate", true)
        .run::<Suites>()
        .await
        .expect("the run completes");
    assert!(report.passed(), "{}", report.diagnostics());
    assert_eq!(report.artifact("logs/device.log").bytes, BEFORE);
    assert_eq!(report.artifact("logs/device.segment2.log").bytes, b"x\n");
}

// A watch left running past its step is cancelled by the sweep with a
// warning; until then its matcher and marker turned the lines it captured
// into a row and a record, and its artifact holds only what came after it.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn swept_watch_warning() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("device.log");
    std::fs::write(&path, BEFORE).unwrap();
    let report = Harness::template(TAIL).parameter("path", path.to_str().unwrap()).run::<Suites>().await.expect("the run completes");

    assert!(report.passed(), "{}", report.diagnostics());
    assert_eq!(
        report.records::<StepHandleCancelled>(),
        vec![StepHandleCancelled {
            step: "tail".into(),
            kind: "watch".into(),
            name: "logs/device.log".into()
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
