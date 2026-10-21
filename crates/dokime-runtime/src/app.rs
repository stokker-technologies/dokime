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

// Third Party Imports
use harmos::{Application, state::Origin as HarmosOrigin};

// Local Imports
use crate::config::Config;
use crate::runtime_state::RuntimeState;
use crate::streams::Streams as StreamCatalog;
use crate::transactions::Transactions;
use dokime_domain::state::records::Records;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type Origin = HarmosOrigin<RuntimeState>;
pub(crate) type Builder = harmos::Builder<Dokime>;
pub(crate) type HarmosRuntime = harmos::Runtime<Dokime>;
pub(crate) type StateManager = harmos::state::StateManager<Dokime>;
pub(crate) type Streams = harmos::streams::StreamManager<Dokime>;

#[cfg(feature = "simulation")]
pub type Simulated = harmos::simulation::SimulatedRuntime<Dokime>;

/* === Structs === */

#[derive(Clone, Copy, Debug)]
pub struct Dokime;

#[derive(Clone)]
pub struct Resources {
    pub(crate) analysis: crate::report::analysis::Resources,
    pub(crate) renderer: crate::report::Renderer,
    pub(crate) storage_settings: crate::config::StorageSettings,
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
    pub fn new(storage_settings: crate::config::StorageSettings) -> Self {
        Self { analysis: crate::report::analysis::Resources::new(), renderer: crate::report::Renderer, storage_settings }
    }
}

impl Application for Dokime {
    type State = RuntimeState;
    type Transaction = Transactions;
    type Record = Records;
    type Streams = StreamCatalog;
    type Resources = Resources;
}

/* === Public Functions === */

pub(crate) fn builder(origin: impl Into<Origin>, resources: Resources) -> Builder {
    HarmosRuntime::builder(origin, resources)
        // Retain SDK Initialization While Admitting Its Nested Host Publications.
        .operation_settings(harmos::operations::Settings {
            preparation_bytes: std::num::NonZeroUsize::new(8 * 1024 * 1024),
            ..Default::default()
        })
        .register(Transactions::catalog())
        .register(Records::catalog())
        .register(StreamCatalog::catalog())
        .register(Lanes::catalog())
}

#[tracing::instrument(
    level = "debug",
    skip_all,
    fields(
        output_root = %config.output_root.display(),
        seed_plugins = config.seed_plugins,
    )
)]
pub async fn launch(config: Config) -> crate::runtime::Result<crate::Runtime> {
    let (runtime_config, mut startup) = config.into_runtime_config();
    if let Some(brand) = &mut startup.brand {
        brand.materialize_logos(&runtime_config.paths().cache().join("brand"))?;
    }
    let runtime = crate::Runtime::start(runtime_config).await?;
    let handle = runtime.handle();
    handle.set_brand(startup.brand);
    if startup.native_host_telemetry {
        // Binding declares the native host source. Hardware collection starts
        // only when an execution requests a host capture.
        handle.bind_native_host_telemetry();
    }
    Ok(runtime)
}

#[cfg(feature = "simulation")]
#[tracing::instrument(level = "debug", skip_all, fields(seed))]
pub fn simulate(origin: impl Into<Origin>, seed: u64) -> Result<Simulated, harmos::runtime::Incompatible> {
    builder(origin, Resources::new(crate::config::storage_defaults())).entropy_seed(seed).simulate()
}

/* === Private Functions === */

/* === Tests === */
