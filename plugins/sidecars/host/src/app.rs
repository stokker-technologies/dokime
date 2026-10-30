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
    controls: Arc<host::controls::State>,
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
    id = "plugin_host",
    description = "Native host telemetry collected through the supervised plugin boundary.",
    restart = "never",
    requires(dokime::harmos::builtins::streams::Publish, dokime::operations::effects::ReadEnvironment, dokime::operations::effects::PrepareEffect, dokime::operations::effects::ConfirmEffect),
    contributions(dokime::RunStart<host::adapters::CaptureAdapters>, dokime::Evidence<host::Catalog>, dokime::Settings<crate::Settings>),
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
        let settings = config.settings::<crate::Settings>();
        let settings = dokime::plugin::refusal(settings)?;
        let options = HostOptions::from(settings.probes);

        let controls = Arc::new(host::controls::State::new());
        setup.resource::<host::controls::State>(controls.clone());

        let tracker = TaskTracker::new();
        let workers = Arc::new(Workers(tracker));
        setup.resource::<Workers>(workers.clone());

        // Calibrate Once At Plugin Start, Away From The Async Runtime Thread.
        let task = workers.0.spawn_blocking(|| {
            let mut collector = collectors::system::SystemCollector::new();
            collector.calibrate_cpu(host::system::POLICY.attempt_timeout / 2)?;
            Ok::<_, crate::Error>(collector)
        });
        let collector = dokime::plugin::refusal(dokime::plugin::refusal(task.await)?)?;
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

        let observers = ObserverAttachment::from_catalog::<host::observers::Observers>(<Self as Declared>::STREAMS)?;
        Ok(Self {
            controls,
            workers,
            observers: Some(observers),
        })
    }

    async fn initialize(&mut self, host: &Supervisor) -> Result<(), Refusal> {
        dokime::plugin::refusal(self.controls.attach(host.clone()))?;
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
