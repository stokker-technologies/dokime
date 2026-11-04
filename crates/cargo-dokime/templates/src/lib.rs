/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub mod bench;
pub mod domain;
pub mod error;
pub mod listeners;
pub mod observers;
pub mod operations;
pub mod suites;

/* === Re-Exports === */

pub use error::{Error, Result};

/* === Imports === */

// Standard Library Imports
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime::harmos::operations::{OperationPolicy, RetryPolicy};
use dokime::harmos::plugin::{Attachment, Declared};
use dokime::harmos::sidecar::{ObserverAttachment, Publisher};
use dokime::prelude::*;

/* === Constants === */

const AMBIENT_C: f64 = 22.0;

const DEFAULT_PORT: &str = "/dev/ttyUSB0";

const LIFECYCLE: OperationPolicy = OperationPolicy {
    timeout: Duration::from_secs(5),
    attempt_timeout: Duration::from_secs(5),
    cancellation_timeout: std::time::Duration::ZERO,
    retries: RetryPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

type Observing = ObserverAttachment<Publisher<Plugin>>;

/* === Structs === */

pub struct Bench {
    pub sensor: bench::Sensor,
    pub upload_configured: bool,
    units: Mutex<Vec<String>>,
}

pub struct Plugin {
    bench: Arc<Bench>,
    observers: Option<Arc<Observing>>,
}

/* === Enums === */

#[dokime::catalog]
pub enum Records {}

/* === Traits === */

/* === Implementations === */

impl Bench {
    pub(crate) fn observed(&self, unit: String) {
        self.units.lock().unwrap_or_else(PoisonError::into_inner).push(unit);
    }

    pub fn connected(&self) -> Vec<String> {
        self.units.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }
}

impl dokime::harmos::builtins::lifecycle::Policy for Plugin {
    const INITIALIZATION_POLICY: OperationPolicy = LIFECYCLE;
    const SHUTDOWN_POLICY: OperationPolicy = LIFECYCLE;
}

#[dokime::plugin(
    id = "{{crate}}",
    requires(),
    contributions(
        dokime::Suites<suites::Suites>,
        dokime::Records<Records, listeners::Listeners>,
        dokime::Evidence<observers::Streams>,
        dokime::Settings<domain::model::settings::BenchSettings>,
    ),
    observers = observers::Observers,
    restart = "never", description = "A thermal bench: one plugin, one suite, one template that holds a chamber at temperature and decides whether it stayed there.\nThe identity, the operations it serves and calls, and the contributions it makes to the host are literal data on the attribute: they are what the host reads *before* the plugin runs, out of a binary it never executes. A template calls `ReadChamber` through the host; the plugin serves it privately, so its own runs may call it without naming it in `requires(...)`, which lists only what other plugins or the host provide.")]
impl dokime::Plugin for Plugin {
    const CATALOGS: &'static [dokime::harmos::Catalog] = &[domain::operations::Operations::CATALOG];
    fn attachment(&self) -> Option<Arc<dyn Attachment<Supervisor>>> {
        self.observers.as_ref().map(|observers| Arc::clone(observers) as Arc<dyn Attachment<Supervisor>>)
    }
}

impl dokime::harmos::Lifecycle for Plugin {
    type Config = Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    async fn prepare(config: Configuration, setup: &mut dokime::harmos::Setup<Self>) -> std::result::Result<Self, Refusal> {
        let mut owner = {
            let settings = dokime::plugin::refusal(config.settings::<domain::model::settings::BenchSettings>())?;
            // The port is discovered, so it is absent until an operator saves what
            // the card offered them: a plugin has to open before discovery can run,
            // which is exactly why a discovered field is an `Option<T>`.
            let port = match settings.chamber.sensor_port {
                Some(saved) => saved.display().to_string(),
                None => DEFAULT_PORT.to_owned(),
            };
            tracing::info!("opening the bench sensor on {port}");
            Self {
                bench: Arc::new(Bench {
                    sensor: bench::Sensor::open(port, AMBIENT_C),
                    units: Mutex::default(),
                    // A missing optional store identifier decodes as `None`, which is how "not
                    // configured" is told apart from "configured and empty".
                    upload_configured: settings.chamber.cloud_token.is_some(),
                }),
                observers: None,
            }
        };
        setup.resource::<Bench>(owner.resources());
        owner.observers = Some(ObserverAttachment::from_catalog::<observers::Observers>(<Self as Declared>::STREAMS)?);
        dokime::execution::prepare::<Self, suites::Suites>(setup)?;
        Ok(owner)
    }

    async fn stop(&mut self, _: &Supervisor) -> std::result::Result<(), Refusal> {
        dokime::execution::interrupt().await;
        Ok(())
    }

    async fn cleanup(&mut self) -> std::result::Result<(), Refusal> {
        self.observers.take();
        self.bench.sensor.close();
        Ok(())
    }
}

impl Plugin {
    fn resources(&self) -> Arc<Bench> {
        Arc::clone(&self.bench)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
