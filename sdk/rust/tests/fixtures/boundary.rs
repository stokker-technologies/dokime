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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Row)]
#[harmos(package = "execution")]
struct Reading {
    value: f64,
}

/// One Granted Window of Readings, Each Holding the Window's Sample Count.
#[derive(dokime::Message)]
#[harmos(package = "execution")]
struct ReadingColumns {
    value: Vec<f64>,
}

#[derive(dokime::Parameters)]
#[harmos(package = "execution")]
struct CaptureRows {
    rows: u64,
}

#[derive(Default)]
struct Published {
    count: AtomicU64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<()> {
    let boundary_stream = Stream::<Reading>::new("execution.boundary", "Boundary Samples");
    let boundary_observer = Observer::new(&boundary_stream, async |context: &Context| {
        // A Scope Samples Every Stream of the Plugin; Only the Boundary Template's Own Scope Counts Its Rows.
        let Ok(published) = context.get::<Published>() else {
            return Ok(Reading { value: 0.0 });
        };
        let index = published.count.fetch_add(1, Ordering::SeqCst);
        Ok(Reading { value: index as f64 })
    })
    .every(std::time::Duration::from_millis(1));
    let boundary = Template::new("execution.boundary", "Publication Boundary")
        .parameters(CaptureRows { rows: 1024 })
        .initialize(async |context: &Context| {
            context.set(Published::default());
            Ok(())
        })
        .step(Step::new("Capture", async |context: &Context| {
            // A Scoped Sleep Settles After Every Window Due by Its End, So It Spans the Remaining Rows.
            let rows = context.parameters::<CaptureRows>()?.rows;
            let published = context.get::<Published>()?;
            context.sleep(Duration::from_millis(rows.saturating_sub(published.count.load(Ordering::SeqCst)))).await
        }))
        .stream(boundary_stream)
        .observer(boundary_observer);
    // Two Window Observers at 1 kHz: One Takes the Host Window, One Its Plugin Duration Setting.
    let granted = async |_: &Context, window: &SampleWindow| {
        Ok(ReadingColumns {
            value: vec![window.count as f64; window.count as usize],
        })
    };
    let host_stream = Stream::<Reading>::new("execution.window.host", "Host Window Samples");
    let host_observer = Observer::window(&host_stream, granted).every(Duration::from_millis(1));
    let setting_stream = Stream::<Reading>::new("execution.window.setting", "Setting Window Samples");
    let setting_observer = Observer::window(&setting_stream, granted).every(Duration::from_millis(1)).window_setting("sample_window");
    let windows = Template::new("execution.windows", "Window Override")
        .parameters(CaptureRows { rows: 1000 })
        .step(Step::new("Capture", async |context: &Context| {
            context.sleep(Duration::from_millis(context.parameters::<CaptureRows>()?.rows)).await
        }))
        .stream(host_stream)
        .observer(host_observer)
        .stream(setting_stream)
        .observer(setting_observer);
    // One 1 MHz Window Holding More Rows Than the Shared Plugin Publication Pool Admits at Once: the Step's
    // 10 s Sleep Cuts the Hour-Long Window, While the Real-Clock Connection Demand Never Reaches Its End.
    let overflow_stream = Stream::<Reading>::new("execution.window.overflow", "Overflow Window Samples");
    let overflow_observer = Observer::window(&overflow_stream, async |_: &Context, window: &SampleWindow| {
        Ok(ReadingColumns {
            value: vec![0.0; window.count as usize],
        })
    })
    .every(Duration::from_micros(1))
    .window_setting("overflow_window");
    let overflow = Template::new("execution.overflow", "Window Overflow")
        .step(Step::new("Capture", async |context: &Context| context.sleep(Duration::from_secs(10)).await))
        .stream(overflow_stream)
        .observer(overflow_observer);
    // A 1 kHz Stream Whose Observer Answers Each Window Long After Its End, So a Real-Clock Host Falls Behind.
    let lag_stream = Stream::<Reading>::new("execution.window.lag", "Lagging Window Samples");
    let lag_observer = Observer::window(&lag_stream, async |_: &Context, window: &SampleWindow| {
        std::thread::sleep(Duration::from_millis(100));
        Ok(ReadingColumns {
            value: vec![0.0; window.count as usize],
        })
    })
    .every(Duration::from_millis(1));
    let lag = Template::new("execution.lag", "Window Lag")
        .step(Step::new("Capture", async |context: &Context| context.sleep(Duration::from_secs(2)).await))
        .stream(lag_stream)
        .observer(lag_observer);
    Plugin::new("execution", "1.0.0")
        .setting(dokime::SettingDeclaration {
            name: "overflow_window".into(),
            title: "Overflow Window".into(),
            description: "Sampling time one window of the overflow stream covers".into(),
            default: Some(dokime::SettingValue {
                value: Some(dokime::setting_value::Value::Duration(prost_types::Duration { seconds: 3600, nanos: 0 })),
            }),
        })
        .setting(dokime::SettingDeclaration {
            name: "sample_window".into(),
            title: "Sample Window".into(),
            description: "Sampling time one window of the setting stream covers".into(),
            default: Some(dokime::SettingValue {
                value: Some(dokime::setting_value::Value::Duration(prost_types::Duration { seconds: 0, nanos: 10_000_000 })),
            }),
        })
        .suite(Suite::new("execution", "Execution").template(boundary).template(windows).template(overflow).template(lag))
        .requires_host(HostOperation::TimeSleep)
        .serve()
}

/* === Tests === */
