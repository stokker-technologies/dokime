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
use super::*;
use crate::suites::common::bench::{COARSEST_STEP_S, FINEST_STEP_S, SHORTEST_S, bench};
use crate::suites::common::{measurement, system, verdicts};
use crate::{error::Error, prompts::checkpoint};
use dokime::document::{self, PlotDefinition, PlotStyle};
use dokime::domain::model::files::wire::FileWatchPolicy;
use dokime::domain::model::files::wire::FileWatchSelection;
use dokime::domain::model::files::wire::FileWatchSource;
use dokime::domain::model::files::wire::FileWatchSourcePayload;
use dokime::domain::model::files::wire::LineMatcher;
use dokime::domain::model::files::wire::LocalWatchSource;
use dokime::domain::model::identity::TemplateId;
use dokime::expressions::telemetry;
use dokime::harmos::operations::Host;
use dokime::host::ToolConnectionAcquired;
use dokime::operations::files::WatchFile;
use dokime::prelude::*;
use obc_runtime::device::{Device, Power};

/* === Constants === */

const CONNECTION_BUDGET_S: f64 = 30.0;
const STARTUP_STREAM: &str = "obc.charger.startup.samples";

/* === Macros === */

bench!(Parameters);

/* === Type Aliases === */

/* === Structs === */

// A type of its own rather than a home on [`InputMeasurement`]: four suites measure
// the same physical thing and each owns the stream its evidence travels, so
// the shape declares no home and every stream declares itself.
#[dokime::stream(
    id = "obc.charger.startup.samples",
    window = 4096,
    row = measurement::InputMeasurement,
    label = "Charger Input Samples"
, description = "Measured AC voltage, current, and supply timing.")]
pub struct Samples;

// Read from the charger's own log file, not measured: a matcher turns each
// status line into one row.
#[dokime::stream(
    id = "obc.charger.startup.log",
    window = 256,
    label = "Charger Log",
    description = "Input power the charger reported in its log file."
)]
pub struct ChargerLog {
    #[row(unit = "s")]
    pub elapsed_s: f64,

    #[row(unit = "W")]
    pub input_power_w: f64,
}

pub struct Startup {
    produced: u64,
    device: Device,
    log: PathBuf,
    accepted: bool,
    prompt_action: String,
    operator_note: String,
}

// Records

#[dokime::record(
    id = "obc.charger.startup.initialized",
    label = "Charger Initialized",
    description = "The Charger was initialized for this attempt."
)]
#[derive(Clone, Debug)]
pub struct Initialized {}

#[dokime::record(id = "obc.charger.startup.started", label = "AC Supply Energized", description = "The AC supply was energized.")]
#[derive(Clone, Debug)]
pub struct Started {}

#[dokime::record(
    id = "obc.charger.startup.checkpoint",
    label = "Review the charger status indicator and start-up trace",
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
    id = "obc.charger.startup.completed",
    label = "Start-up measurement and operator decision captured",
    description = "The start-up measurement and the operator's decision, together."
)]
#[derive(Clone, Debug)]
pub struct Completed {
    #[record(description = "How many rows the start-up measurement produced.")]
    pub sample_count: u64,

    #[record(description = "What the operator chose.")]
    pub prompt_action: String,

    #[record(description = "What the operator wrote.")]
    pub operator_note: String,

    #[record(description = "Whether the start-up behaved the way this procedure required.")]
    pub accepted: bool,
}

#[dokime::parameters]
pub struct Parameters {
    #[parameter(label = "AC Voltage (RMS)", unit = "V", default = 230.0)]
    pub ac_voltage_v: f64,

    #[parameter(label = "Load Impedance Magnitude", unit = "ohm", default = 100.0)]
    pub load_ohm: f64,

    #[parameter(label = "Supply Frequency", unit = "Hz", default = 50.0)]
    pub frequency_hz: f64,

    #[parameter(label = "Current Phase Lag", unit = "deg", default = 15.0)]
    pub phase_deg: f64,

    #[parameter(label = "RMS Current Limit", unit = "A", default = 5.0)]
    pub current_limit_a: f64,

    #[parameter(label = "Observation Duration", unit = "s", default = 0.02, min = SHORTEST_S)]
    pub duration_s: f64,

    #[parameter(
        label = "Simulation Sample Interval",
        unit = "s",
        default = 0.001,
        min = FINEST_STEP_S,
        max = COARSEST_STEP_S,
    )]
    pub sample_interval_s: f64,

    #[parameter(label = "Attach measurement summary", default = true)]
    pub emit_artifact: bool,

    #[parameter(label = "Attach reference.png from image directory", default = false)]
    pub attach_reference: bool,

    #[parameter(label = "Pace samples in real time", default = false)]
    pub realtime: bool,

    #[parameter(
        label = "Observe until stopped",
        default = false,
        description = "Whether a host stop is the expected end of this run, named once more on the template's own `graceful_stop`."
    )]
    pub continuous: bool,

    #[parameter(label = "Wait for the unit to report itself connected", default = false)]
    pub await_connection: bool,

    #[parameter(
        label = "Connection wait budget",
        unit = "s",
        default = CONNECTION_BUDGET_S,
        min = 0.0,
    )]
    pub connection_budget_s: f64,
}

/* === Enums === */

#[dokime::catalog]
pub enum Prompts {
    StartupCheck(checkpoint::StartupCheck),
}

#[dokime::catalog]
pub enum Required {
    AskReadiness(crate::domain::operations::AskReadiness),
    ListConnections(crate::domain::operations::ListConnections),
}

// `SystemInputs` and the unit's connection: a catalog names one list, so this
// one repeats the six host streams `system::SystemInputs` declares.
#[dokime::catalog]
pub enum Subscriptions {
    ToolConnectionAcquired(ToolConnectionAcquired),
    System(plugin_host::System),
    Network(plugin_host::Network),
    WifiLink(plugin_host::WifiLink),
    WifiLatency(plugin_host::WifiLatency),
    WifiScan(plugin_host::WifiScan),
    Bluetooth(plugin_host::Bluetooth),
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
    ChargerLog(ChargerLog),
}

/* === Traits === */

/* === Implementations === */

#[dokime::template(
    id = STARTUP_TEMPLATE, label = "Start-Up Sequence",
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
    parameters = Parameters, description = "Measure the AC start-up inrush, inspect the Charger, then observe settling.")]
impl Startup {
    async fn initialize(inputs: &Parameters, ctx: &mut InitializationContext<'_>) -> Result<Self, Error> {
        // The route rather than the handle: a template names no plugin at all,
        // so `ctx.call` selecting the explicit route type is the only road
        // to it — and the one that holds the answer to the timeout the route
        // declared, which is what a host call would get.
        if ctx.call::<crate::domain::operations::AskReadiness>(crate::domain::operations::AskReadiness {}).await?.stopped {
            return Err(Error::Device(obc_runtime::Error::Stopped));
        }
        let mut device = Device::initialize(inputs.supply()).map_err(Error::Device)?;
        let log = charger_log();
        std::fs::File::create_new(&log).map_err(|error| {
            let message = format!("charger log: {error}");
            Error::Observation(message)
        })?;
        device.logging(&log);
        Ok(Self {
            produced: 0,
            device,
            log,
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
        let connections = ctx.call::<crate::domain::operations::ListConnections>(crate::domain::operations::ListConnections {}).await?;
        ctx.info(format!("record=bench_connections count={}", connections.resources.len()));
        // The host's own record type, so the step names no id and reads no
        // detail map: the unit's identity is a field of the fact it waited for.
        let connected = ctx
            .listen::<ToolConnectionAcquired>()
            .timeout(Duration::from_secs_f64(ctx.parameters().connection_budget_s))
            .next()
            .await
            .transpose()?
            .expect("a live subscription yields until it is dropped");
        ctx.info(format!("record=connected resource={}", connected.resource_id));
        Ok(())
    }

    #[dokime::step(label = "Initialize and Start Charger")]
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

    #[dokime::step(
        label = "Tail the Charger Log",
        description = "Watches the charger's log file while it reports its first readings.\nThe watch is the host's `WatchFile` operation: it captures only what the charger writes after the watch starts into the `charger/device.log` artifact, and its matcher turns every status line into a row of the charger log stream. Bounded, it settles on its own once the readings are in."
    )]
    async fn tail(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let watch = ctx
            .host()
            .submit(WatchFile {
                execution_id: None,
                source: Some(FileWatchSource {
                    payload: Some(FileWatchSourcePayload::Local(LocalWatchSource {
                        root: self.log.to_string_lossy().into_owned(),
                    })),
                }),
                output: "charger".into(),
                artifact: Some("charger/device.log".into()),
                policy: FileWatchPolicy::Stream,
                selection: FileWatchSelection::All,
                wait_for_source_seconds: None,
                poll_interval_ms: Some(100),
                maximum_seconds: Some(1),
                maximum_bytes: None,
                matchers: vec![LineMatcher::publishing::<ChargerLog>(
                    r"t=(?P<elapsed_s>[\d.]+) state=Running input_power_w=(?P<input_power_w>[\d.]+)",
                )],
                markers: Vec::new(),
            })
            .await?;
        // The watch takes its baseline first; the charger then logs its
        // readings while it runs.
        watch.armed().await?;
        let interval_s = ctx.parameters().sample_interval_s;
        for index in 0..5 {
            self.device.sample(f64::from(index) * interval_s);
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        let captured = watch.join().await?.into_owned();
        ctx.info(format!("record=charger_log bytes={}", captured.artifacts.iter().map(|artifact| artifact.size_bytes).sum::<u64>()));
        Ok(())
    }

    #[dokime::step(label = "Measure Start-Up Inrush")]
    async fn publish(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let inputs = ctx.parameters();
        let (total, interval_s) = (inputs.sample_count()?, inputs.sample_interval_s);
        let checkpoint = (total / 2).max(1);
        let continuous = inputs.continuous;
        let realtime = inputs.realtime || continuous;
        let interval = Duration::from_secs_f64(interval_s);
        if continuous {
            loop {
                publish_runtime_sample(ctx, interval_s, &mut self.device, self.produced, total, checkpoint).await?;
                self.produced += 1;
                tokio::time::sleep(interval).await;
            }
        } else {
            let remaining = sample_range(self.produced, checkpoint)?;
            for index in ctx.progress("Measuring start-up inrush", remaining) {
                let index = index as u64;
                publish_runtime_sample(ctx, interval_s, &mut self.device, index, total, checkpoint).await?;
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
            prompt_type: "obc.charger.startup.check".into(),
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
        ctx.info(format!("record=prompt_answered action={} accepted={}", self.prompt_action, self.accepted));
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
                publish_runtime_sample(ctx, interval_s, &mut self.device, index, total, checkpoint).await?;
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
            ctx.artifacts().write("charger/startup-summary.json", summary)?;
        }
        if inputs.attach_reference {
            let directory: PathBuf = ctx.parameter("images_directory").map_err(|error| Error::Image(error.to_string()))?;
            let path = directory.join("reference.png");
            // The image is an installed asset this step never holds in memory:
            // the size is read from the entry, and the bytes stream past.
            let size = std::fs::metadata(&path).map_err(|error| Error::Image(error.to_string()))?.len();
            if size > 4 * 1024 * 1024 {
                return Err(Error::Image("reference.png exceeds 4 MiB".into()));
            }
            ctx.artifacts().import(&path, "charger/reference.png").map_err(|error| Error::Image(error.to_string()))?;
        }
        Ok(())
    }

    async fn cleanup(&mut self, ctx: &mut CleanupContext<'_>) -> Result<(), Error> {
        self.device.stop();
        let _ = std::fs::remove_file(&self.log);
        ctx.info("Charger resources released");
        Ok(())
    }

    #[dokime::criterion(
        id = "obc.charger.startup.acceptance",
        label = "Start-up trace and operator inspection",
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
        Ok(samples.len() as u64 >= ctx.parameters().sample_count()? && verdicts::decided(ctx.records::<Completed>(), |run| run.accepted)?)
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn publish_runtime_sample<T, O, I>(
    ctx: &mut StepContext<'_, T, O, I>, interval_s: f64, device: &mut Device, index: u64, total: u64, checkpoint: u64,
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
    measurement::publish::<Samples, _, _, _>(ctx, index, reading, phase).await;

    Ok(())
}

// The charger's log lives outside the execution's artifact folder, where only
// the watch's capture of it becomes evidence.
fn charger_log() -> PathBuf {
    let started = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    std::env::temp_dir().join(format!("obc-charger-{}-{started}.log", std::process::id()))
}

fn sample_range(start: u64, end: u64) -> Result<std::ops::Range<usize>, Error> {
    let start = usize::try_from(start).map_err(|_| Error::Observation("sample index exceeds platform capacity".into()))?;
    let end = usize::try_from(end).map_err(|_| Error::Observation("sample count exceeds platform capacity".into()))?;
    Ok(start..end)
}

fn signal(field: &str) -> dokime::expressions::DataExpression {
    telemetry(STARTUP_STREAM, field)
}

fn voltage_distribution(identifier: &str) -> PlotDefinition {
    let mut plot = Plot::histogram("Voltage Distribution");
    plot.id(identifier);
    plot.unit("V");
    plot.x(signal("voltage_v"));

    plot
}

fn current_distribution(identifier: &str) -> PlotDefinition {
    let mut plot = Plot::histogram("Current Distribution");
    plot.id(identifier);
    plot.unit("A");
    plot.x(signal("current_a"));

    plot
}

fn voltage_by_phase(identifier: &str) -> PlotDefinition {
    let mut plot = Plot::box_summary("Voltage By Phase");
    plot.id(identifier);
    plot.unit("V");
    plot.series_by(signal("phase"));
    plot.y(signal("voltage_v"));

    plot
}

fn mean_signal_by_phase(identifier: &str) -> PlotDefinition {
    let mut plot = Plot::bar("Mean Signal By Phase");
    plot.id(identifier);
    plot.unit("V");
    plot.x(signal("phase"));
    plot.y(signal("voltage_v").mean());
    plot.y(signal("voltage_rms_v").mean());

    plot
}

fn sample_count_by_phase(identifier: &str) -> PlotDefinition {
    let mut plot = Plot::bar("Sample Count By Phase");
    plot.id(identifier);
    plot.unit("samples");
    plot.x(signal("phase"));
    plot.y(signal("sample_index").count());

    plot
}

fn signal_correlation(identifier: &str) -> PlotDefinition {
    let mut plot = Plot::xy_scatter("Signal Correlation");
    plot.id(identifier);
    plot.style(PlotStyle::Points);
    plot.x(signal("voltage_v"));
    plot.y(signal("current_a"));
    plot.series_by(signal("phase"));

    plot
}

fn startup_document() -> Document {
    let mut document = Document::new("default", "Start-Up Sequence");

    // === What One Execution Shows === //

    let mut voltage = Plot::time_series("AC Voltage");
    voltage.id("obc.charger.startup.sequence.signals");
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

    let mut timeline = Section::new("Signal Timeline");
    timeline.plot(voltage);
    document.section(timeline);

    let mut distributions = Section::new("Signal Distributions");
    distributions.plot(voltage_distribution("obc.charger.startup.sequence.voltage_distribution"));
    distributions.plot(current_distribution("obc.charger.startup.sequence.current_distribution"));
    distributions.plot(voltage_by_phase("obc.charger.startup.sequence.voltage_by_phase"));
    document.section(distributions);

    let mut phase_breakdown = Section::new("Phase Breakdown");
    phase_breakdown.plot(mean_signal_by_phase("obc.charger.startup.sequence.mean_signal_by_phase"));
    phase_breakdown.plot(sample_count_by_phase("obc.charger.startup.sequence.sample_count_by_phase"));
    phase_breakdown.plot(signal_correlation("obc.charger.startup.sequence.signal_correlation"));
    document.section(phase_breakdown);

    // === The Current, And The Host Around It === //

    let mut current = Plot::time_series("AC Current");
    current.id("obc.charger.startup.sequence.current");
    current.unit("A");
    current.y(signal("current_a"));
    current.y(signal("current_rms_a"));

    let mut ac_current = Section::new("AC Current");
    ac_current.plot(current);
    document.section(ac_current);

    document.section(system::section(STARTUP_TEMPLATE));

    document
}

/* === Tests === */
