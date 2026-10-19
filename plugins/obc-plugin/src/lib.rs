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
use dokime_runtime::harmos::operations::{OperationPolicy, RestartPolicy, RetryPolicy, ServingError, WireTable};
use dokime_runtime::harmos::plugin::{Attachment, Declared};
use dokime_runtime::harmos::sidecar::{ObserverAttachment, Publisher};
use dokime_runtime::prelude::*;
use obc_runtime::app;
use obc_runtime::{Stop, Testbench};

/* === Constants === */
pub const SOURCE: &str = <ObcPlugin as Declared>::ID;

const LIFECYCLE: OperationPolicy = OperationPolicy {
    timeout: Duration::from_secs(5),
    attempt_timeout: Duration::from_secs(5),
    retries: RetryPolicy::Never,
    restart: RestartPolicy::Never,
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
#[dokime_runtime::catalog]
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

impl dokime_runtime::harmos::operations::lifecycle::Policy for ObcPlugin {
    const INITIALIZATION_POLICY: OperationPolicy = LIFECYCLE;
    const SHUTDOWN_POLICY: OperationPolicy = LIFECYCLE;
}

#[dokime_runtime::plugin(
    id = "obc-plugin",
    provides = operations::Operations,
    requires(),
    contributions(
        dokime_runtime::Suites<suites::Suites>,
        dokime_runtime::Records<Records, listeners::Listeners>,
        dokime_runtime::Evidence<observers::system::Streams, analyses::Analyses>,
        dokime_runtime::Settings<BenchSettings>,
    ),
    observers = observers::Observers,
    restart(
        credit = 3,
        sustained_health_ms = 30_000,
        backoff_base_ms = 250,
        backoff_cap_ms = 10_000
    )
, description = "Supervised plugin hosting Dokime's charger testbench workflows.")]
impl dokime_runtime::Plugin for ObcPlugin {
    type Config = Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = Resources;

    fn prepare(config: Configuration) -> Result<Self, Refusal> {
        let settings = dokime_runtime::plugin::refusal(config.settings::<BenchSettings>())?;
        Ok(Self {
            runtime: None,
            resources: Arc::new(Resources { app: OnceLock::new(), connections: Mutex::default(), settings }),
            observers: None,
        })
    }

    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        let runtime = dokime_runtime::plugin::refusal(app::launch(Testbench::default()).await)?;
        let _ = self.resources.app.set(obc_runtime::Client::new(runtime.handle()));
        self.runtime = Some(runtime);
        self.observers = Some(ObserverAttachment::from_catalog::<observers::Observers>(self.resources(), <Self as Declared>::STREAMS)?);
        Ok(())
    }

    fn resources(&self) -> Arc<Resources> {
        Arc::clone(&self.resources)
    }

    fn bind_operations(&self, table: &mut WireTable<Supervisor>) -> Result<(), ServingError> {
        table.bind_operations::<operations::Implemented, _>(self.resources())?;
        table.bind_operations::<analyses::Implemented, _>(self.resources())?;
        table.bind_operations::<listeners::Listeners, _>(self.resources())?;
        dokime_runtime::execution::serve::<Self, suites::Suites>(table)?;
        self.observers.as_ref().ok_or(ServingError::Catalog)?.bind(table)
    }

    fn attachment(&self) -> Option<Arc<dyn Attachment<Supervisor>>> {
        self.observers.as_ref().map(|observers| Arc::clone(observers) as Arc<dyn Attachment<Supervisor>>)
    }

    async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        dokime_runtime::execution::interrupt().await;
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
