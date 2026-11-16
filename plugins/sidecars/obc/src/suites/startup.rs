/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Charger Start-Up: Energize the Supply, Inspect the Charger, Observe Settling

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Mutex;

// Third Party Imports

// Local Imports
use super::{parameters, read_supply, sample_count, supply, timing};
use crate::catalog::{field, flag, plot, records as records_card, wide};
use crate::evidence::attach;
use crate::measurement::{Measurement, Survey, charging};
use crate::prompts::{STARTUP_CHECK, inspect};
use crate::records::{self, Checkpoint, Completed, Initialized, Started};
use crate::settings;
use crate::testbench::{self, Testbench};
use dokime::plugin::{Context, Criterion, Observer, Record, Step, Stream, Suite, Template};
use dokime::{
    ContourRender, DataExpression, Document, GridAxisEntry, GridChart, PlotKind, PlotStyle, Reduction, ReferenceBand, ReferenceLine,
    Result, Section, Severity, XyInterval, data_expression,
};

/* === Constants === */

pub const SUITE: &str = "obc.charger.startup";
pub const TEMPLATE: &str = "obc.charger.startup.sequence";
pub const SAMPLES: &str = "obc.charger.startup.samples";
pub const SURVEY: &str = "obc.charger.startup.survey";
pub const INITIALIZED: &str = "obc.charger.startup.initialized";
pub const STARTED: &str = "obc.charger.startup.started";
pub const CHECKPOINT: &str = "obc.charger.startup.checkpoint";
pub const COMPLETED: &str = "obc.charger.startup.completed";

// The Largest Reference Image the Evidence Step Attaches.
const MOST_IMAGE_BYTES: u64 = 4 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// The Operator's Inspection, Carried From the Prompt to the Evidence Step.
#[derive(Default)]
struct Decision {
    accepted: bool,
    action: String,
    note: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn contracts() -> (Record<Initialized>, Record<Started>, Record<Checkpoint>, Record<Completed>) {
    (
        Record::new(INITIALIZED, "Charger Initialized").description("The charger was initialized for this attempt."),
        Record::new(STARTED, "AC Supply Energized").description("The AC supply was energized."),
        Record::new(CHECKPOINT, "Review the Charger Status Indicator and Start-Up Trace")
            .description("The operator was asked to look at the bench."),
        Record::new(COMPLETED, "Start-Up Measurement and Operator Decision Captured")
            .description("The start-up measurement and the operator's decision, together."),
    )
}

pub fn suite() -> Suite {
    let samples =
        Stream::<Measurement>::new(SAMPLES, "Charger Input Samples").description("Measured AC voltage, current and supply timing.");
    let observer =
        Observer::new(&samples, async |ctx: &Context| testbench::measure(ctx, SAMPLES).await).interval_setting(settings::SAMPLE_INTERVAL);
    let survey = Stream::<Survey>::new(SURVEY, "Thermal Channel Survey").description("Module heatsink sensors scanned by the bench.");
    let surveyor =
        Observer::new(&survey, async |ctx: &Context| testbench::survey(ctx, SURVEY).await).interval_setting(settings::SAMPLE_INTERVAL);
    let template = Template::new(TEMPLATE, "Start-Up Sequence")
        .description("Measure the AC start-up inrush, inspect the charger, then observe settling.")
        .tag("device")
        .tag("startup");
    let template = parameters(template, supply(None, None));
    let template = parameters(template, timing("Observation Duration", 0.02));
    let template = parameters(
        template,
        [
            flag("emit_artifact", "Attach Measurement Summary", true),
            flag("attach_reference", "Attach reference.png From the Image Directory", false),
            flag("realtime", "Pace Samples in Real Time", false),
            flag("continuous", "Observe Until Stopped", false),
        ],
    );
    let template = template
        .initialize(initialize)
        .step(Step::new("Initialize and Start Charger", setup).identifier("setup"))
        .step(Step::new("Measure Start-Up Inrush", inrush).identifier("inrush"))
        .step(Step::new("Inspect Powered Charger", inspection).identifier("inspection"))
        .step(Step::new("Observe Settling and Attach Evidence", settle).identifier("settle"))
        .criterion(
            Criterion::new("Start-Up Trace and Operator Inspection", accepted)
                .identifier("acceptance")
                .description("Passed once the trace is unfaulted and, for a bounded run, the operator accepted it."),
        )
        .cleanup(cleanup)
        .stream(samples)
        .observer(observer)
        .stream(survey)
        .observer(surveyor)
        .document(document());
    Suite::new(SUITE, "Charger Start-Up")
        .description("Start the charger on AC, inspect it, and capture its settled measurements.")
        .tag("device")
        .tag("startup")
        .template(template)
}

/// The Start-Up Report Covers All Ten Plot Kinds Over the Captured Samples.
pub fn document() -> Document {
    let mut voltage = plot("obc.charger.startup.sequence.signals", "AC Voltage", PlotKind::TimeSeries, "V");
    voltage.style = PlotStyle::LinePoints as i32;
    voltage.y = vec![field(SAMPLES, "voltage_v"), field(SAMPLES, "voltage_rms_v")];
    voltage.marker_records = vec![STARTED.into(), CHECKPOINT.into(), COMPLETED.into()];
    voltage.reference_bands.push(ReferenceBand {
        center: Some(parameter("ac_voltage_v")),
        tolerance: Some(DataExpression {
            kind: Some(data_expression::Kind::Literal(dokime::Value {
                kind: Some(dokime::value::Kind::Number(23.0)),
            })),
            ..Default::default()
        }),
        label: "Nominal RMS ±23 V".into(),
        ..Default::default()
    });
    let timeline = Section::new("Signal Timeline").id("obc.charger.startup.sequence.timeline").plot(voltage);

    // Scattered Electrical Samples and the Independent Thermal Survey
    let mut surface = plot("obc.charger.startup.sequence.power_map", "Input Power Map", PlotKind::Contour, "W");
    surface.x = Some(field(SAMPLES, "voltage_v"));
    surface.y = vec![field(SAMPLES, "current_a")];
    surface.z = Some(field(SAMPLES, "input_power_w"));
    surface.reduction = Some(Reduction::Mean as i32);
    surface.render = ContourRender::Bands as i32;
    let mut trajectory = plot("obc.charger.startup.sequence.trajectory", "Electrical Trajectory", PlotKind::Trajectory, "A");
    trajectory.x = Some(field(SAMPLES, "voltage_v"));
    trajectory.y = vec![field(SAMPLES, "current_a")];
    let mut grid = plot("obc.charger.startup.sequence.channel_survey", "Channel Survey", PlotKind::GridChart, "K");
    grid.y = vec![field(SURVEY, "rise_k")];
    grid.reduction = Some(Reduction::Mean as i32);
    grid.grid = Some(GridChart {
        row: Some(field(SURVEY, "module")),
        column: Some(field(SURVEY, "sensor")),
        frame: Some(field(SURVEY, "sweep")),
        rows: ["A", "B", "C", "D"].iter().enumerate().map(|(index, name)| axis(index, format!("Module {name}"))).collect(),
        columns: (0..8).map(|index| axis(index, format!("Sensor {}", index + 1))).collect(),
        summary_label: "Total Rise".into(),
        ..Default::default()
    });
    let maps = Section::new("Signal Maps")
        .id("obc.charger.startup.sequence.maps")
        .card(wide(surface, 6))
        .card(wide(trajectory, 6))
        .card(wide(grid, 12));

    let mut voltages = plot("obc.charger.startup.sequence.voltage_distribution", "Voltage Distribution", PlotKind::Histogram, "V");
    voltages.x = Some(field(SAMPLES, "voltage_v"));
    voltages.y = vec![field(SAMPLES, "voltage_v")];
    let mut currents = plot("obc.charger.startup.sequence.current_distribution", "Current Distribution", PlotKind::Histogram, "A");
    currents.x = Some(field(SAMPLES, "current_a"));
    currents.y = vec![field(SAMPLES, "current_a")];
    let mut by_phase = plot("obc.charger.startup.sequence.voltage_by_phase", "Voltage by Phase", PlotKind::BoxPlot, "V");
    by_phase.series_by = Some(field(SAMPLES, "phase"));
    by_phase.y = vec![field(SAMPLES, "voltage_v")];
    let mut spectrum = plot("obc.charger.startup.sequence.spectrum", "Voltage Spectrum", PlotKind::FrequencySeries, "V");
    spectrum.y = vec![field(SAMPLES, "voltage_v")];
    spectrum.sample_rate_hz = Some(1000.0);
    let distributions = Section::new("Signal Distributions")
        .id("obc.charger.startup.sequence.distributions")
        .plot(voltages)
        .plot(currents)
        .plot(by_phase)
        .plot(spectrum);

    let mut means = plot("obc.charger.startup.sequence.mean_signal_by_phase", "Mean Signal by Phase", PlotKind::Bar, "V");
    means.x = Some(field(SAMPLES, "phase"));
    means.y = vec![field(SAMPLES, "voltage_v").mean(), field(SAMPLES, "voltage_rms_v").mean()];
    let mut counts = plot("obc.charger.startup.sequence.sample_count_by_phase", "Sample Count by Phase", PlotKind::Bar, "samples");
    counts.x = Some(field(SAMPLES, "phase"));
    counts.y = vec![field(SAMPLES, "sample_index").count()];
    let mut share = plot("obc.charger.startup.sequence.phase_share", "Samples by Phase", PlotKind::Pie, "samples");
    share.x = Some(field(SAMPLES, "phase"));
    share.y = vec![field(SAMPLES, "sample_index").count()];
    share.hole = Some(0.5);
    let mut totals = plot("obc.charger.startup.sequence.phase_totals", "Phase Sample Totals", PlotKind::Bar, "samples");
    totals.x = Some(field(SAMPLES, "phase"));
    totals.y = vec![field(SAMPLES, "sample_index").count()];
    totals.pareto = true;
    let mut correlation = plot("obc.charger.startup.sequence.signal_correlation", "Signal Correlation", PlotKind::Xy, "A");
    correlation.style = PlotStyle::Points as i32;
    correlation.x = Some(field(SAMPLES, "voltage_v"));
    correlation.y = vec![field(SAMPLES, "current_a")];
    correlation.series_by = Some(field(SAMPLES, "phase"));
    correlation.interval = Some(XyInterval::Sigma as i32);
    let breakdown = Section::new("Phase Breakdown")
        .id("obc.charger.startup.sequence.breakdown")
        .plot(means)
        .plot(counts)
        .plot(share)
        .plot(totals)
        .plot(correlation);

    let mut current = plot("obc.charger.startup.sequence.current", "AC Current", PlotKind::TimeSeries, "A");
    current.y = vec![field(SAMPLES, "current_a"), field(SAMPLES, "current_rms_a")];
    current.reference_lines.push(ReferenceLine {
        value: Some(parameter("current_limit_a")),
        label: "RMS Current Limit".into(),
        ..Default::default()
    });
    let ac_current = Section::new("AC Current").id("obc.charger.startup.sequence.ac_current").plot(current);
    let evidence =
        Section::new("Evidence").id("obc.charger.startup.sequence.evidence").card(records_card("obc.charger.startup.sequence.records"));

    Document::new("Start-Up Sequence")
        .id("obc.charger.startup.sequence.report")
        .section(timeline)
        .section(maps)
        .section(distributions)
        .section(breakdown)
        .section(ac_current)
        .section(charging(SAMPLES, TEMPLATE))
        .section(evidence)
}

/* === Private Functions === */

async fn initialize(ctx: &Context) -> Result<()> {
    // Open the Bench With the Model Advancing Per Sample or With the Host Clock
    let realtime: bool = ctx.parameter("realtime")?;
    let step_s = if realtime { None } else { Some(ctx.parameter("sample_interval_s")?) };
    Testbench::open(ctx, read_supply(ctx)?, step_s).await?;
    ctx.set(Mutex::new(Decision::default()));
    ctx.log(Severity::Info, "Charger Initialized").await
}

async fn setup(ctx: &Context) -> Result<()> {
    let (initialized, started, _, _) = contracts();
    ctx.record(&initialized, Initialized {}).await?;
    // Energize the Supply at the Current Model Time
    let bench = ctx.get::<Testbench>()?;
    let supply = {
        let mut state = bench.state();
        let now_s = state.now_s();
        state.charger.start(now_s)?;
        state.label = "running";
        state.charger.supply
    };
    let duration_s: f64 = ctx.parameter("duration_s")?;
    let note = format!("Energized at {} V RMS Into {} Ω for {duration_s} s.", supply.voltage_rms_v, supply.load_ohm);
    ctx.log(Severity::Info, note.clone()).await?;
    ctx.record(&started, Started { note }).await?;
    Ok(())
}

async fn inrush(ctx: &Context) -> Result<()> {
    let bench = ctx.get::<Testbench>()?;
    let total = bench.samples_for(ctx.parameter("duration_s")?)?;
    let checkpoint = (total / 2).max(1);
    bench.state().sequence = Some((checkpoint, total));
    if ctx.parameter::<bool>("continuous")? {
        // Observe Until a Host Stop or Cancellation Ends the Step
        loop {
            bench.wait(ctx, total).await?;
            let produced = bench.state().index;
            ctx.progress(produced, produced, "Observing Continuously").await?;
        }
    }
    let produced = bench.state().index;
    bench.wait(ctx, checkpoint.saturating_sub(produced)).await?;
    ctx.progress(checkpoint, total, "Measuring Start-Up Inrush").await
}

async fn inspection(ctx: &Context) -> Result<()> {
    let (_, _, checkpoint, _) = contracts();
    let produced = ctx.get::<Testbench>()?.state().index;
    let record = Checkpoint {
        sample_index: produced,
        prompt_type: STARTUP_CHECK.into(),
    };
    ctx.record(&checkpoint, record).await?;
    // Ask for Operator Inspection
    let answer = inspect(ctx, STARTUP_CHECK, "Charger Status Indicator Steady").await?;
    ctx.log(Severity::Info, format!("Inspection Answered: {}", answer.action)).await?;
    let decision = ctx.get::<Mutex<Decision>>()?;
    let mut decision = decision.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *decision = Decision {
        accepted: answer.accepted,
        action: answer.action,
        note: answer.note,
    };
    Ok(())
}

async fn settle(ctx: &Context) -> Result<()> {
    let (_, _, _, completed) = contracts();
    let bench = ctx.get::<Testbench>()?;
    let total = bench.samples_for(ctx.parameter("duration_s")?)?;
    let (accepted, action, note) = {
        let decision = ctx.get::<Mutex<Decision>>()?;
        let decision = decision.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        (decision.accepted, decision.action.clone(), decision.note.clone())
    };
    if accepted {
        let produced = bench.state().index;
        bench.wait(ctx, total.saturating_sub(produced)).await?;
        ctx.progress(total, total, "Observing Settling").await?;
    }
    let (produced, supply, unfaulted, step_s) = {
        let state = bench.state();
        (state.index, state.charger.supply, state.charger.fault.is_none(), state.step_s)
    };
    let record = Completed {
        sample_count: produced,
        prompt_action: action.clone(),
        operator_note: note.clone(),
        accepted: accepted && unfaulted,
    };
    ctx.record(&completed, record).await?;
    if ctx.parameter("emit_artifact")? {
        // Attach the Measurement Summary
        let summary = serde_json::json!({
            "supply": {
                "voltage_rms_v": supply.voltage_rms_v,
                "load_ohm": supply.load_ohm,
                "frequency_hz": supply.frequency_hz,
                "phase_deg": supply.phase_deg,
                "current_limit_a": supply.current_limit_a,
            },
            "sample_count": produced,
            "duration_s": produced as f64 * step_s,
            "prompt_action": action,
            "operator_note": note,
            "accepted": accepted,
        });
        attach(ctx, "startup-summary.json", "application/json", &serde_json::to_vec_pretty(&summary).unwrap_or_default()).await?;
    }
    if ctx.parameter("attach_reference")? {
        // Attach the Installed Reference Image
        let path = settings::images_directory(ctx).await?.join("reference.png");
        let size = std::fs::metadata(&path)
            .map_err(|error| dokime::Error::Invalid(format!("Attach Reference Image: Place reference.png in {}: {error}", path.display())))?
            .len();
        if size > MOST_IMAGE_BYTES {
            return Err(dokime::Error::Invalid("Attach Reference Image: Reduce reference.png to at Most 4 MiB".into()));
        }
        let bytes = std::fs::read(&path).map_err(|error| dokime::Error::Invalid(format!("Attach Reference Image: {error}")))?;
        attach(ctx, "reference.png", "image/png", &bytes).await?;
    }
    Ok(())
}

async fn accepted(ctx: &Context) -> Result<bool> {
    let bench = ctx.get::<Testbench>()?;
    let unfaulted = bench.state().charger.fault.is_none();
    if ctx.parameter("continuous")? {
        return Ok(unfaulted && sample_count(ctx, SAMPLES, "").await? > 0);
    }
    let (_, _, _, completed) = contracts();
    let Some(completed) = records::last(ctx, &completed, "").await? else {
        return Ok(false);
    };
    // Check / Number of Saved Rows and Operator Acceptance
    let total = bench.samples_for(ctx.parameter("duration_s")?)?;
    Ok(sample_count(ctx, SAMPLES, "").await? >= total && completed.accepted)
}

async fn cleanup(ctx: &Context) -> Result<()> {
    ctx.log(Severity::Info, "Charger Resources Released").await
}

fn parameter(name: &str) -> DataExpression {
    DataExpression {
        kind: Some(data_expression::Kind::Parameter(name.into())),
        ..Default::default()
    }
}

fn axis(index: usize, label: String) -> GridAxisEntry {
    GridAxisEntry {
        key: index.to_string(),
        label,
        order: index as i32,
        ..Default::default()
    }
}

/* === Tests === */
