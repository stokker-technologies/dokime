/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Single-Phase Impedance, Efficiency and Power Factor

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Mutex, PoisonError};

// Third Party Imports

// Local Imports
use crate::device::Bench;
use crate::prompts::inspection::{CHECK, inspect};
use crate::records::charger::{self, Completed, Started};
use crate::streams::measurement;
use crate::suites::{self, parameters, read_supply, supply, timing};
use dokime::plugin::{Context, Criterion, Step, Template};
use dokime::{Document, Parameter, Result};

/* === Constants === */

pub const TEMPLATE: &str = "obc.steady_state.single_phase";
const SAMPLES: &str = measurement::SAMPLES;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Decision {
    accepted: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn template() -> Template {
    let template = parameters(Template::new(TEMPLATE, "Single Phase"), supply());
    let template = parameters(template, timing("Observation Duration", 0.05));
    template
        .parameter(Parameter::number("minimum_efficiency", "Minimum Efficiency").default(0.9).range(0.0..=1.0))
        .parameter(Parameter::number("minimum_power_factor", "Minimum Power Factor").default(0.9).range(0.0..=1.0))
        .initialize(initialize)
        .step(Step::new("Energize Charger", start).identifier("start"))
        .step(Step::new("Measure Steady Operation", measure).identifier("measure"))
        .step(Step::new("Inspect Steady Operation", inspection).identifier("inspection"))
        .criterion(Criterion::new("Conversion Efficiency", efficiency).identifier("efficiency"))
        .criterion(Criterion::new("Input Power Factor", power_factor).identifier("power_factor"))
        .cleanup(cleanup)
        .uses_stream(SAMPLES)
        .document(document())
}
pub fn document() -> Document {
    measurement::document(SAMPLES, TEMPLATE, "Single Phase")
}
async fn initialize(ctx: &Context) -> Result<()> {
    let mut supply = read_supply(ctx)?;
    supply.phases = 1;
    suites::open(ctx, supply, Some(ctx.parameter("sample_interval_s")?)).await?;
    ctx.set(Mutex::new(Decision { accepted: false }));
    Ok(())
}
async fn start(ctx: &Context) -> Result<()> {
    ctx.record(&charger::initialized(), charger::Initialized {}).await?;
    ctx.get::<Bench>()?.start()?;
    ctx.record(
        &charger::started(),
        Started {
            note: "Single Phase Supply Energized".into(),
        },
    )
    .await?;
    Ok(())
}
async fn measure(ctx: &Context) -> Result<()> {
    let bench = ctx.get::<Bench>()?;
    // Settle the Inrush Before the Acceptance Window
    let settling = bench.samples_for(0.1)?;
    suites::wait(ctx, &bench, settling).await?;
    let samples = bench.samples_for(ctx.parameter("duration_s")?)?;
    suites::wait(ctx, &bench, samples).await
}
async fn inspection(ctx: &Context) -> Result<()> {
    let answer = inspect(ctx, CHECK, "Waveform and Charger Status Indicator Stable").await?;
    let decision = ctx.get::<Mutex<Decision>>()?;
    *decision.lock().unwrap_or_else(PoisonError::into_inner) = Decision { accepted: answer.accepted };
    let bench = ctx.get::<Bench>()?;
    ctx.record(
        &charger::completed(),
        Completed {
            sample_count: bench.index(),
            prompt_action: answer.action,
            operator_note: answer.note,
            accepted: answer.accepted && bench.fault().is_none(),
        },
    )
    .await?;
    Ok(())
}
async fn efficiency(ctx: &Context) -> Result<bool> {
    let bench = ctx.get::<Bench>()?;
    let reading = bench.charger().reading(bench.now_s());
    let measured = reading.dc_voltage_v * reading.dc_current_a / reading.input_power_w;
    let decision = ctx.get::<Mutex<Decision>>()?;
    let accepted = decision.lock().unwrap_or_else(PoisonError::into_inner).accepted;
    Ok(accepted && measured.is_finite() && measured >= ctx.parameter("minimum_efficiency")?)
}
async fn power_factor(ctx: &Context) -> Result<bool> {
    let bench = ctx.get::<Bench>()?;
    let reading = bench.charger().reading(bench.now_s());
    let apparent = 1.0 * reading.voltage_rms_v * reading.current_rms_a;
    let measured = reading.input_power_w / apparent;
    let decision = ctx.get::<Mutex<Decision>>()?;
    let accepted = decision.lock().unwrap_or_else(PoisonError::into_inner).accepted;
    Ok(accepted && measured.is_finite() && measured >= ctx.parameter("minimum_power_factor")?)
}
async fn cleanup(ctx: &Context) -> Result<()> {
    ctx.get::<Bench>()?.stop();
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
