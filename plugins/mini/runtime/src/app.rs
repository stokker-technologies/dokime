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

// Local Imports
use crate::{Testbench, Transactions, error::LaunchError};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub type Origin = harmos::Origin<Testbench>;

pub type Builder = harmos::Builder<ChargerBench>;

pub type Runtime = harmos::Runtime<ChargerBench>;

pub type Handle = harmos::Handle<ChargerBench>;

pub type State = harmos::ManagedState<ChargerBench>;

#[cfg(feature = "sim")]
pub type Simulated = harmos::sim::SimulatedRuntime<ChargerBench>;

/* === Structs === */
#[derive(Clone, Copy, Debug)]
pub struct ChargerBench;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::Application for ChargerBench {
    type State = Testbench;
    type Transaction = Transactions;
    type Record = ();
    type Streams = ();
    type Resources = ();
}

/* === Public Functions === */
/* === Functions === */

pub fn builder(origin: impl Into<Origin>, resources: ()) -> Builder {
    Runtime::builder(origin, resources).register(Transactions::catalog())
}

#[cfg(not(target_arch = "wasm32"))]
#[tracing::instrument(level = "debug", skip_all)]
pub async fn launch(origin: impl Into<Origin>) -> Result<Runtime, LaunchError> {
    Ok(builder(origin, ()).runtime().await?)
}

#[cfg(feature = "sim")]
pub fn simulate(origin: impl Into<Origin>, seed: u64) -> Result<Simulated, harmos::Incompatible> {
    builder(origin, ()).entropy_seed(seed).simulate()
}

/* === Private Functions === */

/* === Tests === */
