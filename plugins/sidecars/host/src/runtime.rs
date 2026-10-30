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
use dokime::harmos::plugin::Declared;
use dokime::prelude::{Configuration, Refusal, Supervisor};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Workers(pub(crate) TaskTracker);

pub struct Resources {
    controls: Arc<host::controls::State>,
    workers: Arc<Workers>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Resources {
    pub async fn prepare<P>(config: Configuration, setup: &mut dokime::harmos::Setup<P>) -> Result<Self, Refusal>
    where
        P: dokime::harmos::Lifecycle + Declared,
    {
        let settings = config.settings::<crate::Settings>();
        let settings = dokime::plugin::refusal(settings)?;
        let options = HostOptions::from(settings.probes);

        let controls = Arc::new(host::controls::State::new(P::ID));
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

        Ok(Self { controls, workers })
    }

    pub fn initialize(&self, host: &Supervisor) -> Result<(), Refusal> {
        dokime::plugin::refusal(self.controls.attach(host.clone()))?;
        Ok(())
    }

    pub async fn stop(&self) {
        self.workers.0.close();
        self.workers.0.wait().await;
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
