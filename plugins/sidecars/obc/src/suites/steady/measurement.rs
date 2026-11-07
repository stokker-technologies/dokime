/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::atomic::{AtomicU64, Ordering};

// Third Party Imports

// Local Imports
use super::*;
use crate::domain::document::{self as system, Subscriptions};
use crate::domain::model::bench::{COARSEST_STEP_S, FINEST_STEP_S, SHORTEST_S, bench};
use crate::domain::model::measurement;
use crate::domain::model::measurement::Row;
use crate::domain::model::settings::STEADY_INTERVAL_MS;
use crate::domain::{Error, model::prompts::checkpoint};
use crate::operations::charger::*;
use dokime::definition::ParameterDefault;
use dokime::document::PlotStyle;
use dokime::domain::model::identity::TemplateId;
use dokime::expressions::telemetry;
use dokime::prelude::*;

/* === Constants === */

const STEADY_STREAM: &str = "obc.charger.steady.samples";

/* === Macros === */

bench!(Parameters);

/* === Type Aliases === */

/* === Structs === */

pub struct SteadyResources {
    device_id: u64,
    interval_s: f64,
    produced: AtomicU64,
}

pub struct Template {
    accepted: bool,
    prompt_action: String,
}

// The stream this template samples, a type of its own rather than a home on
// [`Row`]: four suites measure the same physical thing and each owns the
// stream its evidence travels, so the shape declares no home and every
// stream declares itself.
#[dokime::stream(
    id = "obc.charger.steady.samples",
    window = 4096,
    row = Row,
    label = "Steady-State AC Measurements"
, description = "Voltage and current of the execution-owned onboard charger; elapsed_s is simulated time.")]
pub struct Samples;

// Pushed by a step rather than measured on a cadence: the Charger reports these
// when something happens to it — the hold starts, the hold is reached — which
// is the device's schedule and not the engine's. The stream is declared by
// this row type, so its contract exists with nothing sampling it, and the step
// that observes the event pushes the row.
#[dokime::stream(
    id = "obc.charger.steady.events",
    window = 4096,
    label = "Steady-State Supply Events",
    description = "What happened to the Charger's supply, as the measuring step saw it."
)]
pub struct BenchEvent {
    #[row(role = state)]
    pub event: String,

    #[row(unit = "s")]
    pub elapsed_s: f64,

    #[row(unit = "V")]
    pub voltage_rms_v: f64,
}

// Records

#[dokime::record(
    id = "obc.charger.steady.initialized",
    label = "Steady-State Charger Initialized",
    description = "The Charger was initialized for this attempt."
)]
#[derive(Clone, Debug)]
pub struct Initialized {}

#[dokime::record(id = "obc.charger.steady.started", label = "Steady-State Charger Energized", description = "The supply was energized.")]
#[derive(Clone, Debug)]
pub struct Started {}

#[dokime::record(
    id = "obc.charger.steady.completed",
    label = "Steady-State Measurement and Operator Decision Captured",
    description = "The steady-state measurement and the operator's decision, together."
)]
#[derive(Clone, Debug)]
pub struct Completed {
    #[record(description = "How many rows the steady-state measurement produced.")]
    pub sample_count: u64,

    #[record(description = "Whether the measurement behaved the way this procedure required.")]
    pub accepted: bool,
}

#[dokime::parameters]
pub struct Parameters {
    #[parameter(
        label = "AC RMS Voltage",
        unit = "V",
        expansion = "cartesian",
        default = ParameterDefault::space([207.0, 230.0, 253.0]),
    )]
    pub ac_voltage_v: f64,

    #[parameter(
        label = "Load Impedance Magnitude",
        unit = "ohm",
        expansion = "cartesian",
        default = ParameterDefault::linspace(80.0, 120.0, 3),
    )]
    pub load_ohm: f64,

    #[parameter(label = "Supply Frequency", unit = "Hz", default = 50.0)]
    pub frequency_hz: f64,

    #[parameter(label = "Current Phase Lag", unit = "deg", default = 15.0)]
    pub phase_deg: f64,

    #[parameter(label = "RMS Current Limit", unit = "A", default = 5.0)]
    pub current_limit_a: f64,

    #[parameter(label = "Observation Duration", unit = "s", default = 0.05, min = SHORTEST_S)]
    pub duration_s: f64,

    #[parameter(
        label = "Simulation Sample Interval",
        unit = "s",
        default = 0.001,
        min = FINEST_STEP_S,
        max = COARSEST_STEP_S,
    )]
    pub sample_interval_s: f64,
}

/* === Enums === */

#[dokime::catalog]
pub enum Required {
    AskReadiness(crate::operations::AskReadiness),
    InitializeCharger(InitializeCharger),
    StartCharger(StartCharger),
    ReadCharger(ReadCharger),
    SampleCharger(SampleCharger),
    ReleaseCharger(ReleaseCharger),
    EncodeChargerSummary(EncodeChargerSummary),
}

#[dokime::catalog]
pub enum Prompts {
    SteadyCheck(checkpoint::SteadyCheck),
}

#[dokime::catalog]
pub enum EventStreams {
    BenchEvent(BenchEvent),
}

#[dokime::catalog]
pub enum Records {
    Initialized(Initialized),
    Started(Started),
    Completed(Completed),
}

#[dokime::catalog]
pub enum Streams {
    Samples(Samples),
}

/* === Traits === */

/* === Implementations === */

#[dokime::template(
    id = STEADY_TEMPLATE, label = "Steady-State Measurement",
    requires = Required,
    template_id = TemplateId::from_uuid(STEADY_TEMPLATE_ID).expect("steady template UUID"),
    tags = ["device", "steady"], prompts = Prompts,
    // Every host stream the bench offers: cpu, network, wifi, wifi latency,
    // wifi stream occupancy and bluetooth. Each is one running instance the
    // host shares with every other execution leasing it, ticking at the
    // cadence its own producer declared.
    subscribes = Subscriptions,
    // The stream this template pushes into. A row type states its own
    // contract but not which template owns it; naming it here is what binds
    // the two.
    streams = EventStreams,
    // What this template's steps publish. The contracts come off the structs in
    // this file, bound to this template.
    records = Records,
    attributes(path = "Testbench/Steady State", provider = "sidecar"),
    parameters = Parameters, description = "Sweep AC voltage and load while the SDK samples the steady state of one shared Charger.")]
impl Template {
    async fn initialize(_: &Parameters, ctx: &mut InitializationContext<'_>) -> Result<(Self, SteadyResources), Error> {
        // Check Bench Readiness
        let operation = crate::operations::AskReadiness {};
        let readiness = ctx.operation(operation).await?;
        if readiness.stopped {
            return Err(Error::Observation("the bench is stopped".into()));
        }

        // Initialize Charger
        let operation = InitializeCharger {
            supply: ctx.parameters().supply(),
            logging: false,
        };
        let device = ctx.operation(operation).await?;
        let template = Self {
            accepted: false,
            prompt_action: String::new(),
        };

        let shared = SteadyResources {
            device_id: device.device_id,
            interval_s: ctx.parameters().sample_interval_s,
            produced: AtomicU64::new(0),
        };
        Ok((template, shared))
    }

    #[dokime::step(label = "Initialize and Energize Charger")]
    async fn start(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        // Read Charger Identity
        let operation = ReadCharger { device_id: ctx.resources().device_id };
        let device = ctx.operation(operation).await?;
        let supply = device.supply;

        // === Device Under Test === //
        let mut section = TraceabilitySection::new("Device Under Test");
        let model = TraceabilityRow::text("Model", "ObcPlugin AC load");
        section.row(model);
        let voltage_rms_v = TraceabilityRow::number("Nominal Supply", supply.voltage_rms_v).unit("V").source("device");
        section.row(voltage_rms_v);
        let load_ohm = TraceabilityRow::number("Load Impedance", supply.load_ohm).unit("ohm").source("device");
        section.row(load_ohm);
        let frequency_hz = TraceabilityRow::number("Line Frequency", supply.frequency_hz).unit("Hz").source("device");
        section.row(frequency_hz);
        let current_limit_a = TraceabilityRow::number("Protection Limit", supply.current_limit_a).unit("A").source("device");
        section.row(current_limit_a);
        ctx.traceability().section(section);

        // === Bench === //
        let mut section = TraceabilitySection::new("Bench");
        let application = TraceabilityRow::text("Application", "ObcPlugin testbench");
        section.row(application);
        let interval_s = TraceabilityRow::number("Sample Interval", ctx.resources().interval_s).unit("s");
        section.row(interval_s);
        ctx.traceability().section(section);

        // Publish Initialized

        let record = Initialized {};
        ctx.publish(record).await;
        // Start Charger
        let operation = StartCharger { device_id: ctx.resources().device_id };
        ctx.operation(operation).await?;
        ctx.info("record=sidecar_started");
        let record = Started {};
        ctx.publish(record).await;
        Ok(())
    }

    #[dokime::step(
        label = "Measure AC Waveform",
        description = "Waits on the template's own observer, and reports what it saw around it.\nThe wait is for an exact number of rows rather than a length of time, so what the run measured does not depend on how quickly it got going. The two event rows travel a stream nothing samples: the step pushes them through a handle it holds open across the wait, beside its log lines rather than instead of them."
    )]
    async fn stream(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let wanted = ctx.parameters().sample_count()?;
        let interval_s = ctx.resources().interval_s;
        // Read Nominal Supply
        let operation = ReadCharger { device_id: ctx.resources().device_id };
        let device = ctx.operation(operation).await?;
        let voltage_rms_v = device.supply.voltage_rms_v;
        let events = ctx.stream::<BenchEvent>().open();
        let mut samples = ctx.listen::<Samples>();
        let started = ctx.resources().produced.load(Ordering::Acquire);
        let event = BenchEvent {
            event: "hold_started".into(),
            elapsed_s: 0.0,
            voltage_rms_v,
        };
        events.push(event).await;
        // Wait for Published Measurements
        while let Some(row) = samples.next().await {
            if (row.sample_index as u64 + 1).saturating_sub(started) >= wanted {
                break;
            }
        }
        // Publish Hold Completion
        let event = BenchEvent {
            event: "hold_reached".into(),
            elapsed_s: wanted as f64 * interval_s,
            voltage_rms_v,
        };
        events.push(event).await;
        ctx.info(format!("record=sidecar_held samples={wanted}"));
        Ok(())
    }

    #[dokime::step(label = "Inspect Steady-State Trace")]
    async fn prompt(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        ctx.info("record=sidecar_prompt_requested");
        // Ask for Operator Inspection
        let prompt = checkpoint::SteadyCheck {
            operator_note: "Waveform and charger status indicator stable".into(),
            ..Default::default()
        };
        let response = ctx.ask(prompt).await?;
        self.accepted = response.accepted;
        self.prompt_action = if self.accepted { "continue" } else { "decline" }.into();
        ctx.info(format!("record=sidecar_prompt_answered action={} accepted={}", self.prompt_action, self.accepted));
        Ok(())
    }

    #[dokime::step(label = "Attach Measurement Evidence")]
    async fn complete(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        // Read Final Charger State
        let operation = ReadCharger { device_id: ctx.resources().device_id };
        let device = ctx.operation(operation).await?;
        let produced = ctx.resources().produced.load(Ordering::Acquire);
        // Publish Completed
        let record = Completed {
            sample_count: produced,
            accepted: self.accepted && device.fault.is_none(),
        };
        ctx.publish(record).await;
        // Encode Measurement Summary
        let operation = EncodeChargerSummary {
            supply: device.supply,
            sample_count: produced,
            duration_s: None,
            prompt_action: self.prompt_action.clone(),
            operator_note: None,
            accepted: self.accepted,
        };
        let summary = ctx.operation(operation).await?.into_owned();
        ctx.artifacts().write("charger/steady-state-summary.json", summary.json)?;
        Ok(())
    }

    #[dokime::criterion(id = STEADY_ACCEPTANCE_CRITERION, label = STEADY_ACCEPTANCE_CRITERION_LABEL, description = "Passed once the sampler reached the case's sample count and the operator accepted the trace.")]
    fn accepted(ctx: &CriterionContext<'_>) -> Result<bool, Error> {
        let samples = ctx.samples::<Samples>();
        let records = ctx.records::<Completed>()?;
        let completed = records.last().ok_or_else(|| Error::MissingEvidence(Completed::RECORD_TYPE.into()))?;

        // Check / Number of Rows
        let number_ok = samples.len() as u64 >= ctx.parameters().sample_count()?;

        // Check / Operator Acceptance
        let accepted_ok = completed.accepted;
        Ok(number_ok && accepted_ok)
    }

    #[dokime::observer(interval_ms = STEADY_INTERVAL_MS, streams = Streams)]
    async fn samples(ctx: &mut ObserverContext<'_>) -> Result<(), Error> {
        // Count Published Rows
        let index = ctx.resources().produced.load(Ordering::Acquire);
        // Sample Charger
        let operation = SampleCharger {
            device_id: ctx.resources().device_id,
            elapsed_s: 0.1 + index as f64 * ctx.resources().interval_s,
        };
        let sample = ctx.operation(operation).await?;
        let row = measurement::row(index, sample.reading, "stream");
        ctx.stream::<Row>().into::<Samples>().push(row).await;
        ctx.resources().produced.store(index + 1, Ordering::Release);

        Ok(())
    }

    async fn cleanup(&mut self, ctx: &mut CleanupContext<'_>) -> Result<(), Error> {
        // Release Charger
        let operation = ReleaseCharger { device_id: ctx.resources().device_id };
        let result = ctx.operation(operation).await;
        if let Err(error) = &result {
            ctx.warn(format!("charger cleanup: {error}"));
        }
        result?;
        ctx.info("Charger resources released");
        Ok(())
    }

    fn document() -> Document {
        let mut execution = Section::new("AC Voltage");
        let mut plot = Plot::time_series("AC Voltage");
        plot.id("obc.charger.steady.measurement.execution.signals");
        plot.unit("V");
        plot.style(PlotStyle::LinePoints);
        plot.y(telemetry(STEADY_STREAM, "voltage_v"));
        plot.y(telemetry(STEADY_STREAM, "voltage_rms_v"));
        plot.marker_records([Started::marker(), Completed::marker()]);
        execution.plot(plot);
        let mut plot = Plot::time_series("Current");
        plot.id("obc.charger.steady.measurement.current");
        plot.unit("A");
        plot.y(telemetry(STEADY_STREAM, "current_a"));
        execution.plot(plot);

        let mut document = Document::new("default", "Steady-State Measurement");
        document.section(execution);

        let mut charging = Section::new("Battery Charging");
        for (column, label, unit) in [
            (Row::column_dc_voltage_v(), "Battery Voltage", "V"),
            (Row::column_dc_current_a(), "Charging Current", "A"),
            (Row::column_input_power_w(), "AC Input Power", "W"),
        ] {
            let mut plot = Plot::time_series(label);
            plot.id(format!("obc.charger.steady.{}", column.field()));
            plot.unit(unit);
            plot.y(column.on(STEADY_STREAM));
            charging.plot(plot);
        }
        document.section(charging);
        let section = system::section(STEADY_TEMPLATE);
        document.section(section);

        document
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
