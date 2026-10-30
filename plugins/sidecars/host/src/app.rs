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
use std::sync::Arc;

// Third Party Imports

// Local Imports
use dokime::harmos::operations::OperationPolicy;
use dokime::harmos::plugin::{Attachment, Declared};
use dokime::harmos::sidecar::{ObserverAttachment, Publisher};
use dokime::prelude::{Configuration, Refusal, Supervisor};
use plugin_host::{Resources, Result, host};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Observing = ObserverAttachment<Publisher<HostPlugin>>;

/* === Structs === */

pub struct HostPlugin {
    resources: Resources,
    observers: Option<Arc<Observing>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl dokime::harmos::builtins::lifecycle::Policy for HostPlugin {
    const INITIALIZATION_POLICY: OperationPolicy = host::wifi_scan::POLICY;
    const SHUTDOWN_POLICY: OperationPolicy = host::wifi_scan::POLICY;
}

#[dokime::plugin(
    id = "plugin_host",
    description = "Native host telemetry collected through the supervised plugin boundary.",
    restart = "never",
    requires(dokime::harmos::builtins::streams::Publish, dokime::operations::effects::ReadEnvironment, dokime::operations::effects::PrepareEffect, dokime::operations::effects::ConfirmEffect),
    contributions(dokime::RunStart<host::adapters::CaptureAdapters>, dokime::Evidence<host::Evidence>, dokime::Settings<plugin_host::Settings>),
    observers = host::observers::Observers
)]
impl dokime::Plugin for HostPlugin {
    const CATALOGS: &'static [dokime::harmos::Catalog] = &[host::Catalog::CATALOG, host::operations::Operations::CATALOG];

    fn attachment(&self) -> Option<Arc<dyn Attachment<Supervisor>>> {
        let observers = self.observers.as_ref()?.clone();
        Some(observers)
    }
}

impl dokime::harmos::Lifecycle for HostPlugin {
    type Config = Configuration;
    type Error = Refusal;
    type Context = Supervisor;

    async fn prepare(config: Configuration, setup: &mut dokime::harmos::Setup<Self>) -> Result<Self, Refusal> {
        let resources = Resources::prepare::<Self>(config, setup).await?;
        let observers = ObserverAttachment::from_catalog::<host::observers::Observers>(<Self as Declared>::STREAMS)?;
        Ok(Self { resources, observers: Some(observers) })
    }

    async fn initialize(&mut self, host: &Supervisor) -> Result<(), Refusal> {
        self.resources.initialize(host)?;
        Ok(())
    }

    async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        self.resources.stop().await;
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        self.observers.take();
        self.resources.stop().await;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
