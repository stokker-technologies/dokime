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
mod observers;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
// Third Party Imports
use dokime::harmos::operations::{OperationPolicy, RetryPolicy, ServingError, WireTable};
use dokime::harmos::plugin::contract::{Attachment, Declared};
use dokime::harmos::plugin::sidecar::{ObserverAttachment, Publisher};
// Local Imports
use dokime::{Refusal, Supervisor};
use error::Error;

/* === Constants === */
const POLICY: OperationPolicy = OperationPolicy {
    timeout: Duration::from_secs(5),
    attempt_timeout: Duration::from_secs(5),
    retries: RetryPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */
type Observing = ObserverAttachment<Resources, Publisher<TelemetrySidecar>>;

/* === Structs === */
#[dokime::section(
    label = "Fixture",
    description = "This fixture's whole settings card: a lifecycle marker path, test wiring rather than an operator knob."
)]
#[derive(Debug, Default, PartialEq)]
struct Config {
    lifecycle: Option<PathBuf>,
    record_lifecycle: Option<PathBuf>,
}

#[dokime::settings]
#[derive(Debug, Default, PartialEq)]
struct TelemetrySidecarSettings {
    config: Config,
}

#[dokime::row]
#[derive(Clone, PartialEq)]
pub struct Reading {
    count: u64,
    value: f64,
}

#[dokime::stream(id = "validation-telemetry.probe", window = 8, row = Reading)]
pub struct Probe;

pub(crate) struct Resources {
    pub(crate) record_lifecycle: Option<PathBuf>,
}

struct TelemetrySidecar {
    lifecycle: Option<PathBuf>,
    resources: Arc<Resources>,
    observers: Option<Arc<Observing>>,
}

struct RecordChecks;

struct CaptureStartup;

struct CheckHealth;

/* === Enums === */
#[dokime::catalog]
enum Operations {
    CheckHealth(CheckHealth),
    Start(observers::controls::Start),
    Stop(observers::controls::Stop),
}

#[dokime::catalog]
enum Implemented {
    CheckHealth(CheckHealth),
}

#[dokime::catalog]
pub enum Streams {
    Probe(Probe),
    FixtureRow(observers::continuous::FixtureRow),
}

#[dokime::catalog]
pub enum Records {
    Started(observers::record_demand::Started),
}

#[dokime::catalog]
enum Suites {
    RecordChecks(RecordChecks),
}

#[dokime::catalog]
enum Templates {
    CaptureStartup(CaptureStartup),
}

#[dokime::catalog]
enum Subscriptions {
    Started(observers::record_demand::Started),
}

/* === Traits === */

/* === Implementations === */
#[dokime::operation(
    id = "validation-telemetry.check_health",
    version = 1,
    description = "Answers whether the fixture is reachable; a dependent plugin calls it.",
    policy = POLICY,
    visibility = dokime::harmos::operations::Visibility::Shared
)]
impl CheckHealth {
    fn new(_: Arc<Resources>) -> Self {
        Self
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<(), Refusal> {
        Ok(())
    }
}

impl dokime::harmos::operations::lifecycle::Policy for TelemetrySidecar {
    const INITIALIZATION_POLICY: OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: OperationPolicy = POLICY;
}

#[dokime::plugin(
    id = "validation-telemetry",
    provides = Operations,
    requires(),
    contributions(
        dokime::Suites<Suites>,
        dokime::Records<Records>,
        dokime::Evidence<Streams>,
        dokime::Settings<TelemetrySidecarSettings>,
    ),
    streams(Probe, observers::continuous::FixtureRow),
    observers = observers::Observers,
    observer_controls(observers::controls::Start, observers::controls::Stop),
    restart = "never", description = "Publishes continuous telemetry for runtime ingest tests.")]
impl dokime::Plugin for TelemetrySidecar {
    type Config = dokime::plugin::Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = Resources;

    fn prepare(config: dokime::plugin::Configuration) -> Result<Self, Refusal> {
        let settings = dokime::plugin::refusal(config.settings::<TelemetrySidecarSettings>())?;
        Ok(Self {
            lifecycle: settings.config.lifecycle,
            resources: Arc::new(Resources {
                record_lifecycle: settings.config.record_lifecycle,
            }),
            observers: None,
        })
    }

    async fn initialize(&mut self, host: &Supervisor) -> Result<(), Refusal> {
        tracing::info!("telemetry fixture initialized");
        dokime::plugin::refusal(
            host.publisher::<Self>()
                .publish_on::<Probe>(&Reading {
                    count: u64::MAX,
                    value: 12.5,
                })
                .await
                .map_err(Error::from),
        )?;
        if let Some(path) = &self.lifecycle {
            dokime::plugin::refusal(std::fs::write(path, b"initialized"))?;
        }
        self.observers = Some(ObserverAttachment::from_catalog::<observers::Observers>(
            self.resources(),
            <Self as Declared>::STREAMS,
        )?);
        Ok(())
    }

    fn resources(&self) -> Arc<Resources> {
        Arc::clone(&self.resources)
    }

    fn bind_operations(&self, table: &mut WireTable<Supervisor>) -> Result<(), ServingError> {
        table.bind_operations::<Implemented, _>(self.resources())?;
        dokime::execution::serve::<Self, Suites>(table)?;
        self.observers
            .as_ref()
            .ok_or(ServingError::Catalog)?
            .bind::<observers::controls::Start, observers::controls::Stop>(table)
    }

    fn attachment(&self) -> Option<Arc<dyn Attachment<Supervisor>>> {
        self.observers
            .as_ref()
            .map(|observers| Arc::clone(observers) as Arc<dyn Attachment<Supervisor>>)
    }

    async fn shutdown(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        dokime::execution::interrupt().await;
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        self.observers.take();
        if let Some(path) = &self.lifecycle {
            assert!(
                dokime::plugin::refusal(std::env::current_exe())?.is_file(),
                "extraction must survive cleanup"
            );
            dokime::plugin::refusal(std::fs::write(path, b"finalized"))?;
        }
        Ok(())
    }
}

#[dokime::suite(id = "validation-telemetry.records", label = "Record Demand", templates = Templates)]
impl RecordChecks {}

#[dokime::template(id = "validation-telemetry.capture", label = "Capture Startup", subscribes = Subscriptions)]
impl CaptureStartup {
    async fn initialize(
        _: &mut dokime::execution::InitializationContext<'_>,
    ) -> Result<Self, Error> {
        Ok(Self)
    }

    #[dokime::step(label = "Hold Record Demand")]
    async fn hold(&mut self, _: &mut dokime::execution::StepContext<'_>) -> Result<(), Error> {
        std::future::pending().await
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn main() -> std::io::Result<()> {
    dokime::plugin::run::<TelemetrySidecar>()
}

/* === Tests === */
