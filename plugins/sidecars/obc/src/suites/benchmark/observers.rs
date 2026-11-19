/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Deterministic Heavy Observer Workload

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::device::{Bench, Supply};
use crate::records::charger;
use crate::{settings, suites};
use dokime::plugin::{Criterion, Observer, Step, Stream, Template, Window};
use dokime::templates::Context;
use dokime::{Document, Parameter, Result};

/* === Constants === */

pub const TEMPLATE: &str = "obc.benchmark.observers";
pub const SAMPLES: &str = "obc.benchmark.observers.samples";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Row)]
#[dokime(package = "obc")]
pub struct Computation {
    pub value: f64,
}
#[derive(dokime::Message)]
#[dokime(package = "obc")]
pub struct ComputationColumns {
    pub value: Vec<f64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn template() -> Template {
    let samples = Stream::<Computation>::new("samples", "Heavy Observer Output");
    let observer = Observer::window(&samples, compute).interval_setting(settings::SAMPLE_INTERVAL);
    charger::catalog(
        Template::of::<()>()
            .identifier("observers")
            .label("Heavy Observers")
            .parameter(Parameter::number("duration_s", "Capture Duration").unit("s").default(0.25).range(0.001..=100.0))
            .parameter(Parameter::count("iterations", "Iterations per Sample").default(4096_u64).range(1.0..=65536.0))
            .initialize(initialize)
            .step(Step::new("Capture Observer Output", capture).identifier("capture"))
            .criterion(Criterion::new("Observer Output Captured", captured).identifier("captured"))
            .cleanup(cleanup)
            .stream(samples)
            .observer(observer)
            .document(Document::new("Heavy Observers").id(format!("{TEMPLATE}.report"))),
    )
}
async fn initialize(ctx: &Context) -> Result<()> {
    suites::open(ctx, Supply::default(), None).await
}
async fn capture(ctx: &Context) -> Result<()> {
    let bench = ctx.get::<Bench>()?;
    let samples = bench.samples_for(ctx.parameter("duration_s")?)?;
    suites::wait(ctx, &bench, samples).await
}
async fn captured(ctx: &Context) -> Result<bool> {
    Ok(ctx.statistics("samples", "value", "").await?.is_some_and(|statistics| statistics.count > 0))
}
async fn cleanup(ctx: &Context) -> Result<()> {
    ctx.get::<Bench>()?.stop();
    Ok(())
}
async fn compute(ctx: &Context, window: &Window) -> Result<ComputationColumns> {
    let iterations: u64 = ctx.parameter("iterations")?;
    let mut value = Vec::new();
    for sample in 0..window.count {
        let mut sum = 0.0;
        for index in 1..=iterations {
            let angle = (window.sequence - 1 + sample) as f64 * 0.001 + index as f64;
            sum += angle.sin().powi(2) / index as f64;
        }
        value.push(sum);
    }
    ctx.get::<Bench>()?.advance_by(window.count);
    Ok(ComputationColumns { value })
}

/* === Private Functions === */

/* === Tests === */
