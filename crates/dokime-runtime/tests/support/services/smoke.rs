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
use std::io::Write;
use std::sync::Arc;

// Third Party Imports

// Local Imports
use dokime_runtime::Refusal;
use dokime_runtime::execution::Runnable;
use dokime_runtime::execution::{Caller, ExecutionHost, SampleOutput, SidecarTemplate, Workflow};
use dokime_runtime::transport::{
    EvidenceBatch, EvidenceReceipt, ExecutionSignal, LeaseAsk, LeaseResolution, OperatorAnswer, OperatorQuestion, RunExecution,
    SidecarExecutionOutcome, SignalReply, TransportFuture,
};

/* === Constants === */
pub const SOURCE: &str = "fixture_suites";
pub const SUITE: &str = "fixture.checks";

pub const PULSE_STREAM: &str = "fixture_suites.pulse";

pub const PULSE_PLOT: &str = "fixture.checks.smoke.pulse";

pub const PULSE_DISPLAY_POINTS: u32 = 400;

const SPECTRUM_SAMPLE_RATE_HZ: f64 = 100.0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Observed<T: SidecarTemplate> {
    pub workflow: Workflow<T>,
    pub watched: tokio::sync::watch::Sender<bool>,
}

struct Admitted {
    host: Arc<dyn ExecutionHost>,
    watched: tokio::sync::watch::Sender<bool>,
}

pub struct Behaviour {
    pulse_rows: u32,
    spectrum_rows: u32,
    hold: bool,
    released: tokio::sync::watch::Sender<bool>,
    artifacts: bool,
    abandoned: bool,
    watched: tokio::sync::watch::Sender<bool>,
}

pub struct Smoke {
    behaviour: Arc<Behaviour>,
}

struct Checks;

struct Fixture {
    runner: Arc<Observed<Check>>,
    behaviour: Arc<Behaviour>,
}
struct FixtureSuites;

#[dokime_runtime::catalog]
enum Suites {
    Checks(Checks),
}

#[harmos::catalog]
enum Operations {
    Run(dokime_runtime::testing::Run<Observed<Check>>),
    Signal(dokime_runtime::testing::Signal<Observed<Check>>),
}

struct Check {
    behaviour: Arc<Behaviour>,
}

// The Window Holds Every Row The Largest Story Pushes Before Ingest Takes Them In.
#[dokime_runtime::stream(
    id = "fixture_suites.pulse",
    window = 32_768,
    label = "Fixture Pulse",
    description = "A stream the smoke step fills only when asked to."
)]
struct Pulse {
    value: f64,
}

#[dokime_runtime::stream(
    id = "fixture_suites.spectrum",
    window = 32_768,
    label = "Fixture Spectrum",
    description = "A stream drawn by a frequency plot that transforms the whole capture at once."
)]
struct Spectrum {
    value: f64,
}

#[dokime_runtime::record(
    id = "fixture.checks.pulse_produced",
    label = "Pulse Produced",
    description = "How many pulse rows the smoke step pushed."
)]
#[derive(Clone, Debug)]
struct PulseProduced {
    #[record(description = "The number of rows pushed onto the pulse stream.")]
    rows: u32,
}

/* === Enums === */
#[dokime_runtime::catalog]
enum Streams {
    Pulse(Pulse),
    Spectrum(Spectrum),
}

#[dokime_runtime::catalog]
enum Records {
    PulseProduced(PulseProduced),
}

#[dokime_runtime::catalog]
enum Templates {
    Check(Check),
}

/* === Traits === */

/* === Implementations === */
impl Behaviour {
    pub fn new(pulse_rows: u32, spectrum_rows: u32, hold: bool) -> Self {
        Self {
            pulse_rows,
            spectrum_rows,
            hold,
            released: tokio::sync::watch::Sender::new(false),
            artifacts: false,
            abandoned: false,
            watched: tokio::sync::watch::Sender::new(false),
        }
    }

    pub fn artifacts(mut self, abandoned: bool) -> Self {
        self.artifacts = !abandoned;
        self.abandoned = abandoned;
        self
    }

    pub fn release(&self) {
        self.released.send_replace(true);
    }
}

impl Smoke {
    pub fn new(behaviour: Behaviour) -> Self {
        Self { behaviour: Arc::new(behaviour) }
    }

    pub fn behaviour(&self) -> Arc<Behaviour> {
        Arc::clone(&self.behaviour)
    }

    pub fn configure(self, config: dokime_runtime::RuntimeConfig, directory: &std::path::Path) -> dokime_runtime::RuntimeConfig {
        dokime_runtime::testing::bound_plugin::<Fixture, Behaviour>(config, directory, self.behaviour).unwrap()
    }
}

#[dokime_runtime::suite(id = "fixture.checks", label = "Fixture Checks", templates = Templates)]
impl Checks {}

#[dokime_runtime::template(
    id = "fixture.checks.smoke",
    label = "Fixture Smoke",
    streams = Streams,
    records = Records
)]
impl Check {
    fn document() -> dokime_runtime::prelude::Document {
        let mut plot = dokime_runtime::prelude::Plot::time_series("Pulse");
        plot.id(PULSE_PLOT);
        plot.y(dokime_runtime::expressions::telemetry("fixture_suites.pulse", "value"));
        plot.display_points = Some(PULSE_DISPLAY_POINTS);
        plot.downsample = dokime_runtime::document::DownsampleTechnique::MinMax;
        let mut section = dokime_runtime::prelude::Section::new("Pulse");
        section.plot(plot);
        let mut spectrum = dokime_runtime::prelude::Plot::fft("Spectrum");
        spectrum.id("fixture.checks.smoke.spectrum");
        spectrum.y(dokime_runtime::expressions::telemetry("fixture_suites.spectrum", "value"));
        spectrum.fft.sample_rate_hz = Some(SPECTRUM_SAMPLE_RATE_HZ);
        section.plot(spectrum);
        let mut document = dokime_runtime::prelude::Document::new("default", "Fixture Smoke");
        document.section(section);
        document
    }

    async fn initialize(ctx: &mut dokime_runtime::execution::InitializationContext<'_>) -> dokime_runtime::Result<Self> {
        let behaviour = Arc::clone(ctx.environment().binding::<Arc<Behaviour>>());
        Ok(Self { behaviour })
    }

    #[dokime_runtime::step(label = "Check")]
    async fn check(&mut self, ctx: &mut dokime_runtime::execution::StepContext<'_>) -> dokime_runtime::Result<()> {
        let behaviour = Arc::clone(&self.behaviour);

        if behaviour.pulse_rows > 0 {
            let pulse = ctx.stream::<Pulse>().open();
            for index in 0..behaviour.pulse_rows {
                pulse.push(Pulse { value: f64::from(index) }).await;
            }
        }
        if behaviour.spectrum_rows > 0 {
            let spectrum = ctx.stream::<Spectrum>().open();
            for index in 0..behaviour.spectrum_rows {
                spectrum.push(Spectrum { value: f64::from(index % 16) }).await;
            }
        }
        ctx.publish(PulseProduced { rows: behaviour.pulse_rows }).await;

        // Stream Artifacts Through The Same Host Bridge As A Sidecar.
        if behaviour.artifacts {
            let mut evidence = ctx.artifacts().create("evidence/pulse.csv")?;
            evidence.write_all(b"sample,value\n")?;
            for index in 0..262_144 {
                writeln!(evidence, "{index:08},{:03}", index % 251)?;
            }
            evidence.finish()?;
            let directory = ctx.artifacts().directory("capture")?;
            std::fs::write(directory.join("chamber.log"), vec![b'.'; 262_144])?;
            // The Wrapper Acknowledges The First Version After The Real Host Admits It.
            let mut watched = behaviour.watched.subscribe();
            watched.wait_for(|seen| *seen).await.map_err(std::io::Error::other)?;
            std::fs::write(directory.join("chamber.log"), b"the chamber log, rewritten in place\n")?;
        }
        if behaviour.abandoned {
            for index in 0..17 {
                let mut partial = ctx.artifacts().create(format!("evidence/partial-{index}.bin"))?;
                partial.write_all(&vec![0xA5; dokime_runtime::transport::MAX_ARTIFACT_CHUNK_BYTES + 1])?;
                partial.abandon();
            }
            ctx.artifacts().write("evidence/settled.txt", b"produced after the abandoned transfers\n")?;
        }

        if behaviour.hold {
            let mut released = behaviour.released.subscribe();
            let _ = released.wait_for(|released| *released).await;
        }
        Ok(())
    }

    #[dokime_runtime::criterion(
        id = "fixture.checks.smoke.pulse_replayed",
        label = "Pulse Replayed",
        description = "Passed when the host handed back every pulse row the step produced, each exactly once."
    )]
    fn pulse_replayed(ctx: &dokime_runtime::execution::CriterionContext<'_>) -> dokime_runtime::Result<bool> {
        let produced = ctx.records::<PulseProduced>()?.last().map_or(0, |produced| produced.rows);
        let mut values = ctx.samples::<Pulse>().iter().map(|row| row.value).collect::<Vec<_>>();
        values.sort_by(f64::total_cmp);
        Ok(values.len() == produced as usize && values.iter().enumerate().all(|(index, value)| *value == index as f64))
    }
}

impl harmos::plugin::Contribution for FixtureSuites {
    const NAMESPACE: &'static str = <dokime_runtime::Suites<Suites> as harmos::plugin::Contribution>::NAMESPACE;
    const SECTION: harmos::plugin::inventory::Section = <dokime_runtime::Suites<Suites> as harmos::plugin::Contribution>::SECTION;
    fn register(registrar: &mut harmos::plugin::Registrar) -> Result<(), harmos::plugin::RegistrationError> {
        <dokime_runtime::Suites<Suites> as harmos::plugin::Contribution>::register(registrar)
    }
}

impl harmos::builtins::lifecycle::Policy for Fixture {
    const INITIALIZATION_POLICY: harmos::operations::OperationPolicy = harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(120),
        attempt_timeout: std::time::Duration::from_secs(120),
        cancellation_timeout: std::time::Duration::ZERO,
        retries: harmos::operations::RetryPolicy::Never,
    };
    const SHUTDOWN_POLICY: harmos::operations::OperationPolicy = Self::INITIALIZATION_POLICY;
}

#[dokime_runtime::plugin(id = "fixture_suites", restart = "never",
    requires(EvidenceBatch, OperatorQuestion, LeaseAsk, harmos::builtins::streams::Publish,
        dokime_runtime::operations::files::WatchFile, dokime_runtime::operations::files::AwaitFileWatchArmed),
    contributions(FixtureSuites, dokime_runtime::Evidence<Streams>, dokime_runtime::Records<Records>))]
impl dokime_runtime::Plugin for Fixture {
    const CATALOGS: &'static [harmos::Catalog] = &[Operations::CATALOG];
}

impl harmos::Lifecycle for Fixture {
    type Config = dokime_runtime::plugin::Configuration;
    type Error = Refusal;
    type Context = dokime_runtime::Supervisor;
    async fn prepare(config: Self::Config, setup: &mut harmos::Setup<Self>) -> Result<Self, Refusal> {
        let behaviour = dokime_runtime::testing::resource::<Behaviour>(&config)?;
        let binding: Arc<dyn std::any::Any + Send + Sync> = Arc::new(behaviour.clone());
        let records = dokime_runtime::catalog::entries::<Records, dokime_runtime::catalog::Records>()
            .into_iter()
            .map(|contract| contract())
            .collect::<Vec<_>>();
        let runner = Arc::new(Observed {
            workflow: Workflow::<Check>::new(SUITE, SOURCE)
                .bound(binding)
                .with_records(&records)
                .with_publisher(<Check as SidecarTemplate>::publish_sample::<Self>),
            watched: behaviour.watched.clone(),
        });
        setup.resource::<dokime_runtime::testing::Runner<Observed<Check>>>(Arc::new(dokime_runtime::testing::Runner(runner.clone())));
        Ok(Self { runner, behaviour })
    }
    async fn stop(&mut self, _: &Self::Context) -> Result<(), Refusal> {
        self.behaviour.release();
        self.runner.interrupt();
        self.runner.shutdown().await;
        Ok(())
    }
}

impl<T: SidecarTemplate> dokime_runtime::execution::Served for Observed<T> {
    const RUN: &'static str = T::RUN;
    const SIGNAL: &'static str = T::SIGNAL;
}

impl ExecutionHost for Admitted {
    fn push(&self, batch: EvidenceBatch) -> TransportFuture<'_, std::result::Result<EvidenceReceipt, String>> {
        let watched = batch.artifact_chunks.iter().any(|chunk| chunk.name == "capture/chamber.log" && !chunk.last);
        Box::pin(async move {
            let receipt = self.host.push(batch).await?;
            if watched {
                self.watched.send_replace(true);
            }
            Ok(receipt)
        })
    }

    fn ask(&self, question: OperatorQuestion) -> TransportFuture<'_, std::result::Result<OperatorAnswer, String>> {
        self.host.ask(question)
    }

    fn lease(&self, ask: LeaseAsk) -> TransportFuture<'_, std::result::Result<LeaseResolution, String>> {
        self.host.lease(ask)
    }
}

impl<T: SidecarTemplate> Runnable for Observed<T> {
    fn template_identifier(&self) -> &str {
        self.workflow.template_identifier()
    }

    fn run(
        &self, request: RunExecution, host: Arc<dyn ExecutionHost>, output: Option<SampleOutput>, caller: Caller,
    ) -> TransportFuture<'_, std::result::Result<SidecarExecutionOutcome, Refusal>> {
        let admitted = Arc::new(Admitted { host, watched: self.watched.clone() });
        self.workflow.run(request, admitted, output, caller)
    }

    fn signal(&self, signal: ExecutionSignal) -> TransportFuture<'_, std::result::Result<SignalReply, Refusal>> {
        Box::pin(async move { self.workflow.signal(signal) })
    }

    fn take_local_samples(&self, token: &str) -> std::result::Result<Vec<dokime_runtime::telemetry::TelemetrySample>, Refusal> {
        self.workflow.take_local_samples(token)
    }

    fn interrupt(&self) {
        self.workflow.interrupt();
    }

    fn shutdown(&self) -> TransportFuture<'_, ()> {
        self.workflow.shutdown()
    }
}

/* === Public Functions === */

/* === Private Functions === */
/* === Tests === */
