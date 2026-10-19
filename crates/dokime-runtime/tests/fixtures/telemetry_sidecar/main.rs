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
use dokime_runtime::harmos::operations::{OperationPolicy, RestartPolicy, RetryPolicy, ServingError, WireTable};
use dokime_runtime::harmos::plugin::{Attachment, Declared};
use dokime_runtime::harmos::sidecar::{ObserverAttachment, Publisher};
// Local Imports
use dokime_runtime::{Refusal, Supervisor};
use error::Error;

/* === Constants === */
const POLICY: OperationPolicy = OperationPolicy {
    timeout: Duration::from_secs(5),
    attempt_timeout: Duration::from_secs(5),
    retries: RetryPolicy::Never,
    restart: RestartPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */
type Observing = ObserverAttachment<Resources, Publisher<TelemetrySidecar>>;

/* === Structs === */
#[dokime_runtime::section(
    label = "Fixture",
    description = "This fixture's whole settings card: a lifecycle marker path, test wiring rather than an operator knob."
)]
#[derive(Debug, Default, PartialEq)]
struct Config {
    lifecycle: Option<PathBuf>,
    record_lifecycle: Option<PathBuf>,
}

#[dokime_runtime::settings]
#[derive(Debug, Default, PartialEq)]
struct TelemetrySidecarSettings {
    config: Config,
}

#[dokime_runtime::row]
#[derive(Clone, PartialEq)]
pub struct Reading {
    count: u64,
    value: f64,
}

#[dokime_runtime::stream(id = "validation-telemetry.probe", window = 8, row = Reading)]
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

// The health request a dependent plugin calls; the request is its operation.
#[dokime_runtime::message]
#[dokime_runtime::operation(
    id = "validation-telemetry.check_health",
    response = (),
    version = 1,
    description = "Answers whether the fixture is reachable; a dependent plugin calls it.",
    policy = POLICY,
    visibility = dokime_runtime::harmos::operations::Visibility::Shared
)]
#[derive(Clone, PartialEq)]
struct CheckHealth {}

struct HealthCheck;

/* === Enums === */
#[dokime_runtime::catalog]
enum Operations {
    CheckHealth(CheckHealth),
}

#[dokime_runtime::catalog]
enum Implemented {
    HealthCheck(HealthCheck),
}

#[dokime_runtime::catalog]
pub enum Streams {
    Probe(Probe),
    FixtureRow(observers::continuous::FixtureRow),
}

#[dokime_runtime::catalog]
pub enum Records {
    Started(observers::record_demand::Started),
}

#[dokime_runtime::catalog]
enum Suites {
    RecordChecks(RecordChecks),
}

#[dokime_runtime::catalog]
enum Templates {
    CaptureStartup(CaptureStartup),
}

#[dokime_runtime::catalog]
enum Subscriptions {
    Started(observers::record_demand::Started),
}

/* === Traits === */

/* === Implementations === */
#[dokime_runtime::operation]
impl HealthCheck {
    async fn initialize(_: Arc<Resources>, _: &Supervisor) -> Self {
        Self
    }

    async fn run(&mut self, _: &Supervisor, _: &mut CheckHealth) -> Result<(), Refusal> {
        Ok(())
    }
}

impl dokime_runtime::harmos::operations::lifecycle::Policy for TelemetrySidecar {
    const INITIALIZATION_POLICY: OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: OperationPolicy = POLICY;
}

#[dokime_runtime::plugin(
    id = "validation-telemetry",
    provides = Operations,
    requires(),
    contributions(
        dokime_runtime::Suites<Suites>,
        dokime_runtime::Records<Records>,
        dokime_runtime::Evidence<Streams>,
        dokime_runtime::Settings<TelemetrySidecarSettings>,
    ),
    observers = observers::Observers,
    restart = "never", description = "Publishes continuous telemetry for runtime ingest tests.")]
impl dokime_runtime::Plugin for TelemetrySidecar {
    type Config = dokime_runtime::plugin::Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = Resources;

    fn prepare(config: dokime_runtime::plugin::Configuration) -> Result<Self, Refusal> {
        let settings = dokime_runtime::plugin::refusal(config.settings::<TelemetrySidecarSettings>())?;
        Ok(Self {
            lifecycle: settings.config.lifecycle,
            resources: Arc::new(Resources { record_lifecycle: settings.config.record_lifecycle }),
            observers: None,
        })
    }

    async fn initialize(&mut self, host: &Supervisor) -> Result<(), Refusal> {
        tracing::info!("telemetry fixture initialized");
        dokime_runtime::plugin::refusal(
            host.publisher::<Self>().publish_on::<Probe>(&Reading { count: u64::MAX, value: 12.5 }).await.map_err(Error::from),
        )?;
        if let Some(path) = &self.lifecycle {
            dokime_runtime::plugin::refusal(std::fs::write(path, b"initialized"))?;
        }
        self.observers = Some(ObserverAttachment::from_catalog::<observers::Observers>(self.resources(), <Self as Declared>::STREAMS)?);
        Ok(())
    }

    fn resources(&self) -> Arc<Resources> {
        Arc::clone(&self.resources)
    }

    fn bind_operations(&self, table: &mut WireTable<Supervisor>) -> Result<(), ServingError> {
        table.bind_operations::<Implemented, _>(self.resources())?;
        dokime_runtime::execution::serve::<Self, Suites>(table)?;
        self.observers.as_ref().ok_or(ServingError::Catalog)?.bind(table)
    }

    fn attachment(&self) -> Option<Arc<dyn Attachment<Supervisor>>> {
        self.observers.as_ref().map(|observers| Arc::clone(observers) as Arc<dyn Attachment<Supervisor>>)
    }

    async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        dokime_runtime::execution::interrupt().await;
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        self.observers.take();
        if let Some(path) = &self.lifecycle {
            assert!(dokime_runtime::plugin::refusal(std::env::current_exe())?.is_file(), "extraction must survive cleanup");
            dokime_runtime::plugin::refusal(std::fs::write(path, b"finalized"))?;
        }
        Ok(())
    }
}

#[dokime_runtime::suite(id = "validation-telemetry.records", label = "Record Demand", templates = Templates)]
impl RecordChecks {}

#[dokime_runtime::template(id = "validation-telemetry.records.capture", label = "Capture Startup", subscribes = Subscriptions)]
impl CaptureStartup {
    async fn initialize(_: &mut dokime_runtime::execution::InitializationContext<'_>) -> Result<Self, Error> {
        Ok(Self)
    }

    #[dokime_runtime::step(label = "Hold Record Demand")]
    async fn hold(&mut self, _: &mut dokime_runtime::execution::StepContext<'_>) -> Result<(), Error> {
        std::future::pending().await
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn main() -> std::io::Result<()> {
    dokime_runtime::plugin::run::<TelemetrySidecar>()
}

/* === Tests === */
