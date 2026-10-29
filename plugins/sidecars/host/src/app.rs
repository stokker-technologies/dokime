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
use tokio::sync::Mutex;
use tokio_util::task::TaskTracker;

// Local Imports
use crate::{HostOptions, Result, collectors, host};
use dokime::harmos::operations::OperationPolicy;
use dokime::harmos::plugin::{Attachment, Declared};
use dokime::harmos::sidecar::{ObserverAttachment, Publisher};
use dokime::prelude::{Configuration, Refusal, Supervisor};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Observing = ObserverAttachment<Publisher<HostPlugin>>;

/* === Structs === */

pub struct Workers(pub(crate) TaskTracker);

pub struct HostPlugin {
    workers: Arc<Workers>,
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
    id = "host_telemetry",
    description = "Native host telemetry collected through the supervised plugin boundary.",
    restart = "never",
    requires(dokime::harmos::builtins::streams::Publish),
    contributions(dokime::Evidence<host::Catalog>, dokime::Settings<crate::Settings>),
    observers = host::Catalog
)]
impl dokime::Plugin for HostPlugin {
    const CATALOGS: &'static [dokime::harmos::Catalog] = &[host::Catalog::CATALOG];

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
        let settings = config.settings::<crate::Settings>();
        let settings = dokime::plugin::refusal(settings)?;
        let options = HostOptions::from(settings.probes);

        let tracker = TaskTracker::new();
        let workers = Arc::new(Workers(tracker));
        setup.resource::<Workers>(workers.clone());

        let collector = collectors::system::SystemCollector::new();
        let state = host::system::State { collector };
        let state = Arc::new(Mutex::new(state));
        setup.resource::<Mutex<host::system::State>>(state);

        let collector = collectors::network::NetworkCollector::new();
        let state = host::network::State { collector };
        let state = Arc::new(Mutex::new(state));
        setup.resource::<Mutex<host::network::State>>(state);

        // Each Wi-Fi Stream Owns Its Collector, So No Observer Reads Another's State.
        let collector = collectors::wifi::WifiLinkCollector::new(options.clone());
        let state = host::wifi_link::State { collector };
        let state = Arc::new(Mutex::new(state));
        setup.resource::<Mutex<host::wifi_link::State>>(state);

        let collector = collectors::wifi::WifiScanCollector::new(options.clone());
        let state = host::wifi_scan::State { collector };
        let state = Arc::new(Mutex::new(state));
        setup.resource::<Mutex<host::wifi_scan::State>>(state);

        let collector = collectors::wifi::WifiLatencyCollector::new(options.clone());
        let state = host::wifi_latency::State { collector };
        let state = Arc::new(Mutex::new(state));
        setup.resource::<Mutex<host::wifi_latency::State>>(state);

        let collector = collectors::bluetooth::BluetoothCollector::new(options);
        let state = host::bluetooth::State { collector };
        let state = Arc::new(Mutex::new(state));
        setup.resource::<Mutex<host::bluetooth::State>>(state);

        let observers = ObserverAttachment::from_catalog::<host::Catalog>(<Self as Declared>::STREAMS)?;
        Ok(Self { workers, observers: Some(observers) })
    }

    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        self.workers.0.close();
        self.workers.0.wait().await;
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        self.observers.take();
        self.workers.0.close();
        self.workers.0.wait().await;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
