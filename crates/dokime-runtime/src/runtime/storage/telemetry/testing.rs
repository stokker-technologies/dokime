/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Saved-Window And Follow-Frontier Volume Fixtures

/* === Modules === */

#[cfg(feature = "testing")]
mod history;

/* === Re-Exports === */

#[cfg(feature = "testing")]
pub use history::telemetry;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use harmos::Application;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct TestApp;

#[derive(Default, serde::Deserialize)]
pub(super) struct StorageConfig {
    #[serde(skip)]
    pub(super) storage: Option<crate::storage::Manager>,
}

#[harmos::message]
pub(super) struct Reading {
    pub(super) value: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Application for TestApp {
    type Lanes = crate::app::Lanes;
    type State = ();
    type Transaction = harmos::state::Never;
    type Record = harmos::state::Never;
    type Streams = ();
}

impl harmos::Lifecycle for TestApp {
    type Config = StorageConfig;
    type Error = harmos::Error;
    type Context = harmos::Startup<Self>;

    async fn prepare(config: Self::Config, setup: &mut harmos::Setup<Self>) -> harmos::Result<Self> {
        setup.settings().storage(config.storage.as_ref().expect("storage fixture is configured"));
        Ok(Self)
    }
    async fn initialize(&mut self, _: &Self::Context) -> harmos::Result<()> {
        Ok(())
    }
    async fn stop(&mut self, _: &Self::Context) -> harmos::Result<()> {
        Ok(())
    }
    async fn cleanup(&mut self) -> harmos::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
