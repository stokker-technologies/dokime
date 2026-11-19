/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Balanced Three-Phase Impedance, Efficiency and Power Factor

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
use dokime::plugin::{Criterion, Step, Template};
use dokime::templates::Context;
use dokime::{Document, Error, Parameter, Result, Value, value};

/* === Constants === */

pub const TEMPLATE: &str = "obc.steady_state.three_phase";
const SAMPLES: &str = "obc.steady_state.three_phase.measurement";

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
    let template = parameters(Template::of::<()>().identifier("three_phase").label("Three Phase"), supply());
    let template = charger::catalog(template).prompt(crate::prompts::catalog().into_iter().next().expect("Inspection Prompt"));
    let template = parameters(template, timing("Observation Duration", 0.05));
    measurement::catalog(
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
            .document(document()),
    )
}
pub fn document() -> Document {
    measurement::document(SAMPLES, TEMPLATE, "Three Phase")
}
async fn initialize(ctx: &Context) -> Result<()> {
    let mut supply = read_supply(ctx)?;
    supply.phases = 3;
    suites::open(ctx, supply, Some(ctx.parameter("sample_interval_s")?)).await?;
    ctx.set(Mutex::new(Decision { accepted: false }));
    Ok(())
}
async fn start(ctx: &Context) -> Result<()> {
    let bench = ctx.get::<Bench>()?;
    let supply = bench.charger().supply;
    let number = |value: f64| Value { kind: Some(value::Kind::Number(value)) };
    let text = |value: &str| Value {
        kind: Some(value::Kind::Text(value.into())),
    };
    // Describe the Device and the Bench
    suites::trace(ctx, "device.model", "Model", text("Simulated Onboard Charger"), "", "device").await?;
    suites::trace(ctx, "device.nominal_supply", "Nominal Supply", number(supply.voltage_rms_v), "V", "device").await?;
    suites::trace(ctx, "device.load_resistance", "Load Resistance", number(supply.load_ohm), "ohm", "device").await?;
    suites::trace(ctx, "device.load_reactance", "Load Reactance", number(supply.reactance_ohm), "ohm", "device").await?;
    suites::trace(ctx, "device.phases", "Supply Phases", number(f64::from(supply.phases)), "", "device").await?;
    suites::trace(ctx, "device.line_frequency", "Line Frequency", number(supply.frequency_hz), "Hz", "device").await?;
    suites::trace(ctx, "device.protection_limit", "Protection Limit", number(supply.current_limit_a), "A", "device").await?;
    suites::trace(ctx, "bench.application", "Application", text("Simulated AC Testbench"), "", "bench").await?;
    suites::trace(ctx, "bench.sample_interval", "Sample Interval", number(bench.step_s()), "s", "bench").await?;
    // Initialize and Energize the Charger
    ctx.record(&charger::initialized(), charger::Initialized {}).await?;
    bench.start()?;
    ctx.record(
        &charger::started(),
        Started {
            note: "Three Phase Supply Energized".into(),
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
    let voltage_rms_v = bench.charger().supply.voltage_rms_v;
    let event = charger::hold();
    ctx.record(
        &event,
        charger::HoldEvent {
            event: "hold_started".into(),
            elapsed_s: bench.now_s(),
            voltage_rms_v,
        },
    )
    .await?;
    // Capture the Steady-State Window
    suites::wait(ctx, &bench, samples).await?;
    ctx.record(
        &event,
        charger::HoldEvent {
            event: "hold_reached".into(),
            elapsed_s: bench.now_s(),
            voltage_rms_v,
        },
    )
    .await?;
    Ok(())
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
            prompt_action: answer.action.clone(),
            operator_note: answer.note.clone(),
            accepted: answer.accepted && bench.fault().is_none(),
        },
    )
    .await?;
    // Attach the Measurement and Operator Decision
    let supply = bench.charger().supply;
    let summary = serde_json::json!({
        "supply": { "voltage_rms_v": supply.voltage_rms_v, "load_ohm": supply.load_ohm,
            "reactance_ohm": supply.reactance_ohm, "frequency_hz": supply.frequency_hz,
            "phases": supply.phases, "current_limit_a": supply.current_limit_a },
        "sample_count": bench.index(), "prompt_action": answer.action, "operator_note": answer.note, "accepted": answer.accepted,
    });
    let bytes = serde_json::to_vec_pretty(&summary)
        .map_err(|error| Error::Invalid(format!("Attach Steady-State Summary: Correct the Measurement Data: {error}")))?;
    suites::attach(ctx, "steady-state-summary.json", "application/json", &bytes).await
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
    let apparent = 3.0 * reading.voltage_rms_v * reading.current_rms_a;
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
