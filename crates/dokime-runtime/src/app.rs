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
        Self { analysis: crate::report::analysis::Resources::new(), renderer: crate::report::Renderer, schema_settings }
    }
}

impl Application for Dokime {
    type State = RuntimeState;
    type Transaction = Transactions;
    type Record = Records;
    type Streams = ();
    type Resources = Resources;
}

/* === Public Functions === */

pub(crate) fn builder(origin: impl Into<Origin>, resources: Resources) -> Builder {
    let preparation_bytes = std::num::NonZeroUsize::new(8 * 1024 * 1024);
    let schema_settings = resources.schema_settings;
    HarmosRuntime::builder(origin, resources)
        .schema_settings(schema_settings)
        // Retain SDK Initialization While Admitting Its Nested Host Publications.
        .operation_settings(harmos::operations::Settings { preparation_bytes, ..Default::default() })
        .register(Transactions::catalog())
        .register(Records::catalog())
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
    let resources = Resources::new(Default::default());
    builder(origin, resources).entropy_seed(seed).simulate()
}

/* === Private Functions === */

/* === Tests === */
