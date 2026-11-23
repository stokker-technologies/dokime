/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plot Families, Plot Load and Thermal Survey

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Mutex, PoisonError};

// Third Party Imports

// Local Imports
use crate::device::{Bench, Power};
use crate::records::charger;
use crate::settings;
use crate::streams::measurement::{self, SAMPLES, charging};
use crate::suites::{self, parameters, read_supply, supply, timing};
use dokime::builders::Plot;
use dokime::plugin::{Criterion, Observer, Prompt, Record, Step, Stream, Template};
use dokime::templates::Context;
use dokime::{
    Card, ContourRender, DataExpression, Document, FieldReference, GridAxisEntry, GridChart, PlotStyle, PromptArchetype, Reduction,
    ReferenceBand, ReferenceLine, Result, Value, XyInterval, data_expression, model, value,
};

/* === Constants === */

pub const TEMPLATE: &str = "obc.benchmark.plots";
pub const SURVEY: &str = "obc.survey";
const STARTED: &str = "obc.benchmark.plots.started";
const COMPLETED: &str = "obc.benchmark.plots.completed";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Record)]
#[dokime(package = "obc")]
pub struct PlotConfirmation {
    pub accepted: bool,
}

#[derive(dokime::Row)]
#[dokime(package = "obc")]
pub struct Survey {
    pub sweep: f64,
    pub module: f64,
    pub sensor: f64,
    pub rise_k: f64,
}
struct Scan {
    point: Mutex<u64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn contracts() -> Record<PlotConfirmation> {
    Record::new("confirmation_record", "Plot Inputs Confirmed")
}
pub fn prompt() -> Prompt {
    Prompt::new("confirm", "Confirm Plot Inputs", "Do the Signal Maps and Traces Look Plausible?")
        .archetype(PromptArchetype::Confirmation)
        .action("yes", "Yes")
        .action("no", "No")
}
pub fn template() -> Template {
    let survey = Stream::<Survey>::new("survey", "Thermal Channel Survey");
    let observer = Observer::new(&survey, scan).interval_setting(settings::SAMPLE_INTERVAL);
    let template = parameters(Template::of::<()>().identifier("plots").label("Plot Families"), supply());
    let template = charger::catalog(template).record(contracts()).prompt(prompt());
    measurement::catalog(
        parameters(template, timing("Capture Duration", 0.25))
            .initialize(initialize)
            .step(Step::new("Capture Plot Inputs", capture).identifier("capture"))
            .step(Step::new("Confirm Plot Inputs", confirmation).identifier("confirmation"))
            .criterion(Criterion::new("Plot Inputs Captured", captured).identifier("captured"))
            .cleanup(cleanup)
            .stream(survey)
            .observer(observer)
            .document(document()),
    )
}
async fn initialize(ctx: &Context) -> Result<()> {
    ctx.set(Scan { point: Mutex::new(0) });
    suites::open(ctx, read_supply(ctx)?, Some(ctx.parameter("sample_interval_s")?)).await?;
    Ok(())
}
async fn capture(ctx: &Context) -> Result<()> {
    let bench = ctx.get::<Bench>()?;
    bench.start()?;
    ctx.record(&charger::started(), charger::Started { note: "Plot Capture Energized".into() }).await?;
    let samples = bench.samples_for(ctx.parameter("duration_s")?)?;
    bench.sequence(samples / 2, samples);
    suites::wait(ctx, &bench, samples).await
}
async fn confirmation(ctx: &Context) -> Result<()> {
    let answer = ctx.prompt(prompt(), &[]).await?;
    let accepted = answer == "Yes";
    let bench = ctx.get::<Bench>()?;
    ctx.record(
        &charger::completed(),
        charger::Completed {
            sample_count: bench.index(),
            prompt_action: answer.clone(),
            operator_note: String::new(),
            accepted: accepted && bench.fault().is_none(),
        },
    )
    .await?;
    ctx.record(&contracts(), PlotConfirmation { accepted }).await?;
    Ok(())
}
async fn captured(ctx: &Context) -> Result<bool> {
    let confirmed = charger::last(ctx, &contracts()).await?.is_some_and(|record| record.accepted);
    Ok(confirmed && suites::sample_count(ctx, SAMPLES, "").await? > 0)
}
async fn cleanup(ctx: &Context) -> Result<()> {
    ctx.get::<Bench>()?.stop();
    Ok(())
}
async fn scan(ctx: &Context) -> Result<Survey> {
    if !suites::captures(ctx, SURVEY) {
        return Ok(Survey::default());
    }
    let scan = ctx.get::<Scan>()?;
    let point = {
        let mut point = scan.point.lock().unwrap_or_else(PoisonError::into_inner);
        let next = *point;
        *point += 1;
        next
    };
    let sweep = (point / 32) as f64;
    let module = ((point % 32) / 8) as f64;
    let sensor = (point % 8) as f64;
    let bench = ctx.get::<Bench>()?;
    let charger = bench.charger();
    let load = if charger.power == Power::Running { charger.reading(bench.now_s()).input_power_w / 600.0 } else { 0.0 };
    Ok(Survey {
        sweep,
        module,
        sensor,
        rise_k: module * 1.5 + sensor * 0.4 + 12.0 * load * (1.0 - (-sweep / 20.0).exp()),
    })
}

/// All Ten Plot Families and One Hundred Independent Time Series.
pub fn document() -> Document {
    let mut voltage = Plot::time_series(FieldReference::new(SAMPLES).field("voltage_v"))
        .id("obc.benchmark.plots.signals")
        .label("AC Voltage")
        .unit("V")
        .message;
    voltage.style = PlotStyle::LinePoints as i32;
    voltage.y = vec![FieldReference::new(SAMPLES).field("voltage_v"), FieldReference::new(SAMPLES).field("voltage_rms_v")];
    voltage.marker_records = vec![STARTED.into(), COMPLETED.into()];
    voltage.reference_bands.push(ReferenceBand {
        center: Some(parameter("ac_voltage_v")),
        tolerance: Some(DataExpression {
            kind: Some(data_expression::Kind::Literal(Value { kind: Some(value::Kind::Number(23.0)) })),
            ..Default::default()
        }),
        label: "Nominal RMS ±23 V".into(),
        ..Default::default()
    });
    let timeline = model::Section::new("Signal Timeline").id("obc.benchmark.plots.timeline").plot(voltage);

    // Scattered Electrical Samples and the Independent Thermal Survey
    let mut surface = Plot::contour(FieldReference::new(SAMPLES).field("input_power_w"))
        .id("obc.benchmark.plots.power_map")
        .label("Input Power Map")
        .unit("W")
        .message;
    surface.x = Some(FieldReference::new(SAMPLES).field("voltage_v"));
    surface.y = vec![FieldReference::new(SAMPLES).field("current_a")];
    surface.reduction = Some(Reduction::Mean as i32);
    surface.render = ContourRender::Bands as i32;
    let trajectory = Plot::trajectory(FieldReference::new(SAMPLES).field("current_a"))
        .x(FieldReference::new(SAMPLES).field("voltage_v"))
        .id("obc.benchmark.plots.trajectory")
        .label("Electrical Trajectory")
        .unit("A")
        .message;
    let mut grid = Plot::grid_chart(FieldReference::new(SURVEY).field("rise_k"))
        .id("obc.benchmark.plots.channel_survey")
        .label("Channel Survey")
        .unit("K")
        .message;
    grid.reduction = Some(Reduction::Mean as i32);
    grid.grid = Some(GridChart {
        row: Some(FieldReference::new(SURVEY).field("module")),
        column: Some(FieldReference::new(SURVEY).field("sensor")),
        frame: Some(FieldReference::new(SURVEY).field("sweep")),
        rows: ["A", "B", "C", "D"].iter().enumerate().map(|(index, name)| axis(index, format!("Module {name}"))).collect(),
        columns: (0..8).map(|index| axis(index, format!("Sensor {}", index + 1))).collect(),
        summary_label: "Total Rise".into(),
        ..Default::default()
    });
    let maps = model::Section::new("Signal Maps")
        .id("obc.benchmark.plots.maps")
        .card(Card::plot(surface).columns(6))
        .card(Card::plot(trajectory).columns(6))
        .card(Card::plot(grid).columns(12));

    let voltages = Plot::histogram(FieldReference::new(SAMPLES).field("voltage_v"))
        .y(FieldReference::new(SAMPLES).field("voltage_v"))
        .id("obc.benchmark.plots.voltage_distribution")
        .label("Voltage Distribution")
        .unit("V")
        .message;
    let currents = Plot::histogram(FieldReference::new(SAMPLES).field("current_a"))
        .y(FieldReference::new(SAMPLES).field("current_a"))
        .id("obc.benchmark.plots.current_distribution")
        .label("Current Distribution")
        .unit("A")
        .message;
    let mut by_phase = Plot::box_plot(FieldReference::new(SAMPLES).field("voltage_v"))
        .id("obc.benchmark.plots.voltage_by_phase")
        .label("Voltage by Phase")
        .unit("V")
        .message;
    by_phase.series_by = Some(FieldReference::new(SAMPLES).field("phase"));
    let mut spectrum = Plot::frequency_series(FieldReference::new(SAMPLES).field("voltage_v"))
        .id("obc.benchmark.plots.spectrum")
        .label("Voltage Spectrum")
        .unit("V")
        .message;
    spectrum.sample_rate_hz = Some(1000.0);
    let distributions = model::Section::new("Signal Distributions")
        .id("obc.benchmark.plots.distributions")
        .plot(voltages)
        .plot(currents)
        .plot(by_phase)
        .plot(spectrum);

    let mut means = Plot::bar(FieldReference::new(SAMPLES).field("voltage_v"))
        .id("obc.benchmark.plots.mean_signal_by_phase")
        .label("Mean Signal by Phase")
        .unit("V")
        .message;
    means.x = Some(FieldReference::new(SAMPLES).field("phase"));
    means.y = vec![FieldReference::new(SAMPLES).field("voltage_v").mean(), FieldReference::new(SAMPLES).field("voltage_rms_v").mean()];
    let mut counts = Plot::bar(FieldReference::new(SAMPLES).field("sample_index"))
        .id("obc.benchmark.plots.sample_count_by_phase")
        .label("Sample Count by Phase")
        .unit("samples")
        .message;
    counts.x = Some(FieldReference::new(SAMPLES).field("phase"));
    counts.y = vec![FieldReference::new(SAMPLES).field("sample_index").count()];
    let mut share = Plot::pie(FieldReference::new(SAMPLES).field("sample_index"))
        .id("obc.benchmark.plots.phase_share")
        .label("Samples by Phase")
        .unit("samples")
        .message;
    share.x = Some(FieldReference::new(SAMPLES).field("phase"));
    share.y = vec![FieldReference::new(SAMPLES).field("sample_index").count()];
    share.hole = Some(0.5);
    let mut totals = Plot::bar(FieldReference::new(SAMPLES).field("sample_index"))
        .id("obc.benchmark.plots.phase_totals")
        .label("Phase Sample Totals")
        .unit("samples")
        .message;
    totals.x = Some(FieldReference::new(SAMPLES).field("phase"));
    totals.y = vec![FieldReference::new(SAMPLES).field("sample_index").count()];
    totals.pareto = true;
    let mut correlation = Plot::xy(FieldReference::new(SAMPLES).field("current_a"))
        .id("obc.benchmark.plots.signal_correlation")
        .label("Signal Correlation")
        .unit("A")
        .message;
    correlation.style = PlotStyle::Points as i32;
    correlation.x = Some(FieldReference::new(SAMPLES).field("voltage_v"));
    correlation.series_by = Some(FieldReference::new(SAMPLES).field("phase"));
    correlation.interval = Some(XyInterval::Sigma as i32);
    let breakdown = model::Section::new("Phase Breakdown")
        .id("obc.benchmark.plots.breakdown")
        .plot(means)
        .plot(counts)
        .plot(share)
        .plot(totals)
        .plot(correlation);

    let mut current = Plot::time_series(FieldReference::new(SAMPLES).field("current_a"))
        .id("obc.benchmark.plots.current")
        .label("AC Current")
        .unit("A")
        .message;
    current.y = vec![FieldReference::new(SAMPLES).field("current_a"), FieldReference::new(SAMPLES).field("current_rms_a")];
    current.reference_lines.push(ReferenceLine {
        value: Some(parameter("current_limit_a")),
        label: "RMS Current Limit".into(),
        ..Default::default()
    });
    let ac_current = model::Section::new("AC Current").id("obc.benchmark.plots.ac_current").plot(current);
    let evidence = model::Section::new("Evidence").id("obc.benchmark.plots.evidence").card(Card::records("obc.benchmark.plots.records"));

    let mut load = model::Section::new("Plot Load").id(format!("{TEMPLATE}.load"));
    for index in 0..100 {
        let plot = Plot::time_series(FieldReference::new(SAMPLES).field("voltage_v"))
            .id(format!("{TEMPLATE}.signal_{index}"))
            .label(format!("Signal {}", index + 1))
            .unit("V");
        load = load.plot(plot);
    }
    Document::new("Plot Families")
        .id("obc.benchmark.plots.report")
        .section(timeline)
        .section(maps)
        .section(distributions)
        .section(breakdown)
        .section(ac_current)
        .section(charging(TEMPLATE))
        .section(evidence)
        .section(load)
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

/* === Private Functions === */

/* === Tests === */
