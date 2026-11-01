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

struct WatchTree;

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

    #[parameter(label = "Cancel And Collect", default = false)]
    cancel: bool,
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
    WatchTree(WatchTree),
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
                recursive: false,
                rules: Vec::new(),
                maximum_files: None,
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
        if ctx.parameters().cancel {
            watch.request_cancel().await?;
            let captured = watch.cancel().await?.into_owned();
            assert_eq!(captured.artifacts.len(), 1);
        } else {
            // Left Running: The Step-End Sweep Cancels It.
            watch.detach();
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

pub(crate) async fn harness_mirror_arms_before_rotation_and_retains_both_segments() {
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

pub(crate) async fn harness_copy_truncate_retains_the_previous_segment() {
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
pub(crate) async fn swept_watch_warning() {
    for cancel in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("device.log");
        std::fs::write(&path, BEFORE).unwrap();
        let report = Harness::template(TAIL)
            .parameter("path", path.to_str().unwrap())
            .parameter("cancel", cancel)
            .run::<Suites>()
            .await
            .expect("the run completes");

        assert!(report.passed(), "{}", report.diagnostics());
        assert_eq!(
            report.records::<StepHandleCancelled>(),
            if cancel {
                Vec::new()
            } else {
                vec![StepHandleCancelled {
                    step: "tail".into(),
                    kind: "watch".into(),
                    name: "logs/device.log".into(),
                }]
            }
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
}

#[dokime_runtime::template(id = "steps.checks.tree", label = "Capture Log Tree", parameters = TailParameters)]
impl WatchTree {
    async fn initialize(_: &TailParameters, _: &mut InitializationContext<'_>) -> Result<Self, Error> {
        Ok(Self)
    }

    #[dokime_runtime::step(label = "Capture Log Tree")]
    async fn capture(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        use model::files::wire::{FileRule, FileRuleAction, FileWatchSelection};
        let root = std::path::PathBuf::from(&ctx.parameters().path);
        let request = WatchFile {
            recursive: true,
            rules: vec![FileRule {
                glob: "Logs/**/*.log".into(),
                action: FileRuleAction::Include,
            }],
            maximum_files: Some(4),
            execution_id: None,
            source: Some(FileWatchSource {
                payload: Some(FileWatchSourcePayload::Local(LocalWatchSource {
                    root: root.to_string_lossy().into_owned(),
                })),
            }),
            output: "logs".into(),
            artifact: None,
            policy: FileWatchPolicy::Stream,
            selection: FileWatchSelection::All,
            wait_for_source_seconds: None,
            poll_interval_ms: Some(50),
            maximum_seconds: None,
            maximum_bytes: None,
            matchers: Vec::new(),
            markers: Vec::new(),
        };
        let stream = ctx.host().submit::<WatchFile>(request.clone()).await?;
        stream.armed().await?;
        std::fs::OpenOptions::new().append(true).open(root.join("Logs/existing.log"))?.write_all(b"after\n")?;
        std::fs::create_dir_all(root.join("Logs/next-day"))?;
        std::fs::write(root.join("Logs/next-day/new.log"), b"new\n")?;
        std::fs::write(root.join("Logs/next-day/ignored.zip"), b"ignored")?;
        let outcome = stream.cancel().await?.into_owned();
        assert_eq!(outcome.artifacts.len(), 2);

        let mirror_request = WatchFile {
            rules: vec![FileRule {
                glob: "**/*.zip".into(),
                action: FileRuleAction::Include,
            }],
            policy: FileWatchPolicy::Mirror,
            ..request.clone()
        };
        let mirror = ctx.host().submit::<WatchFile>(mirror_request).await?;
        mirror.armed().await?;
        std::fs::write(root.join("bundle.zip"), b"updated archive")?;
        std::fs::create_dir_all(root.join("deep"))?;
        std::fs::write(root.join("deep/new.zip"), b"new archive")?;
        std::fs::write(root.join("deep/ignored.log"), b"ignored")?;
        let outcome = mirror.cancel().await?.into_owned();
        assert_eq!(outcome.artifacts.len(), 3);

        for extension in ["diag", "hpl30"] {
            let name = format!("capture.{extension}");
            let snapshot = WatchFile {
                recursive: false,
                rules: Vec::new(),
                source: Some(FileWatchSource {
                    payload: Some(FileWatchSourcePayload::Local(LocalWatchSource {
                        root: root.join(&name).to_string_lossy().into_owned(),
                    })),
                }),
                output: String::new(),
                policy: FileWatchPolicy::SnapshotAtStart,
                ..request.clone()
            };
            ctx.host().submit::<WatchFile>(snapshot.clone()).await?.join().await?;
            for unsafe_name in ["../escape.diag", "/absolute.hpl30", "C:/escape.diag", "logs/CON.diag", "NUL.hpl30"] {
                let invalid = WatchFile {
                    artifact: Some(unsafe_name.into()),
                    ..snapshot.clone()
                };
                assert!(ctx.host().submit::<WatchFile>(invalid).await?.join().await.is_err(), "{unsafe_name}");
                assert!(ctx.artifacts().write(unsafe_name, b"refused").is_err());
            }
        }
        let all = WatchFile {
            rules: Vec::new(),
            policy: FileWatchPolicy::SnapshotAtStart,
            output: "all".into(),
            maximum_files: None,
            ..request.clone()
        };
        ctx.host().submit::<WatchFile>(all).await?.join().await?;
        let capped = WatchFile {
            maximum_files: Some(1),
            output: "capped-files".into(),
            ..request.clone()
        };
        assert!(ctx.host().submit::<WatchFile>(capped).await?.join().await.is_err());
        let capped = WatchFile {
            maximum_bytes: Some(20),
            output: "capped-bytes".into(),
            rules: vec![FileRule {
                glob: "**/*.zip".into(),
                action: FileRuleAction::Include,
            }],
            policy: FileWatchPolicy::Mirror,
            ..request
        };
        assert!(ctx.host().submit::<WatchFile>(capped).await?.join().await.is_err());
        Ok(())
    }
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn recursive_harness_watches() {
    let tree = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tree.path().join("Logs")).unwrap();
    std::fs::write(tree.path().join("Logs/existing.log"), b"before\n").unwrap();
    std::fs::write(tree.path().join("bundle.zip"), b"old").unwrap();
    std::fs::write(tree.path().join("Logs/status"), b"no extension").unwrap();
    for extension in ["diag", "hpl30"] {
        std::fs::write(tree.path().join(format!("capture.{extension}")), b"snapshot").unwrap();
    }
    let report = Harness::template("steps.checks.tree").parameter("path", tree.path().to_str().unwrap()).run::<Suites>().await.unwrap();
    assert!(report.passed(), "{}", report.diagnostics());
    for (name, bytes) in [
        ("logs/Logs/existing.log", b"after\n".as_slice()),
        ("logs/Logs/next-day/new.log", b"new\n"),
        ("logs/bundle.zip", b"updated archive"),
        ("logs/deep/new.zip", b"new archive"),
        ("capture.diag", b"snapshot"),
        ("capture.hpl30", b"snapshot"),
        ("all/Logs/status", b"no extension"),
    ] {
        assert_eq!(report.artifact(name).bytes, bytes, "{name}");
    }
    assert!(!report.artifacts().iter().any(|artifact| artifact.name == "logs/deep/ignored.log"));
    for name in ["capture.diag", "capture.hpl30"] {
        assert_eq!(report.artifact(name).media_type, "application/octet-stream");
    }
}
