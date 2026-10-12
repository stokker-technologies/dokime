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
use std::{path::PathBuf, time::Duration};
// Third Party Imports

// Local Imports
use super::super::parameters::Startup as Inputs;
use super::*;
use crate::{error::Error, prompts::checkpoint};
use dokime::core::TemplateId;
use dokime::document::{self, PlotDefinition, PlotStyle};
use dokime::expressions::telemetry;
use dokime::host::ToolConnectionAcquired;
use dokime::prelude::*;
use mini_runtime::device::{Device, Power};

/* === Constants === */
const STARTUP_STREAM: &str = "mini.charger.startup.samples";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// A type of its own rather than a home on [`InputMeasurement`]: four suites measure
// the same physical thing and each owns the stream its evidence travels, so
// the shape declares no home and every stream declares itself.
#[dokime::stream(
    id = "mini.charger.startup.samples",
    window = 4096,
    row = super::super::measurement::InputMeasurement,
    label = "Charger Input Samples"
, description = "Measured AC voltage, current, and supply timing.")]
pub struct Samples;

pub struct Startup {
    produced: u64,
    device: Device,
    accepted: bool,
    prompt_action: String,
    operator_note: String,
}

/* === Records === */

#[dokime::record(
    id = "mini.charger.startup.initialized",
    label = "Charger Initialized",
    description = "The Charger was initialized for this attempt."
)]
#[derive(Clone, Debug)]
pub struct Initialized {}

#[dokime::record(
    id = "mini.charger.startup.started",
    label = "AC Supply Energized",
    description = "The AC supply was energized."
)]
#[derive(Clone, Debug)]
pub struct Started {}

#[dokime::record(
    id = "mini.charger.startup.checkpoint",
    label = "Review the charger status indicator and startup trace",
    description = "The operator was asked to look at the bench."
)]
#[derive(Clone, Debug)]
pub struct Checkpoint {
    #[record(description = "How far the measurement had got when the operator was asked.")]
    pub sample_index: u64,
    #[record(description = "The question the operator was asked.")]
    pub prompt_type: String,
}

#[dokime::record(
    id = "mini.charger.startup.completed",
    label = "Startup measurement and operator decision captured",
    description = "The startup measurement and the operator's decision, together."
)]
#[derive(Clone, Debug)]
pub struct Completed {
    #[record(description = "How many rows the startup measurement produced.")]
    pub sample_count: u64,
    #[record(description = "What the operator chose.")]
    pub prompt_action: String,
    #[record(description = "What the operator wrote.")]
    pub operator_note: String,
    #[record(description = "Whether the startup behaved the way this procedure required.")]
    pub accepted: bool,
}

/* === Enums === */
#[dokime::catalog]
pub enum Prompts {
    StartupCheck(checkpoint::StartupCheck),
}

#[dokime::catalog]
pub enum Required {
    AskReadiness(crate::operations::readiness::AskReadiness),
    ListConnections(crate::operations::connections::ListConnections),
}

#[dokime::catalog]
pub enum Subscriptions {
    ToolConnectionAcquired(ToolConnectionAcquired),
    System(host::System),
    Network(host::Network),
    Wifi(host::Wifi),
    WifiLatency(host::WifiLatency),
    WifiChannelOccupancy(host::WifiChannelOccupancy),
    Bluetooth(host::Bluetooth),
}

#[dokime::catalog]
pub enum Records {
    Initialized(Initialized),
    Started(Started),
    Checkpoint(Checkpoint),
    Completed(Completed),
}

#[dokime::catalog]
pub enum Streams {
    Samples(Samples),
}

/* === Traits === */

/* === Implementations === */
#[dokime::template(
    id = STARTUP_TEMPLATE, label = "Power Up the Charger",
    template_id = TemplateId::from_uuid(STARTUP_TEMPLATE_ID).expect("startup template UUID"),
    tags = ["device", "startup"], prompts = Prompts, graceful_stop = "continuous",
    requires = Required,
    // Every host stream the bench offers, named by type. Each declares its
    // own cadence, so the fast counters and the slow radio scans run at the
    // rates their producers said they could sustain rather than at one the
    // template guessed.
    subscribes = Subscriptions,
    records = Records,
    streams = Streams,
    attributes(path = "Testbench/Startup", provider = "sidecar"),
    parameters = Inputs, description = "Measure AC startup inrush, inspect the Charger, then observe settling.")]
impl Startup {
    async fn initialize(
        inputs: &Inputs,
        ctx: &mut InitializationContext<'_>,
    ) -> Result<Self, Error> {
        // The route rather than the handle: a template names no plugin at all,
        // so `ctx.call` selecting the explicit route type is the only road
        // to it — and the one that holds the answer to the timeout the route
        // declared, which is what a host call would get.
        if ctx
            .call::<crate::operations::readiness::AskReadiness>(())
            .await?
            .stopped
        {
            return Err(Error::Device(mini_runtime::Error::Stopped));
        }
        Ok(Self {
            produced: 0,
            device: Device::initialize(inputs.supply()).map_err(Error::Device)?,
            accepted: false,
            prompt_action: String::new(),
            operator_note: String::new(),
        })
    }

    fn document() -> Document {
        startup_document()
    }

    #[dokime::step(
        label = "Wait for the Unit",
        description = "Waits for the unit to say it is on the bench before energizing it.\nNothing this execution can produce: the connection is reported by whoever owns the fixture — another plugin, the host, or an operator — into this run's scope, and the run has no business guessing. The wait is bounded, and the SDK's timeout becomes this plugin's own error through the `From` on `Error::Awaited`.\nIt also reports what the bench was told *before* this run started, which is what the process-wide connection listener accumulated across every execution and every gap between them."
    )]
    async fn connected(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        if !ctx.parameters().await_connection {
            return Ok(());
        }
        let connections = ctx
            .call::<crate::operations::connections::ListConnections>(())
            .await?;
        ctx.info(format!(
            "record=bench_connections count={}",
            connections.resources.len()
        ));
        // The host's own record type, so the step names no id and reads no
        // detail map: the unit's identity is a field of the fact it waited for.
        let connected = ctx
            .listen::<ToolConnectionAcquired>()
            .timeout(Duration::from_secs_f64(
                ctx.parameters().connection_budget_s,
            ))
            .next()
            .await
            .transpose()?
            .expect("a live subscription yields until it is dropped");
        ctx.info(format!(
            "record=connected resource={}",
            connected.resource_id
        ));
        Ok(())
    }

    #[dokime::step(label = "Initialize and Power Charger")]
    async fn setup(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        ctx.publish(Initialized {}).await;
        self.device.start().map_err(Error::Device)?;
        ctx.info(format!(
            "record=started voltage_rms_v={} load_ohm={} duration_s={}",
            self.device.supply.voltage_rms_v,
            self.device.supply.load_ohm,
            ctx.parameters().duration_s
        ));
        ctx.publish(Started {}).await;
        Ok(())
    }

    #[dokime::step(label = "Measure Startup Inrush")]
    async fn publish(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let inputs = ctx.parameters();
        let (total, interval_s) = (inputs.sample_count()?, inputs.sample_interval_s);
        let checkpoint = (total / 2).max(1);
        let continuous = inputs.continuous;
        let realtime = inputs.realtime || continuous;
        let interval = Duration::from_secs_f64(interval_s);
        if continuous {
            loop {
                publish_runtime_sample(
                    ctx,
                    interval_s,
                    &mut self.device,
                    self.produced,
                    total,
                    checkpoint,
                )
                .await?;
                self.produced += 1;
                tokio::time::sleep(interval).await;
            }
        } else {
            let remaining = sample_range(self.produced, checkpoint)?;
            for index in ctx.progress("Measuring startup inrush", remaining) {
                let index = index as u64;
                publish_runtime_sample(ctx, interval_s, &mut self.device, index, total, checkpoint)
                    .await?;
                self.produced = index + 1;
                if realtime {
                    tokio::time::sleep(interval).await;
                }
            }
        }
        ctx.info(format!("record=checkpoint samples={}", self.produced));
        Ok(())
    }

    #[dokime::step(label = "Inspect Powered Charger")]
    async fn prompt(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        ctx.publish(Checkpoint {
            sample_index: self.produced,
            prompt_type: "mini.charger.startup.check".into(),
        })
        .await;
        ctx.info("record=prompt_requested");
        let response = ctx
            .ask(checkpoint::StartupCheck {
                operator_note: "charger status indicator steady".into(),
                ..Default::default()
            })
            .await?;
        self.accepted = response.accepted;
        self.operator_note = response.operator_note;
        self.prompt_action = if self.accepted { "continue" } else { "decline" }.into();
        ctx.info(format!(
            "record=prompt_answered action={} accepted={}",
            self.prompt_action, self.accepted
        ));
        Ok(())
    }

    #[dokime::step(label = "Observe Settling and Attach Evidence")]
    async fn finalize(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let inputs = ctx.parameters();
        let (total, interval_s) = (inputs.sample_count()?, inputs.sample_interval_s);
        let checkpoint = (total / 2).max(1);
        if self.accepted {
            let remaining = sample_range(self.produced, total)?;
            for index in ctx.progress("Observing settling", remaining) {
                let index = index as u64;
                publish_runtime_sample(ctx, interval_s, &mut self.device, index, total, checkpoint)
                    .await?;
                self.produced = index + 1;
                if inputs.realtime {
                    tokio::time::sleep(Duration::from_secs_f64(interval_s)).await;
                }
            }
        }
        ctx.publish(Completed {
            sample_count: self.produced,
            prompt_action: self.prompt_action.clone(),
            operator_note: self.operator_note.clone(),
            accepted: self.accepted && self.device.fault.is_none(),
        })
        .await;
        if inputs.emit_artifact {
            let summary = serde_json::to_vec_pretty(&serde_json::json!({
                "supply": self.device.supply,
                "sample_count": self.produced,
                "duration_s": self.produced as f64 * interval_s,
                "prompt_action": self.prompt_action,
                "operator_note": self.operator_note,
                "accepted": self.accepted,
            }))?;
            ctx.artifacts()
                .write("charger/startup-summary.json", summary)?;
        }
        if inputs.attach_reference {
            let directory: PathBuf = ctx
                .parameter("images_directory")
                .map_err(|error| Error::Image(error.to_string()))?;
            let path = directory.join("reference.png");
            // The image is an installed asset this step never holds in memory:
            // the size is read from the entry, and the bytes stream past.
            let size = std::fs::metadata(&path)
                .map_err(|error| Error::Image(error.to_string()))?
                .len();
            if size > 4 * 1024 * 1024 {
                return Err(Error::Image("reference.png exceeds 4 MiB".into()));
            }
            ctx.artifacts()
                .import(&path, "charger/reference.png")
                .map_err(|error| Error::Image(error.to_string()))?;
        }
        Ok(())
    }

    async fn cleanup(&mut self, ctx: &mut CleanupContext<'_>) -> Result<(), Error> {
        self.device.stop();
        ctx.info("Charger resources released");
        Ok(())
    }

    #[dokime::criterion(
        id = "mini.charger.startup.acceptance",
        label = "Startup trace and operator inspection",
        description = "Passed once the trace is unfaulted and, for a bounded run, the operator accepted it."
    )]
    fn accepted(ctx: &CriterionContext<'_>) -> Result<bool, Error> {
        let continuous = ctx.parameters().continuous;
        let samples = ctx.samples::<Samples>();
        if continuous {
            let Some(last) = samples.last() else {
                return Ok(false);
            };
            return Ok(last.phase != "fault");
        }
        Ok(samples.len() as u64 >= ctx.parameters().sample_count()?
            && super::super::support::decided(ctx.records::<Completed>(), |run| run.accepted)?)
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn publish_runtime_sample<T, O, I>(
    ctx: &mut StepContext<'_, T, O, I>,
    interval_s: f64,
    device: &mut Device,
    index: u64,
    total: u64,
    checkpoint: u64,
) -> Result<(), Error> {
    let reading = device.sample(index as f64 * interval_s);
    let phase = if device.power == Power::Faulted {
        "fault"
    } else if index == 0 {
        "start"
    } else if index + 1 == total {
        "complete"
    } else if index + 1 == checkpoint {
        "checkpoint"
    } else if index < checkpoint {
        "before_prompt"
    } else {
        "after_prompt"
    };
    super::super::measurement::publish::<Samples, _, _, _>(ctx, index, reading, phase).await;

    Ok(())
}

fn sample_range(start: u64, end: u64) -> Result<std::ops::Range<usize>, Error> {
    let start = usize::try_from(start)
        .map_err(|_| Error::Observation("sample index exceeds platform capacity".into()))?;
    let end = usize::try_from(end)
        .map_err(|_| Error::Observation("sample count exceeds platform capacity".into()))?;
    Ok(start..end)
}

fn signal(field: &str) -> dokime::expressions::DataExpression {
    telemetry(STARTUP_STREAM, field)
}

fn voltage_distribution(identifier: &str) -> PlotDefinition {
    let mut plot = Plot::histogram("Voltage Distribution");
    plot.id(identifier);
    plot.group(INPUT_DISTRIBUTION_GROUP);
    plot.unit("V");
    plot.y(signal("voltage_v"));

    plot
}

fn current_distribution(identifier: &str) -> PlotDefinition {
    let mut plot = Plot::histogram("Current Distribution");
    plot.id(identifier);
    plot.group(INPUT_DISTRIBUTION_GROUP);
    plot.unit("A");
    plot.y(signal("current_a"));

    plot
}

fn voltage_by_phase(identifier: &str) -> PlotDefinition {
    let mut plot = Plot::box_summary("Voltage By Phase");
    plot.id(identifier);
    plot.group(INPUT_DISTRIBUTION_GROUP);
    plot.unit("V");
    plot.series_by(signal("phase"));
    plot.y(signal("voltage_v"));

    plot
}

fn mean_signal_by_phase(identifier: &str) -> PlotDefinition {
    let mut plot = Plot::bar("Mean Signal By Phase");
    plot.id(identifier);
    plot.group(INPUT_PHASE_GROUP);
    plot.unit("V");
    plot.x(signal("phase"));
    plot.y(signal("voltage_v").mean());
    plot.y(signal("voltage_rms_v").mean());

    plot
}

fn sample_count_by_phase(identifier: &str) -> PlotDefinition {
    let mut plot = Plot::bar("Sample Count By Phase");
    plot.id(identifier);
    plot.group(INPUT_PHASE_GROUP);
    plot.unit("samples");
    plot.x(signal("phase"));
    plot.y(signal("sample_index").count());

    plot
}

fn signal_correlation(identifier: &str) -> PlotDefinition {
    let mut plot = Plot::xy_scatter("Signal Correlation");
    plot.id(identifier);
    plot.group(INPUT_PHASE_GROUP);
    plot.style(PlotStyle::Points);
    plot.x(signal("voltage_v"));
    plot.y(signal("current_a"));
    plot.series_by(signal("phase"));

    plot
}

fn startup_document() -> Document {
    let mut document = Document::new("default", "Power Up the Charger");

    // === What One Execution Shows === //

    let mut voltage = Plot::time_series("AC Voltage");
    voltage.id("mini.charger.startup.sequence.signals");
    voltage.group(CHARGER_TIMELINE_GROUP);
    voltage.unit("V");
    voltage.style(PlotStyle::LinePoints);
    voltage.y(signal("voltage_v"));
    voltage.y(signal("voltage_rms_v"));
    // The prompt markers are the host's own records, so they are named by id;
    // this suite's are named by their type.
    voltage.marker_records([
        Started::marker(),
        document::marker("prompt.request.form"),
        document::marker("prompt.response.submitted"),
        Completed::marker(),
    ]);

    let mut timeline = Section::new(
        "mini.charger.startup.sequence.document.timeline",
        "Signal Timeline",
    );
    timeline.plot(voltage);
    document.section(timeline);

    let mut distributions = Section::new(
        "mini.charger.startup.sequence.document.distributions",
        "Signal Distributions",
    );
    distributions.plot(voltage_distribution(
        "mini.charger.startup.sequence.voltage_distribution",
    ));
    distributions.plot(current_distribution(
        "mini.charger.startup.sequence.current_distribution",
    ));
    distributions.plot(voltage_by_phase(
        "mini.charger.startup.sequence.voltage_by_phase",
    ));
    document.section(distributions);

    let mut phase_breakdown = Section::new(
        "mini.charger.startup.sequence.document.phase_breakdown",
        "Phase Breakdown",
    );
    phase_breakdown.plot(mean_signal_by_phase(
        "mini.charger.startup.sequence.mean_signal_by_phase",
    ));
    phase_breakdown.plot(sample_count_by_phase(
        "mini.charger.startup.sequence.sample_count_by_phase",
    ));
    phase_breakdown.plot(signal_correlation(
        "mini.charger.startup.sequence.signal_correlation",
    ));
    document.section(phase_breakdown);

    // === The Current, And The Host Around It === //

    let mut current = Plot::time_series("AC Current");
    current.id("mini.charger.startup.sequence.current");
    current.unit("A");
    current.y(signal("current_a"));
    current.y(signal("current_rms_a"));

    let mut ac_current = Section::new("mini.charger.startup.current", "AC Current");
    ac_current.plot(current);
    document.section(ac_current);

    // Fold in the host's standard telemetry document sections so the
    // host telemetry this run streams (system, network, WiFi, Bluetooth) renders
    // in the telemetry view alongside the charger signals.
    for section in document::host::sections(false) {
        document.section(section);
    }

    document
}

/* === Tests === */
