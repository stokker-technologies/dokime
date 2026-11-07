/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The file capture built-ins through `ctx.operation(request)` with no host behind
// them, across steps and with or without a stop.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::io::Write;

// Third Party Imports

// Local Imports
use dokime_domain::model;
use dokime_runtime::plugin::testing::Harness;
use dokime_runtime::prelude::*;
use {
    dokime_runtime::operations::files::CopyFiles, dokime_runtime::operations::files::StartCapture,
    dokime_runtime::operations::files::StopCapture, model::files::wire::FileRule, model::files::wire::FileSource,
    model::files::wire::LineMarker, model::files::wire::LineMatcher, model::files::wire::MirrorCapture, model::files::wire::StreamCapture,
};

/* === Constants === */
const SUITE: &str = "steps.checks";
const TAIL: &str = "steps.checks.tail";

const BEFORE: &[u8] = b"before the capture\n";
const APPENDED: &[u8] = b"temp=41.5 state=charging\nFAULT E7\n";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Checks;

struct TailDevice;

struct CaptureTree;

struct AcrossSteps {
    capture_id: u64,
}

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
    #[parameter(label = "Captured File", default = "")]
    path: String,

    #[parameter(label = "Mirror and Rotate", default = false)]
    mirror: bool,

    #[parameter(label = "Copy Truncate", default = false)]
    copy_truncate: bool,

    #[parameter(label = "Stop and Collect", default = false)]
    stop: bool,
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
    TailDevice(TailDevice),
    CaptureTree(CaptureTree),
    AcrossSteps(AcrossSteps),
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
    id = TAIL, label = "Capture a Device Log",
    parameters = TailParameters, streams = Streams, records = Records)]
impl TailDevice {
    async fn initialize(_: &TailParameters, _ctx: &mut InitializationContext<'_>) -> Result<Self, Error> {
        Ok(Self)
    }

    #[dokime_runtime::step(label = "Tail the Device Log")]
    async fn tail(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let path = ctx.parameters().path.clone();
        if ctx.parameters().mirror {
            // Start Capture
            let request = StartCapture::new(FileSource::file(&path), "logs/device.log", MirrorCapture::every(3600));
            let capture = ctx.operation(request).await?;

            // Rotate Or Truncate The Source
            if ctx.parameters().copy_truncate {
                std::fs::write(&path, b"x\n")?;
            } else {
                std::fs::rename(&path, format!("{path}.old"))?;
                std::fs::write(&path, APPENDED)?;
            }

            // Stop Capture
            let request = StopCapture { capture_id: capture.capture_id };
            let captured = ctx.operation(request).await?.into_owned();
            assert_eq!(captured.artifacts.len(), 2);
            return Ok(());
        }

        // Start Capture
        let matcher = LineMatcher::publishing::<DeviceLog>(r"temp=(?P<temperature_c>[\d.]+) state=(?P<state>\w+)");
        let marker = LineMarker::new(r"FAULT (?P<code>\w+)", "steps.checks.fault");
        let request = StartCapture::new(FileSource::file(&path), "logs/device.log", StreamCapture::plain().matcher(matcher).marker(marker));
        let capture = ctx.operation(request).await?;

        // Write To The Source
        std::fs::OpenOptions::new().append(true).open(&path)?.write_all(APPENDED)?;

        // Stop Capture, Or Leave It To Run End
        if ctx.parameters().stop {
            let request = StopCapture { capture_id: capture.capture_id };
            let captured = ctx.operation(request).await?.into_owned();
            assert_eq!(captured.artifacts.len(), 1);
        }
        Ok(())
    }
}

#[dokime_runtime::template(id = "steps.checks.across", label = "Capture Across Steps", parameters = TailParameters)]
impl AcrossSteps {
    async fn initialize(_: &TailParameters, _: &mut InitializationContext<'_>) -> Result<Self, Error> {
        Ok(Self { capture_id: 0 })
    }

    #[dokime_runtime::step(label = "Start Capture")]
    async fn start(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let source = FileSource::file(ctx.parameters().path.as_str());
        let request = StartCapture::new(source, "logs/device.log", StreamCapture::plain());
        self.capture_id = ctx.operation(request).await?.capture_id;
        Ok(())
    }

    #[dokime_runtime::step(label = "Write Between Steps")]
    async fn write(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        std::fs::OpenOptions::new().append(true).open(&ctx.parameters().path)?.write_all(APPENDED)?;
        Ok(())
    }

    #[dokime_runtime::step(label = "Stop Capture")]
    async fn stop(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let request = StopCapture { capture_id: self.capture_id };
        let captured = ctx.operation(request).await?.into_owned();
        assert_eq!(captured.artifacts.len(), 1);
        assert_eq!(captured.artifacts[0].size_bytes, APPENDED.len() as u64);
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

// Whether a step stops a capture or leaves it to run end, its matcher and marker
// turned the lines it captured into a row and a record, and its artifact holds
// only what came after it.
pub(crate) async fn capture_seals_with_or_without_a_stop() {
    for stop in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("device.log");
        std::fs::write(&path, BEFORE).unwrap();
        let report = Harness::template(TAIL)
            .parameter("path", path.to_str().unwrap())
            .parameter("stop", stop)
            .run::<Suites>()
            .await
            .expect("the run completes");

        assert!(report.passed(), "{}", report.diagnostics());
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

pub(crate) async fn capture_spans_steps() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("device.log");
    std::fs::write(&path, BEFORE).unwrap();
    let report = Harness::template("steps.checks.across")
        .parameter("path", path.to_str().unwrap())
        .run::<Suites>()
        .await
        .expect("the run completes");
    assert!(report.passed(), "{}", report.diagnostics());
    assert_eq!(report.artifact("logs/device.log").bytes, APPENDED);
}

#[dokime_runtime::template(id = "steps.checks.tree", label = "Capture Log Tree", parameters = TailParameters)]
impl CaptureTree {
    async fn initialize(_: &TailParameters, _: &mut InitializationContext<'_>) -> Result<Self, Error> {
        Ok(Self)
    }

    #[dokime_runtime::step(label = "Capture Log Tree")]
    async fn capture(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let root = std::path::PathBuf::from(&ctx.parameters().path);
        let logs = || FileSource::tree(&root, [FileRule::include("Logs/**/*.log")]);

        // Capture An Untouched Tree
        let request = StartCapture::new(logs(), "logs", StreamCapture::plain());
        let untouched = ctx.operation(request).await?;
        let request = StopCapture { capture_id: untouched.capture_id };
        assert!(ctx.operation(request).await?.artifacts.is_empty());

        // Stream A Growing Tree
        let request = StartCapture::new(logs(), "logs", StreamCapture::plain());
        let stream = ctx.operation(request).await?;
        std::fs::OpenOptions::new().append(true).open(root.join("Logs/existing.log"))?.write_all(b"after\n")?;
        std::fs::create_dir_all(root.join("Logs/next-day"))?;
        std::fs::write(root.join("Logs/next-day/new.log"), b"new\n")?;
        std::fs::write(root.join("Logs/next-day/empty.log"), b"")?;
        std::fs::write(root.join("Logs/next-day/ignored.zip"), b"ignored")?;
        let request = StopCapture { capture_id: stream.capture_id };
        let outcome = ctx.operation(request).await?.into_owned();
        assert_eq!(outcome.artifacts.len(), 3);
        ctx.artifacts().write("same/device.log", b"first")?;
        ctx.artifacts().write("same/device.log", b"second")?;

        // Mirror Archives
        let source = FileSource::tree(&root, [FileRule::include("**/*.zip")]);
        let request = StartCapture::new(source, "logs", MirrorCapture::every(3600));
        let mirror = ctx.operation(request).await?;
        std::fs::write(root.join("bundle.zip"), b"updated archive")?;
        std::fs::create_dir_all(root.join("deep"))?;
        std::fs::write(root.join("deep/new.zip"), b"new archive")?;
        std::fs::write(root.join("deep/ignored.log"), b"ignored")?;
        let request = StopCapture { capture_id: mirror.capture_id };
        let outcome = ctx.operation(request).await?.into_owned();
        assert_eq!(outcome.artifacts.len(), 3);

        // Copy Files With Ordinary Extensions, Refusing Unsafe Names
        for extension in ["diag", "hpl30"] {
            let name = format!("capture.{extension}");
            let source = FileSource::file(root.join(&name));
            ctx.operation(CopyFiles::new(source.clone(), name)).await?;
            for unsafe_name in ["../escape.diag", "/absolute.hpl30", "C:/escape.diag", "logs/CON.diag", "NUL.hpl30"] {
                assert!(ctx.operation(CopyFiles::new(source.clone(), unsafe_name)).await.is_err(), "{unsafe_name}");
                assert!(ctx.artifacts().write(unsafe_name, b"refused").is_err());
            }
        }

        // Copy Everything
        ctx.operation(CopyFiles::new(FileSource::tree(&root, []), "all")).await?;
        Ok(())
    }
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn harness_captures_trees() {
    let tree = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tree.path().join("Logs")).unwrap();
    std::fs::write(tree.path().join("Logs/existing.log"), b"before\n").unwrap();
    std::fs::write(tree.path().join("Logs/untouched.log"), b"untouched\n").unwrap();
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
        ("logs/Logs/next-day/empty.log", b""),
        ("same/device.log", b"first"),
        ("same/device.2.log", b"second"),
        ("logs/bundle.zip", b"updated archive"),
        ("logs/deep/new.zip", b"new archive"),
        ("capture.diag", b"snapshot"),
        ("capture.hpl30", b"snapshot"),
        ("all/Logs/status", b"no extension"),
    ] {
        assert_eq!(report.artifact(name).bytes, bytes, "{name}");
    }
    assert!(
        !report.artifacts().iter().any(|artifact| ["logs/deep/ignored.log", "logs/Logs/untouched.log"].contains(&artifact.name.as_str()))
    );
    for name in ["capture.diag", "capture.hpl30"] {
        assert_eq!(report.artifact(name).media_type, "application/octet-stream");
    }
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn harness_live_capture() {
    use dokime_runtime::artifacts::{ArtifactSink, Artifacts};
    use dokime_runtime::execution::Environment;
    use dokime_runtime::records::Deliveries;
    use dokime_runtime::telemetry::Streams;
    use dokime_runtime::transport::ArtifactChunk;
    use dokime_runtime::transport::recording::{RecordingSink, RecordingTransport};
    use std::sync::Arc;

    struct Captured {
        sink: RecordingSink,
        changed: tokio::sync::Notify,
    }
    impl ArtifactSink for Captured {
        fn push(&self, chunk: ArtifactChunk) {
            self.sink.push(chunk);
            self.changed.notify_one();
        }
    }
    impl Captured {
        async fn length(&self, length: usize) {
            loop {
                let changed = self.changed.notified();
                if self.sink.assembled("logs/device.log").len() == length {
                    return;
                }
                changed.await;
            }
        }
    }
    let tree = tempfile::tempdir().unwrap();
    let path = tree.path().join("device.log");
    std::fs::write(&path, b"before").unwrap();
    let captured = Arc::new(Captured {
        sink: RecordingSink::new(),
        changed: tokio::sync::Notify::new(),
    });
    let sink = Arc::new(RecordingSink::new());
    let raw = Default::default();
    let configuration = Default::default();
    let environment = Environment::<(), (), ()>::new(
        &(),
        &raw,
        &configuration,
        Artifacts::new("live", captured.clone()),
        Streams::new(sink.clone()),
        Deliveries::new(sink),
    );
    let mut transport = RecordingTransport::new();
    let ctx = environment.step(&mut transport);
    let request = StartCapture::new(FileSource::tree(tree.path(), []), "logs", StreamCapture::plain());
    let capture = ctx.operation(request).await.unwrap();
    assert!(captured.sink.chunks().is_empty());
    std::fs::OpenOptions::new().append(true).open(&path).unwrap().write_all(b"one").unwrap();
    captured.length(3).await;
    let first = captured.sink.chunks();
    assert_eq!(first.len(), 1);
    assert!(!first[0].last);
    std::fs::OpenOptions::new().append(true).open(&path).unwrap().write_all(b"two").unwrap();
    captured.length(6).await;
    assert_eq!(captured.sink.assembled("logs/device.log"), b"onetwo");
    let request = StopCapture { capture_id: capture.capture_id };
    let outcome = ctx.operation(request).await.unwrap();
    assert_eq!(outcome.artifacts.len(), 1);
    assert_eq!(outcome.artifacts[0].artifact, "logs/device.log");
    assert_eq!(outcome.artifacts[0].size_bytes, 6);
    let chunks = captured.sink.chunks();
    assert!(chunks.last().unwrap().last);
    assert!(chunks.iter().all(|chunk| chunk.transfer_id == first[0].transfer_id));
}
