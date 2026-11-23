/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Exact Publication Boundary Fixture

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime::prelude::*;
use dokime::templates::Context;
use dokime::{SettingDeclaration, SettingValue, setting_value};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Row)]
#[dokime(package = "execution")]
struct Reading {
    value: f64,
}

/// One Granted Window of Readings, Each Holding the Window's Sample Count.
#[derive(dokime::Message)]
#[dokime(package = "execution")]
struct ReadingColumns {
    value: Vec<f64>,
}

#[derive(dokime::Parameters)]
#[dokime(package = "execution")]
struct CaptureRows {
    rows: u64,
}

#[derive(Default)]
struct Published {
    count: AtomicU64,
}

/// Marks the Lag Template's Scope, Whose Lagging Observer Waits Inside Each Window.
struct Lagging;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<()> {
    let boundary_stream = Stream::<Reading>::new("boundary", "Boundary Samples");
    let boundary_observer = Observer::new(&boundary_stream, async |context: &Context| {
        // Only the Boundary Template's Own Scope Counts Its Rows; Any Other Demand on the Stream Answers Zero.
        let Ok(published) = context.get::<Published>() else {
            return Ok(Reading { value: 0.0 });
        };
        let index = published.count.fetch_add(1, Ordering::SeqCst);
        Ok(Reading { value: index as f64 })
    })
    .every(Duration::from_millis(1));
    let boundary = Template::of::<()>()
        .identifier("boundary")
        .label("Publication Boundary")
        .parameters(CaptureRows { rows: 1024 })
        .initialize(async |context: &Context| {
            context.set(Published::default());
            Ok(())
        })
        .step(
            Step::new("Capture", async |context: &Context| {
                // A Scoped Sleep Settles After Every Window Due by Its End, So It Spans the Remaining Rows.
                let rows = context.parameters::<CaptureRows>()?.rows;
                let published = context.get::<Published>()?;
                context.sleep(Duration::from_millis(rows.saturating_sub(published.count.load(Ordering::SeqCst)))).await
            })
            .identifier("capture"),
        )
        .stream(boundary_stream)
        .observer(boundary_observer);
    // Two Window Observers at 1 kHz: One Takes the Host Window, One Its Plugin Duration Setting.
    let granted = async |_: &Context, window: &Window| {
        Ok(ReadingColumns {
            value: vec![window.count as f64; window.count as usize],
        })
    };
    let host_stream = Stream::<Reading>::new("window.host", "Host Window Samples");
    let host_observer = Observer::window(&host_stream, granted).every(Duration::from_millis(1));
    let setting_stream = Stream::<Reading>::new("window.setting", "Setting Window Samples");
    let setting_observer = Observer::window(&setting_stream, granted).every(Duration::from_millis(1)).window_setting("sample_window");
    let windows = Template::of::<()>()
        .identifier("windows")
        .label("Window Override")
        .parameters(CaptureRows { rows: 1000 })
        .step(
            Step::new("Capture", async |context: &Context| {
                context.sleep(Duration::from_millis(context.parameters::<CaptureRows>()?.rows)).await
            })
            .identifier("capture"),
        )
        .stream(host_stream)
        .observer(host_observer)
        .stream(setting_stream)
        .observer(setting_observer);
    // One 1 MHz Window Holding More Rows Than the Shared Plugin Publication Pool Admits at Once: the Step's
    // 10 s Sleep Cuts the Hour-Long Window, While the Real-Clock Connection Demand Never Reaches Its End.
    let overflow_stream = Stream::<Reading>::new("window.overflow", "Overflow Window Samples");
    let overflow_observer = Observer::window(&overflow_stream, async |_: &Context, window: &Window| {
        Ok(ReadingColumns { value: vec![0.0; window.count as usize] })
    })
    .every(Duration::from_micros(1))
    .window_setting("overflow_window");
    let overflow = Template::of::<()>()
        .identifier("overflow")
        .label("Window Overflow")
        .step(Step::new("Capture", async |context: &Context| context.sleep(Duration::from_secs(10)).await).identifier("capture"))
        .stream(overflow_stream)
        .observer(overflow_observer);
    // A 1 kHz Stream Whose Observer, in the Lag Template's Own Scope, Waits 100 ms of Host Time Inside Each Window: the
    // Wait Advances a Controlled Clock While the Window Stays Unanswered, So the Stream Falls Behind Its Window End.
    let lag_stream = Stream::<Reading>::new("window.lag", "Lagging Window Samples");
    let lag_observer = Observer::window(&lag_stream, async |context: &Context, window: &Window| {
        if context.get::<Lagging>().is_ok() {
            context.sleep(Duration::from_millis(100)).await?;
        }
        Ok(ReadingColumns { value: vec![0.0; window.count as usize] })
    })
    .every(Duration::from_millis(1));
    let lag = Template::of::<()>()
        .identifier("lag")
        .label("Window Lag")
        .initialize(async |context: &Context| {
            context.set(Lagging);
            Ok(())
        })
        .step(Step::new("Capture", async |context: &Context| context.sleep(Duration::from_secs(2)).await).identifier("capture"))
        .stream(lag_stream)
        .observer(lag_observer);
    Plugin::of::<()>("1.0.0")
        .identifier("execution")
        .setting(SettingDeclaration {
            minimum: None,
            maximum: None,
            name: "overflow_window".into(),
            title: "Overflow Window".into(),
            description: "Sampling time one window of the overflow stream covers".into(),
            default: Some(SettingValue {
                value: Some(setting_value::Value::Duration(prost_types::Duration { seconds: 3600, nanos: 0 })),
            }),
            ..Default::default()
        })
        .setting(SettingDeclaration {
            minimum: None,
            maximum: None,
            name: "sample_window".into(),
            title: "Sample Window".into(),
            description: "Sampling time one window of the setting stream covers".into(),
            default: Some(SettingValue {
                value: Some(setting_value::Value::Duration(prost_types::Duration { seconds: 0, nanos: 10_000_000 })),
            }),
            ..Default::default()
        })
        .suite(
            Suite::of::<()>()
                .identifier("execution.execution")
                .label("Execution")
                .template(boundary)
                .template(windows)
                .template(overflow)
                .template(lag),
        )
        .run()
}

/* === Tests === */
