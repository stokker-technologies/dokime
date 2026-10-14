/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
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
pub use suites::{
    DEFAULT_SAMPLE_COUNT, DEFAULT_TOLERANCE, MiniChecks, Pulse, PulseRow, SAMPLE_INTERVAL_MS,
    pulse_value,
};

/* === Imports === */
// Standard Library Imports
use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime::harmos::operations::{
    OperationPolicy, RestartPolicy, RetryPolicy, ServingError, WireTable,
};
use dokime::harmos::plugin::contract::{Attachment, Declared};
use dokime::harmos::plugin::sidecar::{ObserverAttachment, Publisher};
use dokime::prelude::*;
use mini_runtime::app;
use mini_runtime::{Stop, Testbench};

/* === Constants === */
pub const SOURCE: &str = <MiniSidecar as Declared>::ID;

const LIFECYCLE: OperationPolicy = OperationPolicy {
    timeout: Duration::from_secs(5),
    attempt_timeout: Duration::from_secs(5),
    retries: RetryPolicy::Never,
    restart: RestartPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */
type Observing = ObserverAttachment<Resources, Publisher<MiniSidecar>>;

/* === Structs === */
pub struct Resources {
    app: OnceLock<mini_runtime::Client>,
    connections: Mutex<Vec<String>>,
    pub settings: BenchSettings,
}

pub struct MiniSidecar {
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
    pub fn app(&self) -> &mini_runtime::Client {
        self.app
            .get()
            .expect("the bench runtime launches before any operation is bound")
    }

    pub(crate) fn observed(&self, resource: String) {
        self.connections
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(resource);
    }

    pub fn connected(&self) -> Vec<String> {
        self.connections
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl dokime::harmos::operations::lifecycle::Policy for MiniSidecar {
    const INITIALIZATION_POLICY: OperationPolicy = LIFECYCLE;
    const SHUTDOWN_POLICY: OperationPolicy = LIFECYCLE;
}

#[dokime::plugin(
    id = "mini-sidecar",
    provides = operations::Operations,
    requires(),
    contributions(
        dokime::Suites<suites::Suites>,
        dokime::Records<Records, listeners::Listeners>,
        dokime::Evidence<observers::system::Streams>,
        dokime::Settings<BenchSettings>,
    ),
    streams(
        PulseRow,
        observers::system::SystemRow,
        suites::startup::Samples,
        suites::steady::Samples,
        suites::steady::BenchEvent,
        suites::protection::Samples,
        suites::recovery::Samples,
    ),
    observers = observers::Observers,
    observer_control = observers::controls::Observe,
    restart(
        credit = 3,
        sustained_health_ms = 30_000,
        backoff_base_ms = 250,
        backoff_cap_ms = 10_000
    )
, description = "Supervised plugin hosting Dokime's charger testbench workflows.")]
impl dokime::Plugin for MiniSidecar {
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
        let _ = self
            .resources
            .app
            .set(mini_runtime::Client::new(runtime.handle()));
        self.runtime = Some(runtime);
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
        table.bind_operations::<operations::Implemented, _>(self.resources())?;
        table.bind_operations::<listeners::Listeners, _>(self.resources())?;
        dokime::execution::serve::<Self, suites::Suites>(table)?;
        self.observers
            .as_ref()
            .ok_or(ServingError::Catalog)?
            .bind::<observers::controls::Observe>(table)
    }

    fn attachment(&self) -> Option<Arc<dyn Attachment<Supervisor>>> {
        self.observers
            .as_ref()
            .map(|observers| Arc::clone(observers) as Arc<dyn Attachment<Supervisor>>)
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
