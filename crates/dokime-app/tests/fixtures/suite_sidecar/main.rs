/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod error;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
// Third Party Imports
use dokime::harmos::operations::{
    Host, OperationPolicy, RestartPolicy, RetryPolicy, ServingError, WireTable,
};
// Local Imports
use dokime::transport::MAX_ARTIFACT_CHUNK_BYTES;
use dokime::{Refusal, Supervisor};
use error::Error;

/* === Constants === */
const ARTIFACT_EVIDENCE_PATH: &str = "evidence/pulse.csv";

const ABANDONED_ARTIFACT_PREFIX: &str = "evidence/partial";

const ABANDONED_ARTIFACT_COUNT: usize = 17;

const SETTLED_ARTIFACT_PATH: &str = "evidence/settled.txt";
const SETTLED_ARTIFACT_CONTENT: &str = "produced after the abandoned transfers\n";

const WATCHED_DIRECTORY: &str = "capture";
const WATCHED_FILE: &str = "chamber.log";
const WATCHED_CONTENT: &str = "the chamber log, rewritten in place\n";

const WATCHED_FIRST_BYTES: usize = 256 * 1024;
const ARTIFACT_EVIDENCE_HEADER: &str = "sample,value\n";
const ARTIFACT_EVIDENCE_ROWS: usize = 262_144;

/// Points the pulse plot declares it shows, whatever the capture's length.
const PULSE_DISPLAY_POINTS: u32 = 400;

/// Sample rate the spectrum plot transforms its rows at.
const SPECTRUM_SAMPLE_RATE_HZ: f64 = 100.0;

const POLICY: OperationPolicy = OperationPolicy {
    timeout: Duration::from_secs(5),
    attempt_timeout: Duration::from_secs(5),
    retries: RetryPolicy::Never,
    restart: RestartPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime::section(
    label = "Fixture",
    description = "This fixture's whole settings card, doubling as its own internal state: every field here is test wiring rather than an operator knob, so the section and the plugin's resolved configuration are the same struct."
)]
#[derive(Clone, Debug, Default, PartialEq)]
struct Config {
    marker: Option<std::path::PathBuf>,
    refuse: bool,
    block_execution: bool,
    execution_marker: Option<std::path::PathBuf>,
    execution_release: Option<std::path::PathBuf>,
    artifact_evidence: bool,
    #[setting(
        description = "Abandons more multi-chunk artifacts than the host admits open at once and then produces one more, for the host's \"an abandoned transfer is released, not held until the run ends\" rule."
    )]
    #[setting(
        description = "Abandons more multi-chunk artifacts than the host admits open at once and then produces one more, for the host's \"an abandoned transfer is released, not held until the run ends\" rule."
    )]
    abandon_artifact: bool,
    #[setting(
        description = "Pushes this many rows onto the template's own pulse stream, so the host replays and finalizes a case whose evidence spans many storage pages."
    )]
    pulse_rows: u32,
    #[setting(
        description = "Logs this many lines from the check step, so every execution owns a sidecar stream log the host seals as a native output when the cycle finishes."
    )]
    log_lines: u32,
    #[setting(
        description = "Pushes this many rows onto the template's spectrum stream, whose plot needs the whole capture, so the host keeps those rows while every other stream pages through."
    )]
    spectrum_rows: u32,
}

#[dokime::settings]
#[derive(Clone, Debug, Default, PartialEq)]
struct SuiteSidecarSettings {
    config: Config,
}

#[dokime::stream(
    id = "fixture-suites.pulse",
    window = 1024,
    label = "Fixture Pulse",
    description = "A stream the smoke step fills only when asked to, so its inspected inventory carries one stream revision a reload can be refused over."
)]
pub struct Pulse {
    value: f64,
}

#[dokime::stream(
    id = "fixture-suites.spectrum",
    window = 1024,
    label = "Fixture Spectrum",
    description = "A stream the smoke step fills only when asked to, drawn by a frequency plot that transforms the whole capture at once."
)]
pub struct Spectrum {
    value: f64,
}

#[dokime::record(
    id = "fixture.checks.pulse_produced",
    label = "Pulse Produced",
    description = "How many pulse rows the smoke step pushed, so its criterion can count what the host replayed against what was produced."
)]
#[derive(Clone, Debug)]
struct PulseProduced {
    #[record(description = "The number of rows pushed onto the pulse stream.")]
    rows: u32,
}

struct SuiteSidecar {
    config: Arc<Config>,
}

struct Checks;

struct Smoke {
    behaviour: Behaviour,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
struct Behaviour {
    execution_marker: String,
    artifact_evidence: bool,
    abandon_artifact: bool,
    block_execution: bool,
    execution_release: String,
    pulse_rows: u32,
    log_lines: u32,
    spectrum_rows: u32,
}

struct GetBehaviour {
    config: Arc<Config>,
}

/* === Enums === */
#[dokime::catalog]
pub enum Streams {
    Pulse(Pulse),
    Spectrum(Spectrum),
}

#[dokime::catalog]
enum Records {
    PulseProduced(PulseProduced),
}

#[dokime::catalog]
enum Operations {
    GetBehaviour(GetBehaviour),
}

#[dokime::catalog]
enum Suites {
    Checks(Checks),
}

#[dokime::catalog]
enum ChecksTemplates {
    Smoke(Smoke),
}

#[dokime::catalog]
enum SmokeRequires {
    GetBehaviour(GetBehaviour),
}

/* === Traits === */

/* === Implementations === */
impl dokime::harmos::operations::lifecycle::Policy for SuiteSidecar {
    const INITIALIZATION_POLICY: OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: OperationPolicy = POLICY;
}

#[dokime::plugin(
    id = "fixture-suites",
    provides = Operations,
    requires(),
    contributions(
        dokime::Suites<Suites>,
        dokime::Settings<SuiteSidecarSettings>,
    ),
    restart = "never", description = "Registers one suite for runtime loading tests.")]
impl dokime::Plugin for SuiteSidecar {
    type Config = dokime::plugin::Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = Config;

    fn prepare(config: dokime::plugin::Configuration) -> Result<Self, Refusal> {
        let settings = dokime::plugin::refusal(config.settings::<SuiteSidecarSettings>())?;
        Ok(Self {
            config: Arc::new(settings.config),
        })
    }

    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        tracing::info!("suite fixture initialized");
        if let Some(marker) = &self.config.marker {
            dokime::plugin::refusal(std::fs::write(marker, b"initialized").map_err(Error::from))?;
        }
        if self.config.refuse {
            return dokime::plugin::refusal(Err(Error::InitializationRefused));
        }
        Ok(())
    }

    fn resources(&self) -> Arc<Config> {
        Arc::clone(&self.config)
    }

    fn bind_operations(&self, table: &mut WireTable<Supervisor>) -> Result<(), ServingError> {
        table.bind_operations::<Operations, _>(self.resources())?;
        dokime::execution::serve::<Self, Suites>(table)
    }

    async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        dokime::execution::interrupt().await;
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        Ok(())
    }
}

#[dokime::operation(
    id = "fixture-suites.get_behaviour",
    version = 1,
    description = "Tells one execution what the settings card asked it to do.",
    policy = POLICY
)]
impl GetBehaviour {
    async fn initialize(config: Arc<Config>, _: &Supervisor) -> Self {
        Self { config }
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<Behaviour, Refusal> {
        let path = |path: &Option<std::path::PathBuf>| {
            path.as_ref()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default()
        };
        Ok(Behaviour {
            execution_marker: path(&self.config.execution_marker),
            artifact_evidence: self.config.artifact_evidence,
            abandon_artifact: self.config.abandon_artifact,
            block_execution: self.config.block_execution,
            execution_release: path(&self.config.execution_release),
            pulse_rows: self.config.pulse_rows,
            log_lines: self.config.log_lines,
            spectrum_rows: self.config.spectrum_rows,
        })
    }
}

#[dokime::suite(id = "fixture.checks", label = "Fixture Checks", templates = ChecksTemplates)]
impl Checks {}

#[dokime::template(
    id = "fixture.checks.smoke",
    label = "Fixture Smoke",
    requires = SmokeRequires,
    streams = Streams,
    records = Records
)]
impl Smoke {
    fn document() -> dokime::prelude::Document {
        let mut plot = dokime::prelude::Plot::time_series("Pulse");
        plot.id("fixture.checks.smoke.pulse");
        plot.y(dokime::expressions::telemetry(
            "fixture-suites.pulse",
            "value",
        ));
        plot.display_points = Some(PULSE_DISPLAY_POINTS);
        plot.downsample = dokime::document::DownsampleTechnique::MinMax;
        let mut section = dokime::prelude::Section::new("fixture.checks.smoke.pulse", "Pulse");
        section.plot(plot);
        let mut spectrum = dokime::prelude::Plot::fft("Spectrum");
        spectrum.id("fixture.checks.smoke.spectrum");
        spectrum.y(dokime::expressions::telemetry(
            "fixture-suites.spectrum",
            "value",
        ));
        spectrum.fft.sample_rate_hz = Some(SPECTRUM_SAMPLE_RATE_HZ);
        section.plot(spectrum);
        let mut document = dokime::prelude::Document::new("default", "Fixture Smoke");
        document.section(section);
        document
    }

    async fn initialize(
        ctx: &mut dokime::execution::InitializationContext<'_>,
    ) -> Result<Self, Error> {
        Ok(Self {
            behaviour: ctx.call::<GetBehaviour>(()).await?,
        })
    }

    #[dokime::step(label = "Check")]
    async fn check(&mut self, ctx: &mut dokime::execution::StepContext<'_>) -> Result<(), Error> {
        // The step's host surface reaches the same operation, once as a one
        // shot and once as a submitted instance it joins.
        let host = ctx.host();
        let called = host.call_as::<GetBehaviourDeclaration>(()).await?;
        let submitted = host.submit_as::<GetBehaviourDeclaration>(()).await?;
        let joined = submitted.join().await?;
        if called.into_owned() != self.behaviour || joined.into_owned() != self.behaviour {
            return Err(Error::HostDisagreed);
        }
        if !self.behaviour.execution_marker.is_empty() {
            std::fs::write(
                &self.behaviour.execution_marker,
                std::env::current_exe()?.to_string_lossy().as_bytes(),
            )?;
        }
        for line in 0..self.behaviour.log_lines {
            tracing::info!(line, "fixture check logged");
        }
        if self.behaviour.pulse_rows > 0 {
            let pulse = ctx.stream::<Pulse>().open();
            for index in 0..self.behaviour.pulse_rows {
                pulse
                    .push(Pulse {
                        value: f64::from(index),
                    })
                    .await;
            }
            ctx.publish(PulseProduced {
                rows: self.behaviour.pulse_rows,
            })
            .await;
        }
        if self.behaviour.spectrum_rows > 0 {
            let spectrum = ctx.stream::<Spectrum>().open();
            for index in 0..self.behaviour.spectrum_rows {
                spectrum
                    .push(Spectrum {
                        value: f64::from(index % 16),
                    })
                    .await;
            }
        }
        if self.behaviour.artifact_evidence {
            let mut evidence = ctx.artifacts().create(ARTIFACT_EVIDENCE_PATH)?;
            evidence.write_all(ARTIFACT_EVIDENCE_HEADER.as_bytes())?;
            for index in 0..ARTIFACT_EVIDENCE_ROWS {
                writeln!(evidence, "{index:08},{:03}", index % 251)?;
            }
            evidence.finish()?;
            // The other half of the producing end: a step that is handed a
            // directory and writes into it with nothing but `std::fs`, the way
            // a tool that never heard of Dokime would.
            let directory = ctx.artifacts().directory(WATCHED_DIRECTORY)?;
            std::fs::write(
                directory.join(WATCHED_FILE),
                vec![b'.'; WATCHED_FIRST_BYTES],
            )?;
            // Long enough for the watch to have streamed that version, so what
            // follows is a replacement rather than a first sighting.
            tokio::time::sleep(std::time::Duration::from_millis(400)).await;
            std::fs::write(directory.join(WATCHED_FILE), WATCHED_CONTENT)?;
        }
        if self.behaviour.abandon_artifact {
            for index in 0..ABANDONED_ARTIFACT_COUNT {
                let mut partial = ctx
                    .artifacts()
                    .create(format!("{ABANDONED_ARTIFACT_PREFIX}-{index}.bin"))?;
                // More than one chunk, so pieces are already on their way to the
                // host by the time the step gives up on the artifact.
                partial.write_all(&vec![0xA5; MAX_ARTIFACT_CHUNK_BYTES + 1])?;
                partial.abandon();
            }
            ctx.artifacts()
                .write(SETTLED_ARTIFACT_PATH, SETTLED_ARTIFACT_CONTENT)?;
        }
        if self.behaviour.block_execution {
            if self.behaviour.execution_release.is_empty() {
                std::future::pending::<()>().await;
            } else {
                while !std::path::Path::new(&self.behaviour.execution_release).exists() {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            }
        }
        Ok(())
    }

    #[dokime::criterion(
        id = "fixture.checks.smoke.pulse_replayed",
        label = "Pulse Replayed",
        description = "Passed when the host handed back every pulse row the step produced, each exactly once."
    )]
    fn pulse_replayed(ctx: &dokime::execution::CriterionContext<'_>) -> Result<bool, Error> {
        let produced = ctx
            .records::<PulseProduced>()?
            .last()
            .map_or(0, |produced| produced.rows);
        let mut values = ctx
            .samples::<Pulse>()
            .iter()
            .map(|row| row.value)
            .collect::<Vec<_>>();
        values.sort_by(f64::total_cmp);
        Ok(values.len() == produced as usize
            && values
                .iter()
                .enumerate()
                .all(|(index, value)| *value == index as f64))
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn main() -> std::io::Result<()> {
    dokime::plugin::run::<SuiteSidecar>()
}

/* === Tests === */
