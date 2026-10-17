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
use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};
use std::time::Duration;
// Third Party Imports

// Local Imports
use super::*;
use crate::suites::common::bench::{COARSEST_STEP_S, FINEST_STEP_S, SHORTEST_S, bench};
use crate::suites::common::measurement::InputMeasurement;
use crate::suites::common::system::{self, SystemInputs};
use crate::suites::common::{readiness, verdicts};
use crate::{error::Error, prompts::checkpoint};
use dokime_domain::model::identity::TemplateId;
use dokime_runtime::definition::ParameterDefault;
use dokime_runtime::document::{PlotDefinition, PlotStyle};
use dokime_runtime::expressions::telemetry;
use dokime_runtime::prelude::*;
use obc_runtime::device::Device;

/* === Constants === */
const STEADY_SAMPLE_POLL: Duration = Duration::from_millis(1);

const STEADY_STREAM: &str = "obc.charger.steady.samples";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct SteadyResources {
    device: Mutex<Device>,
    interval_s: f64,
    produced: AtomicU64,
}

pub struct Steady {
    accepted: bool,
    prompt_action: String,
}

// The stream this template samples, a type of its own rather than a home on
// [`InputMeasurement`]: four suites measure the same physical thing and each owns the
// stream its evidence travels, so the shape declares no home and every
// stream declares itself.
#[dokime_runtime::stream(
    id = "obc.charger.steady.samples",
    window = 4096,
    row = InputMeasurement,
    label = "Steady-State AC Measurements"
, description = "Voltage and current of the execution-owned onboard charger; elapsed_s is simulated time.")]
pub struct Samples;

// Pushed by a step rather than measured on a cadence: the Charger reports these
// when something happens to it — the hold starts, the hold is reached — which
// is the device's schedule and not the engine's. The stream is declared by
// this row type, so its contract exists with nothing sampling it, and the step
// that observes the event pushes the row.
#[dokime_runtime::stream(
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

/* === Records === */

#[dokime_runtime::record(
    id = "obc.charger.steady.initialized",
    label = "Steady-state Charger initialized",
    description = "The Charger was initialized for this attempt."
)]
#[derive(Clone, Debug)]
pub struct Initialized {}

#[dokime_runtime::record(
    id = "obc.charger.steady.started",
    label = "Steady-state Charger energized",
    description = "The supply was energized."
)]
#[derive(Clone, Debug)]
pub struct Started {}

#[dokime_runtime::record(
    id = "obc.charger.steady.completed",
    label = "Steady-state measurement and operator decision captured",
    description = "The steady-state measurement and the operator's decision, together."
)]
#[derive(Clone, Debug)]
pub struct Completed {
    #[record(description = "How many rows the steady-state measurement produced.")]
    pub sample_count: u64,
    #[record(description = "Whether the measurement behaved the way this procedure required.")]
    pub accepted: bool,
}

#[dokime_runtime::parameters]
pub struct Parameters {
    #[parameter(
        label = "AC Voltage (RMS)",
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
#[dokime_runtime::catalog]
pub enum Required {
    AskReadiness(crate::operations::readiness::AskReadiness),
}

#[dokime_runtime::catalog]
pub enum Prompts {
    SteadyCheck(checkpoint::SteadyCheck),
}

#[dokime_runtime::catalog]
pub enum Streams {
    BenchEvent(BenchEvent),
}

#[dokime_runtime::catalog]
pub enum Records {
    Initialized(Initialized),
    Started(Started),
    Completed(Completed),
}

#[dokime_runtime::catalog]
pub enum ObserverStreams {
    Samples(Samples),
}

/* === Traits === */

/* === Implementations === */
bench!(Parameters);

#[dokime_runtime::template(
    id = STEADY_TEMPLATE, label = "Charging Input Measurement",
    requires = Required,
    template_id = TemplateId::from_uuid(STEADY_TEMPLATE_ID).expect("steady template UUID"),
    tags = ["device", "steady"], prompts = Prompts,
    // Every host stream the bench offers: cpu, network, wifi, wifi latency,
    // wifi stream occupancy and bluetooth. Each is one running instance the
    // host shares with every other execution leasing it, ticking at the
    // cadence its own producer declared.
    subscribes = SystemInputs,
    // The stream this template pushes into. A row type states its own
    // contract but not which template owns it; naming it here is what binds
    // the two.
    streams = Streams,
    // What this template's steps publish. The contracts come off the structs in
    // this file, bound to this template.
    records = Records,
    attributes(path = "Testbench/Steady State", provider = "sidecar"),
    parameters = Parameters, description = "Sweep AC voltage and load while the SDK samples one shared Charger.")]
impl Steady {
    async fn initialize(
        _: &Parameters,
        ctx: &mut InitializationContext<'_>,
    ) -> Result<(Self, SteadyResources), Error> {
        readiness::ready(ctx).await?;
        let template = Self {
            accepted: false,
            prompt_action: String::new(),
        };

        let shared = SteadyResources {
            device: Mutex::new(
                Device::initialize(ctx.parameters().supply()).map_err(Error::Device)?,
            ),
            interval_s: ctx.parameters().sample_interval_s,
            produced: AtomicU64::new(0),
        };
        Ok((template, shared))
    }

    async fn traceability(
        &self,
        ctx: &TraceabilityContext<'_>,
    ) -> Result<Vec<TraceabilitySection>, Error> {
        let supply = ctx
            .resources()
            .device
            .lock()
            .map_err(|_| Error::DeviceResourcePoisoned)?
            .supply;

        // Device Under Test
        let mut dut = TraceabilitySection::new("dut", "Device Under Test");
        dut.row(TraceabilityRow::text("model", "Model", "ObcPlugin AC load"));
        dut.row(
            TraceabilityRow::number(
                "nominal_voltage_rms_v",
                "Nominal Supply",
                supply.voltage_rms_v,
            )
            .unit("V")
            .source("device"),
        );
        dut.row(
            TraceabilityRow::number("load_ohm", "Load Impedance", supply.load_ohm)
                .unit("ohm")
                .source("device"),
        );
        dut.row(
            TraceabilityRow::number("frequency_hz", "Line Frequency", supply.frequency_hz)
                .unit("Hz")
                .source("device"),
        );
        dut.row(
            TraceabilityRow::number(
                "current_limit_a",
                "Protection Limit",
                supply.current_limit_a,
            )
            .unit("A")
            .source("device"),
        );

        // Bench
        let mut bench = TraceabilitySection::new("bench", "Bench");
        bench.row(TraceabilityRow::text(
            "application",
            "Application",
            "ObcPlugin testbench",
        ));
        bench.row(
            TraceabilityRow::number(
                "sample_interval_s",
                "Sample Interval",
                ctx.resources().interval_s,
            )
            .unit("s"),
        );

        Ok(vec![dut, bench])
    }

    fn document() -> Document {
        steady_document()
    }

    #[dokime_runtime::step(label = "Initialize and Energize Charger")]
    async fn start(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        ctx.publish(Initialized {}).await;
        ctx.resources()
            .device
            .lock()
            .map_err(|_| Error::DeviceResourcePoisoned)?
            .start()
            .map_err(Error::Device)?;
        ctx.info("record=sidecar_started");
        ctx.publish(Started {}).await;
        Ok(())
    }

    #[dokime_runtime::step(
        label = "Measure AC Waveform",
        description = "Waits on the template's own observer, and reports what it saw around it.\nThe wait is for an exact number of rows rather than a length of time, so what the run measured does not depend on how quickly it got going. The two event rows travel a stream nothing samples: the step pushes them through a handle it holds open across the wait, beside its log lines rather than instead of them."
    )]
    async fn stream(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let wanted = ctx.parameters().sample_count()?;
        let interval_s = ctx.resources().interval_s;
        let events = ctx.stream::<BenchEvent>().open();
        events
            .push(BenchEvent {
                event: "hold_started".into(),
                elapsed_s: 0.0,
                voltage_rms_v: nominal(ctx)?,
            })
            .await;
        // An exact number of rows rather than a length of time, so what the run
        // measured does not depend on how quickly it got going. Counted off the
        // sampler's own counter rather than awaited by type: `InputMeasurement` is one
        // shape serving four of this plugin's streams, so it names no home and
        // a subscription by type would not know which stream was meant.
        while ctx.resources().produced.load(Ordering::Acquire) < wanted {
            tokio::time::sleep(STEADY_SAMPLE_POLL).await;
        }
        events
            .push(BenchEvent {
                event: "hold_reached".into(),
                elapsed_s: wanted as f64 * interval_s,
                voltage_rms_v: nominal(ctx)?,
            })
            .await;
        ctx.info(format!("record=sidecar_held samples={wanted}"));
        Ok(())
    }

    #[dokime_runtime::step(label = "Inspect Steady-State Trace")]
    async fn prompt(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        ctx.info("record=sidecar_prompt_requested");
        let response = ctx
            .ask(checkpoint::SteadyCheck {
                operator_note: "Waveform and charger status indicator stable".into(),
                ..Default::default()
            })
            .await?;
        self.accepted = response.accepted;
        self.prompt_action = if self.accepted { "continue" } else { "decline" }.into();
        ctx.info(format!(
            "record=sidecar_prompt_answered action={} accepted={}",
            self.prompt_action, self.accepted
        ));
        Ok(())
    }

    #[dokime_runtime::step(label = "Attach Measurement Evidence")]
    async fn complete(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let device = ctx
            .resources()
            .device
            .lock()
            .map_err(|_| Error::DeviceResourcePoisoned)?
            .clone();
        let produced = ctx.resources().produced.load(Ordering::Acquire);
        ctx.publish(Completed {
            sample_count: produced,
            accepted: self.accepted && device.fault.is_none(),
        })
        .await;
        let summary = serde_json::to_vec_pretty(&serde_json::json!({
            "supply": device.supply,
            "sample_count": produced,
            "prompt_action": self.prompt_action,
            "accepted": self.accepted,
        }))?;
        ctx.artifacts()
            .write("charger/steady-state-summary.json", summary)?;
        Ok(())
    }

    async fn cleanup(&mut self, ctx: &mut CleanupContext<'_>) -> Result<(), Error> {
        ctx.resources()
            .device
            .lock()
            .map_err(|_| Error::DeviceResourcePoisoned)?
            .stop();
        ctx.info("Charger resources released");
        Ok(())
    }

    #[dokime_runtime::criterion(id = STEADY_ACCEPTANCE_CRITERION, label = STEADY_ACCEPTANCE_CRITERION_LABEL, description = "Passed once the sampler reached the case's sample count and the operator accepted the trace.")]
    fn accepted(ctx: &CriterionContext<'_>) -> Result<bool, Error> {
        Ok(
            ctx.samples::<Samples>().len() as u64 >= ctx.parameters().sample_count()?
                && verdicts::decided(ctx.records::<Completed>(), |run| run.accepted)?,
        )
    }

    #[dokime_runtime::observer(interval_ms = 50, streams = ObserverStreams)]
    async fn samples(ctx: &mut ObserverContext<'_>) -> Result<(), Error> {
        let row = reading(ctx.resources())?;
        ctx.stream::<InputMeasurement>()
            .into::<Samples>()
            .push(row)
            .await;

        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn reading(resources: &SteadyResources) -> Result<InputMeasurement, Error> {
    let index = resources.produced.fetch_add(1, Ordering::AcqRel);
    let elapsed_s = 0.1 + index as f64 * resources.interval_s;
    let measured = resources
        .device
        .lock()
        .map_err(|_| Error::DeviceResourcePoisoned)?
        .sample(elapsed_s);

    Ok(InputMeasurement {
        sample_index: index as i64,
        elapsed_s,
        voltage_v: measured.voltage_v,
        current_a: measured.current_a,
        voltage_rms_v: measured.voltage_rms_v,
        current_rms_a: measured.current_rms_a,
        frequency_hz: measured.frequency_hz,
        phase_deg: measured.phase_deg,
        input_power_w: measured.input_power_w,
        dc_voltage_v: measured.dc_voltage_v,
        dc_current_a: measured.dc_current_a,
        phase: "stream".into(),
    })
}

fn nominal(ctx: &StepContext<'_, Steady, SteadyResources, Parameters>) -> Result<f64, Error> {
    Ok(ctx
        .resources()
        .device
        .lock()
        .map_err(|_| Error::DeviceResourcePoisoned)?
        .supply
        .voltage_rms_v)
}

fn steady_voltage(identifier: &str) -> PlotDefinition {
    let mut plot = Plot::time_series("AC Voltage");
    plot.id(identifier);
    plot.unit("V");
    plot.style(PlotStyle::LinePoints);
    plot.y(telemetry(STEADY_STREAM, "voltage_v"));
    plot.y(telemetry(STEADY_STREAM, "voltage_rms_v"));
    plot.marker_records([Started::marker(), Completed::marker()]);

    plot
}

fn steady_current() -> PlotDefinition {
    let mut plot = Plot::time_series("Current");
    plot.id("obc.charger.steady.measurement.current");
    plot.unit("A");
    plot.y(telemetry(STEADY_STREAM, "current_a"));

    plot
}

fn steady_document() -> Document {
    let mut execution = Section::new("AC Voltage");
    execution.plot(steady_voltage(
        "obc.charger.steady.measurement.execution.signals",
    ));
    execution.plot(steady_current());

    let mut document = Document::new("default", "Charging Input Measurement");
    document.section(execution);

    let mut charging = Section::new("Battery Charging");
    for (column, label, unit) in [
        (
            InputMeasurement::column_dc_voltage_v(),
            "Battery Voltage",
            "V",
        ),
        (
            InputMeasurement::column_dc_current_a(),
            "Charging Current",
            "A",
        ),
        (
            InputMeasurement::column_input_power_w(),
            "AC Input Power",
            "W",
        ),
    ] {
        let mut plot = Plot::time_series(label);
        plot.id(format!("obc.charger.steady.{}", column.field()));
        plot.unit(unit);
        plot.y(column.on(STEADY_STREAM));
        charging.plot(plot);
    }
    document.section(charging);
    document.section(system::section(STEADY_TEMPLATE));

    document
}

/* === Tests === */
