/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::{Testbench, Transactions, error::LaunchError};

pub type Origin = dokime::harmos::state::Origin<Testbench>;
pub type Builder = dokime::harmos::Builder<ChargerBench>;
pub type Runtime = dokime::harmos::Runtime<ChargerBench>;
pub type Handle = dokime::harmos::runtime::Handle<ChargerBench>;
pub type State = dokime::harmos::state::ManagedState<ChargerBench>;

#[derive(Clone, Copy, Debug)]
pub struct ChargerBench;

// The bench admits all work on one lane.
#[dokime::harmos::lanes(harmos_path = dokime::harmos)]
pub enum Lanes {
    #[default]
    Bench,
}

impl dokime::harmos::Application for ChargerBench {
    type Lanes = Lanes;
    type State = Testbench;
    type Transaction = Transactions;
    type Record = ();
    type Streams = ();
    type Resources = ();
}

pub fn builder(origin: impl Into<Origin>, resources: ()) -> Builder {
    Runtime::builder(origin, resources).register(Transactions::catalog())
}

#[cfg(not(target_arch = "wasm32"))]
#[tracing::instrument(level = "debug", skip_all)]
pub async fn launch(origin: impl Into<Origin>) -> Result<Runtime, LaunchError> {
    Ok(builder(origin, ()).runtime().await?)
}
