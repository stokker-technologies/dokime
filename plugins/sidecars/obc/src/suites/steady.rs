/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Charger Steady State: Sweep AC Voltage and Load While the Bench Samples Steady Operation

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Mutex;

// Third Party Imports

// Local Imports
use super::{parameters, read_supply, sample_count, supply, timing};
use crate::catalog::{field, plot, records as records_card, sweep};
use crate::evidence::{attach, trace};
use crate::measurement::{Measurement, charging};
use crate::prompts::{STEADY_CHECK, inspect};
use crate::records::{self, Completed, HoldEvent, Initialized, Started};
use crate::settings;
use crate::testbench::{self, Testbench};
use dokime::plugin::{Context, Criterion, Observer, Record, Step, Stream, Suite, Template};
use dokime::{Document, PlotKind, PlotStyle, Result, Section, Severity, Value, value};

/* === Constants === */

pub const SUITE: &str = "obc.charger.steady";
pub const TEMPLATE: &str = "obc.charger.steady.measurement";
pub const SAMPLES: &str = "obc.charger.steady.samples";
pub const INITIALIZED: &str = "obc.charger.steady.initialized";
pub const STARTED: &str = "obc.charger.steady.started";
pub const HOLD: &str = "obc.charger.steady.hold";
pub const COMPLETED: &str = "obc.charger.steady.completed";

// Steady Samples Start Past the Inrush Envelope.
const SETTLED_S: f64 = 0.1;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
struct Decision {
    accepted: bool,
    action: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn contracts() -> (Record<Initialized>, Record<Started>, Record<HoldEvent>, Record<Completed>) {
    (
        Record::new(INITIALIZED, "Steady-State Charger Initialized").description("The charger was initialized for this attempt."),
        Record::new(STARTED, "Steady-State Charger Energized").description("The supply was energized."),
        Record::new(HOLD, "Steady-State Supply Event").description("What happened to the supply, as the measuring step saw it."),
        Record::new(COMPLETED, "Steady-State Measurement and Operator Decision Captured")
            .description("The steady-state measurement and the operator's decision, together."),
    )
}

pub fn suite() -> Suite {
    let samples = Stream::<Measurement>::new(SAMPLES, "Steady-State AC Measurements")
        .description("Voltage and current of the execution-owned charger; elapsed_s is simulated time.");
    let observer =
        Observer::new(&samples, async |ctx: &Context| testbench::measure(ctx, SAMPLES).await).interval_setting(settings::SAMPLE_INTERVAL);
    let template = Template::new(TEMPLATE, "Steady-State Measurement")
        .description("Sweep AC voltage and load while the bench samples the steady state of one charger per case.")
        .tag("device")
        .tag("steady");
    let voltage = sweep("ac_voltage_v", "AC RMS Voltage", "V", &[207.0, 230.0, 253.0]);
    let load = sweep("load_ohm", "Load Impedance Magnitude", "ohm", &[80.0, 100.0, 120.0]);
    let template = parameters(template, supply(Some(voltage), Some(load)));
    let template = parameters(template, timing("Observation Duration", 0.05))
        .initialize(initialize)
        .step(Step::new("Initialize and Energize Charger", start).identifier("start"))
        .step(Step::new("Measure AC Waveform", hold).identifier("hold"))
        .step(Step::new("Inspect Steady-State Trace", inspection).identifier("inspection"))
        .step(Step::new("Attach Measurement Evidence", complete).identifier("complete"))
        .criterion(
            Criterion::new("Steady-State Trace and Operator Acceptance", accepted)
                .identifier("acceptance")
                .description("Passed once the sampler reached the case's sample count and the operator accepted the trace."),
        )
        .cleanup(cleanup)
        .stream(samples)
        .observer(observer)
        .document(document());
    Suite::new(SUITE, "Charger Steady State")
        .description("Measure the charger's steady-state input across AC voltage and load sweeps.")
        .tag("device")
        .tag("steady")
        .template(template)
}

pub fn document() -> Document {
    let mut voltage = plot("obc.charger.steady.measurement.execution.signals", "AC Voltage", PlotKind::TimeSeries, "V");
    voltage.style = PlotStyle::LinePoints as i32;
    voltage.y = vec![field(SAMPLES, "voltage_v"), field(SAMPLES, "voltage_rms_v")];
    voltage.marker_records = vec![STARTED.into(), HOLD.into(), COMPLETED.into()];
    let mut current = plot("obc.charger.steady.measurement.current", "Current", PlotKind::TimeSeries, "A");
    current.y = vec![field(SAMPLES, "current_a")];
    let signals = Section::new("AC Voltage").id("obc.charger.steady.measurement.signals").plot(voltage).plot(current);
    let evidence = Section::new("Evidence").id("obc.charger.steady.measurement.evidence").card(records_card("obc.charger.steady.records"));
    Document::new("Steady-State Measurement")
        .id("obc.charger.steady.measurement.report")
        .section(signals)
        .section(charging(SAMPLES, TEMPLATE))
        .section(evidence)
}

/* === Private Functions === */

async fn initialize(ctx: &Context) -> Result<()> {
    Testbench::open(ctx, read_supply(ctx)?, Some(ctx.parameter("sample_interval_s")?)).await?;
    ctx.get::<Testbench>()?.state().origin_s = SETTLED_S;
    ctx.set(Mutex::new(Decision::default()));
    Ok(())
}

async fn start(ctx: &Context) -> Result<()> {
    let (initialized, started, _, _) = contracts();
    let bench = ctx.get::<Testbench>()?;
    let (supply, step_s) = {
        let state = bench.state();
        (state.charger.supply, state.step_s)
    };
    // Device Under Test and Bench
    let number = |value: f64| Value { kind: Some(value::Kind::Number(value)) };
    let text = |value: &str| Value {
        kind: Some(value::Kind::Text(value.into())),
    };
    trace(ctx, "device.model", "Model", text("Simulated Onboard Charger"), "", "device").await?;
    trace(ctx, "device.nominal_supply", "Nominal Supply", number(supply.voltage_rms_v), "V", "device").await?;
    trace(ctx, "device.load_impedance", "Load Impedance", number(supply.load_ohm), "ohm", "device").await?;
    trace(ctx, "device.line_frequency", "Line Frequency", number(supply.frequency_hz), "Hz", "device").await?;
    trace(ctx, "device.protection_limit", "Protection Limit", number(supply.current_limit_a), "A", "device").await?;
    trace(ctx, "bench.application", "Application", text("Simulated AC Testbench"), "", "bench").await?;
    trace(ctx, "bench.sample_interval", "Sample Interval", number(step_s), "s", "bench").await?;
    ctx.record(&initialized, Initialized {}).await?;
    // Energize the Charger
    {
        let mut state = bench.state();
        state.charger.start(0.0)?;
        state.label = "stream";
    }
    ctx.record(
        &started,
        Started {
            note: "Charger energized for the steady-state hold.".into(),
        },
    )
    .await?;
    Ok(())
}

async fn hold(ctx: &Context) -> Result<()> {
    let (_, _, event, _) = contracts();
    let bench = ctx.get::<Testbench>()?;
    let wanted = bench.samples_for(ctx.parameter("duration_s")?)?;
    let (voltage_rms_v, started_s) = {
        let state = bench.state();
        (state.charger.supply.voltage_rms_v, state.now_s())
    };
    let record = HoldEvent {
        event: "hold_started".into(),
        elapsed_s: started_s,
        voltage_rms_v,
    };
    ctx.record(&event, record).await?;
    // Wait for the Case's Number of Samples
    bench.wait(ctx, wanted).await?;
    let reached_s = bench.state().now_s();
    let record = HoldEvent {
        event: "hold_reached".into(),
        elapsed_s: reached_s,
        voltage_rms_v,
    };
    ctx.record(&event, record).await?;
    ctx.log(Severity::Info, format!("Held for {wanted} Samples")).await
}

async fn inspection(ctx: &Context) -> Result<()> {
    let answer = inspect(ctx, STEADY_CHECK, "Waveform and Charger Status Indicator Stable").await?;
    ctx.log(Severity::Info, format!("Inspection Answered: {}", answer.action)).await?;
    let decision = ctx.get::<Mutex<Decision>>()?;
    *decision.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Decision {
        accepted: answer.accepted,
        action: answer.action,
    };
    Ok(())
}

async fn complete(ctx: &Context) -> Result<()> {
    let (_, _, _, completed) = contracts();
    let bench = ctx.get::<Testbench>()?;
    let (produced, supply, unfaulted) = {
        let state = bench.state();
        (state.index, state.charger.supply, state.charger.fault.is_none())
    };
    let (accepted, action) = {
        let decision = ctx.get::<Mutex<Decision>>()?;
        let decision = decision.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        (decision.accepted, decision.action.clone())
    };
    let record = Completed {
        sample_count: produced,
        prompt_action: action.clone(),
        operator_note: String::new(),
        accepted: accepted && unfaulted,
    };
    ctx.record(&completed, record).await?;
    let summary = serde_json::json!({
        "supply": {
            "voltage_rms_v": supply.voltage_rms_v,
            "load_ohm": supply.load_ohm,
            "frequency_hz": supply.frequency_hz,
            "phase_deg": supply.phase_deg,
            "current_limit_a": supply.current_limit_a,
        },
        "sample_count": produced,
        "prompt_action": action,
        "accepted": accepted,
    });
    attach(ctx, "steady-state-summary.json", "application/json", &serde_json::to_vec_pretty(&summary).unwrap_or_default()).await
}

async fn accepted(ctx: &Context) -> Result<bool> {
    let (_, _, _, completed) = contracts();
    let Some(completed) = records::last(ctx, &completed, "").await? else {
        return Ok(false);
    };
    // Check / Number of Rows and Operator Acceptance
    let wanted = ctx.get::<Testbench>()?.samples_for(ctx.parameter("duration_s")?)?;
    Ok(sample_count(ctx, SAMPLES, "").await? >= wanted && completed.accepted)
}

async fn cleanup(ctx: &Context) -> Result<()> {
    ctx.log(Severity::Info, "Charger Resources Released").await
}

/* === Tests === */
