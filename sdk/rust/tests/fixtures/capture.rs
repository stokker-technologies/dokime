/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Local File Capture Fixture

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

// Third Party Imports

// Local Imports
use dokime::plugin::{CaptureMode, StreamCapture, capture_mode};
use dokime::prelude::*;
use dokime::{FileLocation, FileRule, FileRuleAction, FileSelection, FileSource, file_location, file_source};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Parameters)]
#[dokime(package = "capture")]
struct Device {
    root: String,
    outside: String,
}

#[derive(dokime::Record)]
#[dokime(package = "capture")]
struct Captured {
    files: u32,
    bytes: u64,
    gaps: u32,
}

#[derive(dokime::Record)]
#[dokime(package = "capture")]
struct Refused {
    refused: bool,
    classification: String,
}

#[derive(Default)]
struct Running(AtomicU64);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<()> {
    let copy = Template::new("copy", "Copy Device Files")
        .record(Record::<Captured>::new("captured", "Captured"))
        .parameters(device())
        .step(Step::new("Copy", copy).identifier("copy"));
    let window = Template::new("window", "Capture a Device Window")
        .record(Record::<Captured>::new("captured", "Captured"))
        .parameters(device())
        .initialize(initialize)
        .step(Step::new("Start", start).identifier("start"))
        .step(Step::new("Produce", produce).identifier("produce"))
        .step(Step::new("Stop", stop).identifier("stop"));
    let refused = Template::new("refused", "Escape the Capture Roots")
        .record(Record::<Refused>::new("refusal", "Refused"))
        .parameters(device())
        .step(Step::new("Escape", escape).identifier("escape"));
    let cancelled = Template::new("cancelled", "Cancel a Running Capture")
        .record(Record::<Captured>::new("captured", "Captured"))
        .parameters(device())
        .initialize(initialize)
        .step(Step::new("Start", start).identifier("start"))
        .step(Step::new("Produce", produce).identifier("produce"))
        .step(Step::new("Hold", hold).identifier("hold"));
    Plugin::new("capture", "1.0.0")
        .suite(Suite::new("capture", "Capture").template(copy).template(window).template(refused).template(cancelled))
        .serve()
}

fn device() -> Device {
    Device {
        root: String::new(),
        outside: String::new(),
    }
}

fn tree(root: &str) -> FileSource {
    FileSource {
        kind: Some(file_source::Kind::Tree(FileSelection {
            root: Some(FileLocation {
                kind: Some(file_location::Kind::Local(root.into())),
            }),
            rules: vec![FileRule {
                glob: "**".into(),
                action: FileRuleAction::Include as i32,
            }],
        })),
    }
}

async fn initialize(ctx: &Context) -> Result<()> {
    ctx.set(Running::default());
    Ok(())
}

async fn copy(ctx: &Context) -> Result<()> {
    let device = ctx.parameters::<Device>()?;
    let copied = ctx.host().files.capture(tree(&device.root), "device".into()).await?;
    let bytes = copied.artifacts.iter().map(|artifact| artifact.size_bytes).sum();
    let captured = Captured {
        files: copied.artifacts.len() as u32,
        bytes,
        gaps: copied.gaps.len() as u32,
    };
    ctx.record(&Record::<Captured>::new("captured", "Captured"), captured).await?;
    Ok(())
}

async fn start(ctx: &Context) -> Result<()> {
    let device = ctx.parameters::<Device>()?;
    let mode = CaptureMode {
        kind: Some(capture_mode::Kind::Stream(StreamCapture::default())),
    };
    let capture_id = ctx.host().files.captures.start(tree(&device.root), "window".into(), mode).await?;
    ctx.get::<Running>()?.0.store(capture_id, Ordering::SeqCst);
    Ok(())
}

async fn produce(ctx: &Context) -> Result<()> {
    // The Device Appends to Its Existing Log and Creates a New One Inside the Window.
    let device = ctx.parameters::<Device>()?;
    let root = Path::new(&device.root);
    let existing = std::fs::read(root.join("device.log")).map_err(|error| Error::Invalid(error.to_string()))?;
    std::fs::write(root.join("device.log"), [existing.as_slice(), b"late\n"].concat())
        .map_err(|error| Error::Invalid(error.to_string()))?;
    std::fs::write(root.join("fresh.log"), b"fresh\n").map_err(|error| Error::Invalid(error.to_string()))?;
    Ok(())
}

async fn stop(ctx: &Context) -> Result<()> {
    let capture_id = ctx.get::<Running>()?.0.load(Ordering::SeqCst);
    let stopped = ctx.host().files.captures.stop(capture_id).await?;
    let repeated = ctx.host().files.captures.stop(capture_id).await?;
    if repeated != stopped {
        return Err(Error::Invalid("Stop Capture: Return the Same Outcome Again".into()));
    }
    let captured = Captured {
        files: stopped.artifacts.len() as u32,
        bytes: stopped.artifacts.iter().map(|artifact| artifact.size_bytes).sum(),
        gaps: stopped.gaps.len() as u32,
    };
    ctx.record(&Record::<Captured>::new("captured", "Captured"), captured).await?;
    Ok(())
}

async fn escape(ctx: &Context) -> Result<()> {
    let device = ctx.parameters::<Device>()?;
    let escaped = ctx.host().files.capture(tree(&device.outside), "escape".into()).await;
    // Keep the Typed Refusal; the Story Compares Its Classification, Never Its Message.
    let classification = escaped.as_ref().err().map(|error| format!("{error:?}")).unwrap_or_default();
    let refused = Refused {
        refused: escaped.is_err(),
        classification,
    };
    ctx.record(&Record::<Refused>::new("refusal", "Refused"), refused).await?;
    Ok(())
}

async fn hold(ctx: &Context) -> Result<()> {
    ctx.prompt("Keep the Capture Running?", &["Continue"]).await?;
    Ok(())
}

/* === Tests === */
