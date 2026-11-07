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
use crate::domain::model::bench::{COARSEST_STEP_S, FINEST_STEP_S, SHORTEST_S, bench};
use crate::domain::model::charger::{Charger, Power};
use crate::domain::model::measurement::Row;
use crate::domain::{Error, model::prompts::checkpoint};
use crate::domain::{document as system, model::measurement};
use crate::operations::charger::*;
use crate::operations::wait::Wait;
use dokime::document::{self, PlotStyle};
use dokime::domain::model::files::wire::FileSource;
use dokime::domain::model::files::wire::LineMatcher;
use dokime::domain::model::files::wire::StreamCapture;
use dokime::domain::model::identity::TemplateId;
use dokime::expressions::telemetry;
use dokime::host::ToolConnectionAcquired;
use dokime::operations::files::StartCapture;
use dokime::operations::files::StopCapture;
use dokime::prelude::*;

/* === Constants === */

const CONNECTION_BUDGET_S: f64 = 30.0;
const STARTUP_STREAM: &str = "obc.charger.startup.samples";

/* === Macros === */

bench!(Parameters);

/* === Type Aliases === */

/* === Structs === */

// A type of its own rather than a home on [`Row`]: four suites measure
// the same physical thing and each owns the stream its evidence travels, so
// the shape declares no home and every stream declares itself.
#[dokime::stream(
    id = "obc.charger.startup.samples",
    window = 4096,
    row = Row,
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

pub struct Template {
    produced: u64,
    device: Charger,
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
    label = "Review the Charger Status Indicator and Start-Up Trace",
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
    label = "Start-Up Measurement and Operator Decision Captured",
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
    #[parameter(label = "AC RMS Voltage", unit = "V", default = 230.0)]
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

    #[parameter(label = "Attach Measurement Summary", default = true)]
    pub emit_artifact: bool,

    #[parameter(label = "Attach reference.png from Image Directory", default = false)]
    pub attach_reference: bool,

    #[parameter(label = "Pace Samples in Real Time", default = false)]
    pub realtime: bool,

    #[parameter(
        label = "Observe Until Stopped",
        default = false,
        description = "Whether a host stop is the expected end of this run, named once more on the template's own `graceful_stop`."
    )]
    pub continuous: bool,

    #[parameter(label = "Wait for the Unit to Report Itself Connected", default = false)]
    pub await_connection: bool,

    #[parameter(
        label = "Connection Wait Budget",
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
    AskReadiness(crate::operations::AskReadiness),
    InitializeCharger(InitializeCharger),
    StartCharger(StartCharger),
    SampleCharger(SampleCharger),
    ReleaseCharger(ReleaseCharger),
    EncodeChargerSummary(EncodeChargerSummary),
    Wait(Wait),
    ListConnections(crate::operations::ListConnections),
}

// Host Streams and the Unit Connection
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
impl Template {
    async fn initialize(inputs: &Parameters, ctx: &mut InitializationContext<'_>) -> Result<Self, Error> {
        // Check Bench Readiness
        let operation = crate::operations::AskReadiness {};
        let readiness = ctx.operation(operation).await?;
        if readiness.stopped {
            return Err(Error::Observation("the bench is stopped".into()));
        }

        // Initialize Charger and Its Log
        let operation = InitializeCharger { supply: inputs.supply(), logging: true };
        let device = ctx.operation(operation).await?.into_owned();
        let log = PathBuf::from(device.log.as_ref().expect("logging initialization returns its file"));
        Ok(Self {
            produced: 0,
            device,
            log,
            accepted: false,
            prompt_action: String::new(),
            operator_note: String::new(),
        })
    }

    #[dokime::step(
        label = "Wait for the Unit",
        description = "Waits for the unit to say it is on the bench before energizing it.\nNothing this execution can produce: the connection is reported by whoever owns the fixture — another plugin, the host, or an operator — into this run's scope, and the run has no business guessing. The wait is bounded, and the SDK's timeout becomes this plugin's own error through the `From` on `Error::Awaited`.\nIt also reports what the bench was told *before* this run started, which is what the process-wide connection listener accumulated across every execution and every gap between them."
    )]
    async fn connected(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        if !ctx.parameters().await_connection {
            return Ok(());
        }
        // List Known Connections
        let operation = crate::operations::ListConnections {};
        let connections = ctx.operation(operation).await?;
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
        // Publish Initialized
        let record = Initialized {};
        ctx.publish(record).await;
        // Start Charger
        let operation = StartCharger { device_id: self.device.device_id };
        self.device = ctx.operation(operation).await?.into_owned();
        ctx.info(format!(
            "record=started voltage_rms_v={} load_ohm={} duration_s={}",
            self.device.supply.voltage_rms_v,
            self.device.supply.load_ohm,
            ctx.parameters().duration_s
        ));
        let record = Started {};
        ctx.publish(record).await;
        Ok(())
    }

    #[dokime::step(
        label = "Tail the Charger Log",
        description = "Captures the charger's log file while it reports its first readings.\nThe capture is the host's `StartCapture` operation: it captures only what the charger writes after it starts into the `charger/device.log` artifact, and its matcher turns every status line into a row of the charger log stream. The step stops and seals the capture after writing its readings."
    )]
    async fn tail(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        // Start Capture
        let matcher =
            LineMatcher::publishing::<ChargerLog>(r"t=(?P<elapsed_s>[\d.]+) state=Running input_power_w=(?P<input_power_w>[\d.]+)");
        let request = StartCapture::new(FileSource::file(&self.log), "charger/device.log", StreamCapture::plain().matcher(matcher));
        let capture = ctx.operation(request).await?;

        let interval_s = ctx.parameters().sample_interval_s;
        for index in 0..5 {
            // Sample Charger Into Its Log
            let operation = SampleCharger {
                device_id: self.device.device_id,
                elapsed_s: f64::from(index) * interval_s,
            };
            let sample = ctx.operation(operation).await?.into_owned();
            self.device = sample.charger;
        }

        // Stop Capture
        let request = StopCapture { capture_id: capture.capture_id };
        let captured = ctx.operation(request).await?.into_owned();
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
        if continuous {
            loop {
                // Sample Charger
                let index = self.produced;
                let operation = SampleCharger {
                    device_id: self.device.device_id,
                    elapsed_s: index as f64 * interval_s,
                };
                let sample = ctx.operation(operation).await?.into_owned();
                self.device = sample.charger;
                let phase = if self.device.power == Power::Faulted {
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

                // Publish Measurement
                let row = measurement::row(index, sample.reading, phase);
                ctx.stream::<Row>().into::<Samples>().push(row).await;
                self.produced += 1;
                // Pace the Measurement
                let operation = Wait { duration_s: interval_s };
                ctx.operation(operation).await?;
            }
        } else {
            let remaining = sample_range(self.produced, checkpoint)?;
            for index in ctx.progress("Measuring start-up inrush", remaining) {
                let index = index as u64;
                // Sample Charger
                let operation = SampleCharger {
                    device_id: self.device.device_id,
                    elapsed_s: index as f64 * interval_s,
                };
                let sample = ctx.operation(operation).await?.into_owned();
                self.device = sample.charger;
                let phase = if self.device.power == Power::Faulted {
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

                // Publish Measurement
                let row = measurement::row(index, sample.reading, phase);
                ctx.stream::<Row>().into::<Samples>().push(row).await;
                self.produced = index + 1;
                if realtime {
                    // Pace the Measurement
                    let operation = Wait { duration_s: interval_s };
                    ctx.operation(operation).await?;
                }
            }
        }
        ctx.info(format!("record=checkpoint samples={}", self.produced));
        Ok(())
    }

    #[dokime::step(label = "Inspect Powered Charger")]
    async fn prompt(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let record = Checkpoint {
            sample_index: self.produced,
            prompt_type: "obc.charger.startup.check".into(),
        };
        ctx.publish(record).await;
        ctx.info("record=prompt_requested");
        // Ask for Operator Inspection
        let prompt = checkpoint::StartupCheck {
            operator_note: "charger status indicator steady".into(),
            ..Default::default()
        };
        let response = ctx.ask(prompt).await?;
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
                // Sample Charger
                let operation = SampleCharger {
                    device_id: self.device.device_id,
                    elapsed_s: index as f64 * interval_s,
                };
                let sample = ctx.operation(operation).await?.into_owned();
                self.device = sample.charger;
                let phase = if self.device.power == Power::Faulted {
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

                // Publish Measurement
                let row = measurement::row(index, sample.reading, phase);
                ctx.stream::<Row>().into::<Samples>().push(row).await;
                self.produced = index + 1;
                if inputs.realtime {
                    // Pace the Measurement
                    let operation = Wait { duration_s: interval_s };
                    ctx.operation(operation).await?;
                }
            }
        }
        // Publish Completed
        let record = Completed {
            sample_count: self.produced,
            prompt_action: self.prompt_action.clone(),
            operator_note: self.operator_note.clone(),
            accepted: self.accepted && self.device.fault.is_none(),
        };
        ctx.publish(record).await;
        if inputs.emit_artifact {
            // Encode Measurement Summary
            let operation = EncodeChargerSummary {
                supply: self.device.supply,
                sample_count: self.produced,
                duration_s: Some(self.produced as f64 * interval_s),
                prompt_action: self.prompt_action.clone(),
                operator_note: Some(self.operator_note.clone()),
                accepted: self.accepted,
            };
            let summary = ctx.operation(operation).await?.into_owned();
            ctx.artifacts().write("charger/startup-summary.json", summary.json)?;
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

    #[dokime::criterion(
        id = "obc.charger.startup.acceptance",
        label = "Start-Up Trace and Operator Inspection",
        description = "Passed once the trace is unfaulted and, for a bounded run, the operator accepted it."
    )]
    fn accepted(ctx: &CriterionContext<'_>) -> Result<bool, Error> {
        let continuous = ctx.parameters().continuous;
        let samples = ctx.samples::<Samples>();
        if continuous {
            let Some(last) = samples.last() else {
                return Ok(false);
            };
            // Check / Final Fault State
            let fault_free = last.phase != "fault";
            return Ok(fault_free);
        }
        let records = ctx.records::<Completed>()?;
        let completed = records.last().ok_or_else(|| Error::MissingEvidence(Completed::RECORD_TYPE.into()))?;

        // Check / Number of Rows
        let number_ok = samples.len() as u64 >= ctx.parameters().sample_count()?;

        // Check / Operator Acceptance
        let accepted_ok = completed.accepted;
        Ok(number_ok && accepted_ok)
    }

    async fn cleanup(&mut self, ctx: &mut CleanupContext<'_>) -> Result<(), Error> {
        // Release Charger and Its Log
        let operation = ReleaseCharger { device_id: self.device.device_id };
        let result = ctx.operation(operation).await;
        if let Err(error) = &result {
            ctx.warn(format!("charger cleanup: {error}"));
        }
        result?;
        ctx.info("Charger resources released");
        Ok(())
    }

    fn document() -> Document {
        let mut document = Document::new("default", "Start-Up Sequence");

        // === What One Execution Shows === //

        let mut voltage = Plot::time_series("AC Voltage");
        voltage.id("obc.charger.startup.sequence.signals");
        voltage.unit("V");
        voltage.style(PlotStyle::LinePoints);
        voltage.y(telemetry(STARTUP_STREAM, "voltage_v"));
        voltage.y(telemetry(STARTUP_STREAM, "voltage_rms_v"));
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
        let mut plot = Plot::histogram("Voltage Distribution");
        plot.id("obc.charger.startup.sequence.voltage_distribution");
        plot.unit("V");
        plot.x(telemetry(STARTUP_STREAM, "voltage_v"));
        distributions.plot(plot);
        let mut plot = Plot::histogram("Current Distribution");
        plot.id("obc.charger.startup.sequence.current_distribution");
        plot.unit("A");
        plot.x(telemetry(STARTUP_STREAM, "current_a"));
        distributions.plot(plot);
        let mut plot = Plot::box_summary("Voltage by Phase");
        plot.id("obc.charger.startup.sequence.voltage_by_phase");
        plot.unit("V");
        plot.series_by(telemetry(STARTUP_STREAM, "phase"));
        plot.y(telemetry(STARTUP_STREAM, "voltage_v"));
        distributions.plot(plot);
        document.section(distributions);

        let mut phase_breakdown = Section::new("Phase Breakdown");
        let mut plot = Plot::bar("Mean Signal by Phase");
        plot.id("obc.charger.startup.sequence.mean_signal_by_phase");
        plot.unit("V");
        plot.x(telemetry(STARTUP_STREAM, "phase"));
        plot.y(telemetry(STARTUP_STREAM, "voltage_v").mean());
        plot.y(telemetry(STARTUP_STREAM, "voltage_rms_v").mean());
        phase_breakdown.plot(plot);
        let mut plot = Plot::bar("Sample Count by Phase");
        plot.id("obc.charger.startup.sequence.sample_count_by_phase");
        plot.unit("samples");
        plot.x(telemetry(STARTUP_STREAM, "phase"));
        plot.y(telemetry(STARTUP_STREAM, "sample_index").count());
        phase_breakdown.plot(plot);
        let mut plot = Plot::xy("Signal Correlation");
        plot.id("obc.charger.startup.sequence.signal_correlation");
        plot.style(PlotStyle::Points);
        plot.x(telemetry(STARTUP_STREAM, "voltage_v"));
        plot.y(telemetry(STARTUP_STREAM, "current_a"));
        plot.series_by(telemetry(STARTUP_STREAM, "phase"));
        phase_breakdown.plot(plot);
        document.section(phase_breakdown);

        // === Current and Host Measurements === //

        let mut current = Plot::time_series("AC Current");
        current.id("obc.charger.startup.sequence.current");
        current.unit("A");
        current.y(telemetry(STARTUP_STREAM, "current_a"));
        current.y(telemetry(STARTUP_STREAM, "current_rms_a"));

        let mut ac_current = Section::new("AC Current");
        ac_current.plot(current);
        document.section(ac_current);

        let section = system::section(STARTUP_TEMPLATE);

        document.section(section);

        document
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn sample_range(start: u64, end: u64) -> Result<std::ops::Range<usize>, Error> {
    let start = usize::try_from(start).map_err(|_| Error::Observation("sample index exceeds platform capacity".into()))?;
    let end = usize::try_from(end).map_err(|_| Error::Observation("sample count exceeds platform capacity".into()))?;
    Ok(start..end)
}

/* === Tests === */
