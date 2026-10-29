/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod composition;
mod lifecycle;

/* === Re-Exports === */

pub use composition::{Composition, LiveRuntime, live_runtime};

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::{Application, state::Origin as HarmosOrigin};

// Local Imports
use crate::config::Config;
use crate::runtime_state::RuntimeState;
use crate::transactions::Transactions;
use dokime_domain::state::records::Records;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type Origin = HarmosOrigin<RuntimeState>;
pub(crate) type HarmosRuntime = harmos::Runtime<Dokime>;
pub(crate) type StateManager = harmos::state::StateManager<Dokime>;
pub(crate) type Streams = harmos::streams::StreamManager<Dokime>;
pub(crate) type OperationManager = harmos::operations::OperationManager<Lanes>;

#[cfg(feature = "simulation")]
pub type Simulated = harmos::simulation::SimulatedRuntime<Dokime>;

/* === Structs === */

pub struct Dokime {
    parts: Option<crate::runtime::handle::BootParts>,
    prepared: Option<crate::runtime::handle::Prepared>,
    live: LiveRuntime,
    composition: Vec<Composition>,
}

#[derive(serde::Deserialize)]
pub struct LaunchConfig {
    pub(crate) config: crate::config::RuntimeConfig,
    pub(crate) record_namespace: dokime_domain::model::identity::SessionId,

    #[serde(skip)]
    pub(crate) providers: std::collections::BTreeMap<String, std::sync::Arc<dyn crate::execution_runtime::provider::ExecutionFixture>>,

    #[serde(skip)]
    pub(crate) live: LiveRuntime,

    #[serde(skip)]
    pub(crate) boot: std::sync::Arc<std::sync::Mutex<crate::runtime::rollback::Boot>>,

    #[cfg(feature = "fault-injection")]
    #[serde(skip)]
    pub(crate) decorate: Option<crate::runtime::handle::BackendDecorator>,
}

#[derive(Clone)]
pub struct Resources {
    pub(crate) schema_settings: harmos::config::SchemaSettings,
    pub(crate) analysis: crate::report::analysis::Resources,
    pub(crate) renderer: crate::report::Renderer,
}

/* === Enums === */

#[harmos::lanes]
pub enum Lanes {
    #[default]
    Open,

    #[lane(serial)]
    Execution,
}

/* === Traits === */

/* === Implementations === */

impl Resources {
    pub fn new(schema_settings: harmos::config::SchemaSettings) -> Self {
        Self {
            analysis: crate::report::analysis::Resources::new(),
            renderer: crate::report::Renderer,
            schema_settings,
        }
    }
}

impl Application for Dokime {
    type Lanes = Lanes;
    type State = RuntimeState;
    type Transaction = Transactions;
    type Record = Records;
    type Streams = ();
}

/* === Public Functions === */

#[tracing::instrument(
    level = "debug",
    skip_all,
    fields(
        output_root = %config.output_root.display(),
    )
)]
pub async fn launch(config: Config) -> crate::runtime::Result<crate::Runtime> {
    launch_with(config, Vec::new()).await
}

pub async fn launch_with(config: Config, composition: Vec<Composition>) -> crate::runtime::Result<crate::Runtime> {
    let (runtime_config, mut startup) = config.into_runtime_config();
    let mut runtime_config = runtime_config;
    runtime_config.composition = composition;
    if let Some(brand) = &mut startup.brand {
        brand.materialize_logos(&runtime_config.application.cache().join("brand"))?;
    }
    let runtime = crate::Runtime::start(runtime_config).await?;
    let handle = runtime.handle();
    handle.set_brand(startup.brand);
    Ok(runtime)
}

#[cfg(feature = "simulation")]
#[tracing::instrument(level = "debug", skip_all, fields(seed))]
pub fn simulate(origin: impl Into<Origin>, seed: u64) -> Result<Simulated, harmos::StartupError> {
    HarmosRuntime::simulate(origin, None, seed)
}

/* === Private Functions === */

/* === Tests === */
