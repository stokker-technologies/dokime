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
use std::sync::Arc;

// Third Party Imports
use async_trait::async_trait;
use harmos::sidecar::StreamDeclaration;

// Local Imports
use dokime_runtime::execution::Runnable;
use dokime_runtime::schema::Described;
use dokime_runtime::runtime::EmbeddedProvider;
use dokime_runtime::InstalledPlugin;
use dokime_runtime::runtime::Result;

/* === Constants === */
/// The plugin source and suite the smoke check registers under.
pub(crate) const SOURCE: &str = "fixture-suites";
pub(crate) const SUITE: &str = "fixture.checks";

/// The stream the pulse plot draws.
pub(crate) const PULSE_STREAM: &str = "fixture-suites.pulse";

/// The time series plot over the pulse stream.
pub(crate) const PULSE_PLOT: &str = "fixture.checks.smoke.pulse";

/// Points the pulse plot declares it shows, whatever the capture's length.
pub(crate) const PULSE_DISPLAY_POINTS: u32 = 400;

/// Sample rate the spectrum plot transforms its rows at.
const SPECTRUM_SAMPLE_RATE_HZ: f64 = 100.0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// What one run of the in-process smoke check produces, and the signals the story drives it by.
pub(crate) struct Behaviour {
    pulse_rows: u32,
    spectrum_rows: u32,
    hold: bool,
    released: tokio::sync::watch::Sender<bool>,
}

/// Serves the smoke suite in process, as the plugin fixture used to over its sidecar.
pub(crate) struct Smoke {
    behaviour: Arc<Behaviour>,
}

struct Checks;

struct Check {
    behaviour: Arc<Behaviour>,
}

// In-Process Pushes Have No IPC Backpressure, So The Window Holds Every Row The
// Largest Story Pushes Before Ingest Takes Them In.
#[dokime_runtime::stream(
    id = "fixture-suites.pulse",
    window = 32_768,
    label = "Fixture Pulse",
    description = "A stream the smoke step fills only when asked to."
)]
struct Pulse {
    value: f64,
}

#[dokime_runtime::stream(
    id = "fixture-suites.spectrum",
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
    /// Rows the step pushes onto each stream, and whether it then holds until released.
    pub(crate) fn new(pulse_rows: u32, spectrum_rows: u32, hold: bool) -> Self {
        Self {
            pulse_rows,
            spectrum_rows,
            hold,
            released: tokio::sync::watch::Sender::new(false),
        }
    }

    /// Lets A Held Step Finish.
    pub(crate) fn release(&self) {
        self.released.send_replace(true);
    }
}

impl Smoke {
    pub(crate) fn new(behaviour: Behaviour) -> Self {
        Self {
            behaviour: Arc::new(behaviour),
        }
    }

    pub(crate) fn behaviour(&self) -> Arc<Behaviour> {
        Arc::clone(&self.behaviour)
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
        plot.y(dokime_runtime::expressions::telemetry(
            "fixture-suites.pulse",
            "value",
        ));
        plot.display_points = Some(PULSE_DISPLAY_POINTS);
        plot.downsample = dokime_runtime::document::DownsampleTechnique::MinMax;
        let mut section = dokime_runtime::prelude::Section::new("Pulse");
        section.plot(plot);
        let mut spectrum = dokime_runtime::prelude::Plot::fft("Spectrum");
        spectrum.id("fixture.checks.smoke.spectrum");
        spectrum.y(dokime_runtime::expressions::telemetry(
            "fixture-suites.spectrum",
            "value",
        ));
        spectrum.fft.sample_rate_hz = Some(SPECTRUM_SAMPLE_RATE_HZ);
        section.plot(spectrum);
        let mut document = dokime_runtime::prelude::Document::new("default", "Fixture Smoke");
        document.section(section);
        document
    }

    async fn initialize(
        ctx: &mut dokime_runtime::execution::InitializationContext<'_>,
    ) -> dokime_runtime::Result<Self> {
        let behaviour = Arc::clone(ctx.environment().binding::<Arc<Behaviour>>());
        Ok(Self { behaviour })
    }

    #[dokime_runtime::step(label = "Check")]
    async fn check(&mut self, ctx: &mut dokime_runtime::execution::StepContext<'_>) -> dokime_runtime::Result<()> {
        let behaviour = Arc::clone(&self.behaviour);

        // Publish The Requested Rows
        if behaviour.pulse_rows > 0 {
            let pulse = ctx.stream::<Pulse>().open();
            for index in 0..behaviour.pulse_rows {
                pulse
                    .push(Pulse {
                        value: f64::from(index),
                    })
                    .await;
            }
        }
        if behaviour.spectrum_rows > 0 {
            let spectrum = ctx.stream::<Spectrum>().open();
            for index in 0..behaviour.spectrum_rows {
                spectrum
                    .push(Spectrum {
                        value: f64::from(index % 16),
                    })
                    .await;
            }
        }
        ctx.publish(PulseProduced {
            rows: behaviour.pulse_rows,
        })
        .await;

        // Hold Until The Story Releases The Step
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

#[async_trait]
impl EmbeddedProvider for Smoke {
    fn catalog(&self) -> InstalledPlugin {
        InstalledPlugin {
            source: SOURCE.into(),
            version: "0.1.0".into(),
            enabled: true,
            suites: dokime_runtime::definition::SuiteSet::new([Checks::definition()]),
            ..Default::default()
        }
    }

    fn streams(&self) -> Vec<StreamDeclaration> {
        vec![declaration::<Pulse>(), declaration::<Spectrum>()]
    }

    async fn start(&self) -> Result<Vec<Arc<dyn Runnable>>> {
        let binding: Arc<dyn std::any::Any + Send + Sync> = Arc::new(self.behaviour());
        Ok(vec![
            Arc::new(dokime_runtime::execution::Workflow::<Check>::new(SUITE, SOURCE).bound(binding))
                as Arc<dyn Runnable>,
        ])
    }

    async fn stop(&self) -> Result<()> {
        self.behaviour.release();
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn declaration<R>() -> StreamDeclaration
where
    R: harmos::StreamSpec + harmos::RowMetadata + Described,
{
    StreamDeclaration {
        metadata: <R as harmos::StreamSpec>::metadata(),
        row_metadata: <R as harmos::RowMetadata>::row_metadata(),
        id: <R as harmos::StreamSpec>::ID.into(),
        schema: <R as harmos::StreamSpec>::VERSION,
        window: <R as harmos::StreamSpec>::WINDOW,
        row: harmos::schema::graph::Schema::from_declaration(
            <R as Described>::DEFINITION,
            &harmos::config::SchemaSettings::default(),
        )
        .unwrap(),
    }
}

/* === Tests === */
