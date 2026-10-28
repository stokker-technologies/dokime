/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub mod analyses;
pub mod domain;
pub mod error;
mod listeners;
mod observers;
pub mod operations;
pub mod prompts;
pub mod settings;
pub mod suites;

/* === Re-Exports === */

pub use error::Error;
pub use settings::BenchSettings;
pub use suites::checks::{DEFAULT_SAMPLE_COUNT, DEFAULT_TOLERANCE, ObcChecks, Pulse, PulseRow, SAMPLE_INTERVAL_MS, pulse_value};

/* === Imports === */

// Standard Library Imports
use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime::harmos::Dependencies;
use dokime::harmos::operations::{OperationPolicy, RetryPolicy, ServingError, WireTable};
use dokime::harmos::plugin::{Attachment, Declared};
use dokime::harmos::sidecar::{ObserverAttachment, Publisher};
use dokime::prelude::*;
use obc_runtime::app;
use obc_runtime::{Stop, Testbench};

/* === Constants === */

pub const SOURCE: &str = <ObcPlugin as Declared>::ID;
const LIFECYCLE: OperationPolicy = OperationPolicy {
    timeout: Duration::from_secs(5),
    attempt_timeout: Duration::from_secs(5),
    retries: RetryPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

type Observing = ObserverAttachment<Resources, Publisher<ObcPlugin>>;

/* === Structs === */

pub struct Resources {
    app: OnceLock<obc_runtime::Client>,
    connections: Mutex<Vec<String>>,
    pub settings: BenchSettings,
}

pub struct ObcPlugin {
    runtime: Option<app::Runtime>,
    resources: Arc<Resources>,
    observers: Option<Arc<Observing>>,
}

/* === Enums === */

#[dokime::catalog]
pub enum Records {}

/* === Traits === */

/* === Implementations === */

impl Resources {
    pub fn app(&self) -> &obc_runtime::Client {
        self.app.get().expect("the bench runtime launches before any operation is bound")
    }

    pub(crate) fn observed(&self, resource: String) {
        self.connections.lock().unwrap_or_else(PoisonError::into_inner).push(resource);
    }

    pub fn connected(&self) -> Vec<String> {
        self.connections.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }
}

impl dokime::harmos::builtins::lifecycle::Policy for ObcPlugin {
    const INITIALIZATION_POLICY: OperationPolicy = LIFECYCLE;
    const SHUTDOWN_POLICY: OperationPolicy = LIFECYCLE;
}

#[dokime::plugin(
    id = "obc",
    provides = operations::Operations,
    requires(),
    contributions(
        dokime::Suites<suites::Suites>,
        dokime::Records<Records, listeners::Listeners>,
        dokime::Evidence<observers::system::Streams, analyses::Analyses>,
        dokime::Settings<BenchSettings>,
    ),
    observers = observers::Observers,
    restart(
        credit = 3,
        sustained_health_ms = 30_000,
        backoff_base_ms = 250,
        backoff_cap_ms = 10_000
    )
, description = "Supervised plugin hosting Dokime's charger testbench workflows.")]
impl dokime::Plugin for ObcPlugin {
    type Config = Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = Resources;

    fn prepare(config: Configuration) -> Result<Self, Refusal> {
        let settings = dokime::plugin::refusal(config.settings::<BenchSettings>())?;
        Ok(Self {
            runtime: None,
            resources: Arc::new(Resources {
                app: OnceLock::new(),
                connections: Mutex::default(),
                settings,
            }),
            observers: None,
        })
    }

    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        let runtime = dokime::plugin::refusal(app::launch(Testbench::default()).await)?;
        let _ = self.resources.app.set(obc_runtime::Client::new(runtime.handle()));
        self.runtime = Some(runtime);
        self.observers = Some(ObserverAttachment::from_catalog::<observers::Observers>(self.resources(), <Self as Declared>::STREAMS)?);
        Ok(())
    }

    fn resources(&self) -> Arc<Resources> {
        Arc::clone(&self.resources)
    }

    fn bind_operations(&self, table: &mut WireTable) -> Result<(), ServingError> {
        // Operations reach the bench through the resources they declare.
        let mut dependencies = Dependencies::new();
        dependencies.insert(self.resources()).expect("each dependency is inserted once");
        table.register(operations::Operations::registration(dependencies.clone()))?;
        table.register(listeners::Listeners::registration(dependencies))?;
        dokime::execution::serve::<Self, suites::Suites>(table)?;
        self.observers.as_ref().ok_or(ServingError::Catalog)?.bind(table)
    }

    fn attachment(&self) -> Option<Arc<dyn Attachment<Supervisor>>> {
        self.observers.as_ref().map(|observers| Arc::clone(observers) as Arc<dyn Attachment<Supervisor>>)
    }

    async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        dokime::execution::interrupt().await;
        if let Some(runtime) = &self.runtime
            && let Err(error) = runtime.state.commit(Stop).await
        {
            tracing::warn!(%error, "the charger bench runtime did not commit its stop transaction");
        }
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        self.observers.take();
        if let Some(runtime) = self.runtime.take()
            && let Err(error) = runtime.stop().await
        {
            tracing::error!(%error, "the charger bench runtime failed to shut down");
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
