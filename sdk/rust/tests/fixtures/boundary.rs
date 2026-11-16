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
        if context.execution_id().is_empty() {
            return Ok(Reading { value: 0.0 });
        }
        let published = context.get::<Published>()?;
        let index = published.count.fetch_add(1, Ordering::SeqCst);
        Ok(Reading { value: index as f64 })
    })
    .every_ms(1);
    let boundary = Template::new("execution.boundary", "Publication Boundary")
        .parameters(CaptureRows { rows: 1024 })
        .initialize(async |context: &Context| {
            context.set(Published::default());
            Ok(())
        })
        .step(Step::new("Capture", async |context: &Context| {
            // A Scoped Sleep Settles After Every Tick Due by Its End, So It Spans the Remaining Rows.
            let rows = context.parameters::<CaptureRows>()?.rows;
            let published = context.get::<Published>()?;
            context.sleep(Duration::from_millis(rows.saturating_sub(published.count.load(Ordering::SeqCst)))).await
        }))
        .stream(boundary_stream)
        .observer(boundary_observer);
    Plugin::new("execution", "1.0.0")
        .suite(Suite::new("execution", "Execution").template(boundary))
        .requires_host(HostOperation::TimeSleep)
        .serve()
}

/* === Tests === */
