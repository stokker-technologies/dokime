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
}

impl dokime::harmos::Lifecycle for ChargerBench {
    type Config = ();
    type Error = std::io::Error;
    type Context = dokime::harmos::Startup<Self>;
    async fn prepare(_: (), setup: &mut dokime::harmos::Setup<Self>) -> Result<Self, Self::Error> {
        setup.settings().register(Transactions::catalog());
        Ok(Self)
    }
    async fn initialize(&mut self, _: &Self::Context) -> Result<(), Self::Error> {
        Ok(())
    }
    async fn stop(&mut self, _: &Self::Context) -> Result<(), Self::Error> {
        Ok(())
    }
    async fn cleanup(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[tracing::instrument(level = "debug", skip_all)]
pub async fn launch(origin: impl Into<Origin>) -> Result<Runtime, LaunchError> {
    Ok(Runtime::launch(origin, ()).await?)
}
