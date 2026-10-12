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
use harmos::{Application, Origin as HarmosOrigin};
// Local Imports
use crate::config::Config;
use crate::records::Records;
use crate::runtime_state::RuntimeState;
use crate::streams::Streams as StreamCatalog;
use crate::transactions::Transactions;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type Origin = HarmosOrigin<RuntimeState>;

pub(crate) type Builder = harmos::Builder<Dokime>;

pub(crate) type HarmosRuntime = harmos::Runtime<Dokime>;

pub(crate) type StateManager = harmos::StateManager<Dokime>;

pub(crate) type Streams = harmos::StreamManager<Dokime>;

#[cfg(feature = "sim")]
pub type Simulated = harmos::sim::SimulatedRuntime<Dokime>;

/* === Structs === */

#[derive(Clone, Copy, Debug)]
pub struct Dokime;

/* === Enums === */
#[harmos::lanes]
pub(crate) enum Lanes {
    #[default]
    Open,
    #[lane(serial)]
    Execution,
}

/* === Traits === */

/* === Implementations === */
impl Application for Dokime {
    type State = RuntimeState;
    type Transaction = Transactions;
    type Record = Records;
    type Streams = StreamCatalog;
    type Resources = ();
}

/* === Public Functions === */
/* === Functions === */

pub(crate) fn builder(origin: impl Into<Origin>, resources: ()) -> Builder {
    HarmosRuntime::builder(origin, resources)
        // Retain SDK Initialization While Admitting Its Nested Host Publications.
        .operation_preparation_bytes(std::num::NonZeroUsize::new(8 * 1024 * 1024).unwrap())
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
pub async fn launch(config: Config) -> crate::Result<crate::Runtime> {
    let (runtime_config, mut startup) = config.into_runtime_config();
    if let Some(brand) = &mut startup.brand {
        brand.materialize_logo(&runtime_config.paths().cache().join("brand"))?;
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

#[cfg(feature = "sim")]
#[tracing::instrument(level = "debug", skip_all, fields(seed))]
pub fn simulate(origin: impl Into<Origin>, seed: u64) -> Result<Simulated, harmos::Incompatible> {
    builder(origin, ()).entropy_seed(seed).simulate()
}

/* === Private Functions === */

/* === Tests === */
