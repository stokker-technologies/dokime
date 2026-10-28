/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Dokime Application Lifecycle

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports

// Local Imports
use super::{Dokime, LaunchConfig, Resources};
use crate::runtime::{Error, Result};
use dokime_domain::state::records::Records;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl harmos::Lifecycle for Dokime {
    type Config = Option<LaunchConfig>;
    type Error = Error;
    type Context = harmos::Startup<Self>;

    async fn prepare(config: Self::Config, setup: &mut harmos::Setup<Self>) -> Result<Self> {
        let mut owner = Self {
            parts: None,
            prepared: None,
            live: Default::default(),
            composition: Vec::new(),
        };
        let resources = if let Some(config) = config {
            owner.live = config.live.clone();
            config.boot.lock().expect("startup owners are healthy").cleanup = Some(setup.settings().startup_cleanup());
            let parts = crate::RuntimeHandle::prepare(config).await?;
            owner.composition = parts.config.composition.clone();
            crate::runtime::storage::configure(setup, &parts);
            owner.prepared = Some(crate::runtime::handle::Prepared::configure(setup, &parts, &owner.live)?);
            let resources = parts.resources.clone();
            owner.parts = Some(parts);
            resources
        } else {
            Resources::new(Default::default())
        };
        setup.resource::<Resources>(Arc::new(resources.clone()));
        setup.resource::<tokio::sync::RwLock<Option<crate::RuntimeHandle>>>(owner.live.clone());
        setup.settings().schema_settings(resources.schema_settings);
        let preparation_bytes = std::num::NonZeroUsize::new(8 * 1024 * 1024);
        setup.settings().operation_settings(harmos::operations::Settings { preparation_bytes, ..Default::default() });
        setup.settings().register(crate::transactions::Transactions::catalog()).register(Records::catalog());
        for composition in &owner.composition {
            (composition.prepare)(setup, &owner.live, &owner.parts.as_ref().expect("native composition has bootstrap settings").config)?;
        }
        Ok(owner)
    }

    async fn initialize(&mut self, context: &Self::Context) -> Result<()> {
        let Some(parts) = self.parts.take() else { return Ok(()) };
        let prepared = self.prepared.take().expect("native preparation retains its resources");
        let handle = crate::RuntimeHandle::initialize(&context.runtime, parts, prepared).await?;
        *self.live.write().await = Some(handle.clone());
        for composition in &self.composition {
            (composition.initialize)(handle.clone()).await?;
        }
        Ok(())
    }

    async fn stop(&mut self, _: &Self::Context) -> Result<()> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<()> {
        for composition in self.composition.iter().rev() {
            (composition.cleanup)().await?;
        }
        self.live.write().await.take();
        self.parts.take();
        self.prepared.take();
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
