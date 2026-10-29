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
use std::sync::{Arc, Mutex};

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::{
    FixtureHostTelemetrySource, HOST_FIXTURE_SOURCE, HostOptions, HostStreamSample, HostTelemetryConfig, HostTelemetrySourceKind,
    HostTelemetryStream,
    collectors::{
        bluetooth::BluetoothCollector,
        network::NetworkCollector,
        system::SystemCollector,
        wifi::{WifiCollector, WifiCongestionCache},
    },
    sample::default_source,
};
use dokime_domain::model::telemetry::TelemetryBatch;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct Host {
    source: HostTelemetrySourceKind,
    fixture: Arc<Mutex<FixtureHostTelemetrySource>>,
    system: Arc<Mutex<SystemCollector>>,
    network: Arc<Mutex<NetworkCollector>>,
    wifi_link: Arc<Mutex<WifiCollector>>,
    wifi_latency: Arc<Mutex<WifiCollector>>,
    wifi_occupancy: Arc<Mutex<WifiCollector>>,
    bluetooth: Arc<Mutex<BluetoothCollector>>,
    options: HostOptions,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HostSnapshot {
    pub system: Option<HostStreamSample>,
    pub network: Option<HostStreamSample>,
    pub wifi: Option<HostStreamSample>,
    pub wifi_latency: Option<HostStreamSample>,
    pub wifi_channel_occupancy: Vec<HostStreamSample>,
    pub bluetooth: Option<HostStreamSample>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Host {
    pub fn new(config: HostTelemetryConfig) -> Self {
        let options = config.options.clone();
        let wifi_congestion_cache = WifiCongestionCache::shared();

        Self {
            source: config.source,
            fixture: Arc::new(Mutex::new(FixtureHostTelemetrySource::new())),
            system: Arc::new(Mutex::new(SystemCollector::new())),
            network: Arc::new(Mutex::new(NetworkCollector::new())),
            wifi_link: Arc::new(Mutex::new(WifiCollector::new(options.clone(), wifi_congestion_cache.clone()))),
            wifi_latency: Arc::new(Mutex::new(WifiCollector::new(options.clone(), wifi_congestion_cache.clone()))),
            wifi_occupancy: Arc::new(Mutex::new(WifiCollector::new(options.clone(), wifi_congestion_cache))),
            bluetooth: Arc::new(Mutex::new(BluetoothCollector::new(options.clone()))),
            options,
        }
    }

    pub fn fixture() -> Self {
        Self::new(HostTelemetryConfig {
            source: HostTelemetrySourceKind::Fixture,
            options: HostOptions::default(),
        })
    }

    pub fn collect_stream(&self, stream: HostTelemetryStream) -> crate::Result<Option<TelemetryBatch>> {
        self.collect_stream_with_options(stream, &self.options)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn collect_stream_with_options(&self, stream: HostTelemetryStream, options: &HostOptions) -> crate::Result<Option<TelemetryBatch>> {
        let samples = self.collect_stream_samples_with_options(stream, options)?;
        if samples.is_empty() {
            return Ok(None);
        }

        let source = self.source_id();
        let stream_id = stream.identifier();
        let telemetry_samples = samples
            .into_iter()
            .map(|sample| {
                let timestamp = sample.timestamp.clone();
                let mut telemetry_sample =
                    dokime_domain::model::telemetry::TelemetrySample::new(crate::new_id(), stream_id.clone(), 0, sample.timestamp, source);
                telemetry_sample.values = sample.values;
                // A row that already carries its own "timestamp" keeps it;
                // one that does not is stamped with the instant the host
                // measured it, agreeing with `telemetry_sample.timestamp`.
                telemetry_sample.values.entry("timestamp".to_owned()).or_insert_with(|| Value::from(timestamp));
                telemetry_sample
            })
            .collect();

        Ok(Some(TelemetryBatch::new(crate::new_id(), stream_id, telemetry_samples)))
    }

    fn collect_stream_samples(&self, stream: HostTelemetryStream) -> crate::Result<Vec<HostStreamSample>> {
        self.collect_stream_samples_with_options(stream, &self.options)
    }

    fn collect_stream_samples_with_options(
        &self, stream: HostTelemetryStream, options: &HostOptions,
    ) -> crate::Result<Vec<HostStreamSample>> {
        if self.source == HostTelemetrySourceKind::Fixture {
            return Ok(self.fixture.lock().expect("fixture host telemetry mutex is not poisoned").collect(stream));
        }

        match stream {
            HostTelemetryStream::System => Ok(vec![self.system.lock().expect("system collector mutex is not poisoned").collect()?]),
            HostTelemetryStream::Network => Ok(vec![self.network.lock().expect("network collector mutex is not poisoned").collect()?]),
            HostTelemetryStream::Wifi => {
                Ok(vec![self.wifi_link.lock().expect("wifi link collector mutex is not poisoned").collect_link_with_options(options)?])
            }
            HostTelemetryStream::WifiLatency => Ok(vec![
                self.wifi_latency.lock().expect("wifi latency collector mutex is not poisoned").collect_latency_with_options(options)?,
            ]),
            HostTelemetryStream::WifiChannelOccupancy => self
                .wifi_occupancy
                .lock()
                .expect("wifi occupancy collector mutex is not poisoned")
                .collect_channel_occupancy_with_options(options),
            HostTelemetryStream::Bluetooth => {
                Ok(vec![self.bluetooth.lock().expect("bluetooth collector mutex is not poisoned").collect()?])
            }
        }
    }

    pub fn snapshot(&self) -> HostSnapshot {
        HostSnapshot {
            system: self.collect_stream_samples(HostTelemetryStream::System).ok().and_then(|mut samples| samples.pop()),
            network: self.collect_stream_samples(HostTelemetryStream::Network).ok().and_then(|mut samples| samples.pop()),
            wifi: self.collect_stream_samples(HostTelemetryStream::Wifi).ok().and_then(|mut samples| samples.pop()),
            wifi_latency: self.collect_stream_samples(HostTelemetryStream::WifiLatency).ok().and_then(|mut samples| samples.pop()),
            wifi_channel_occupancy: self.collect_stream_samples(HostTelemetryStream::WifiChannelOccupancy).unwrap_or_default(),
            bluetooth: self.collect_stream_samples(HostTelemetryStream::Bluetooth).ok().and_then(|mut samples| samples.pop()),
        }
    }

    fn source_id(&self) -> &'static str {
        match self.source {
            HostTelemetrySourceKind::Native => default_source(),
            HostTelemetrySourceKind::Fixture => HOST_FIXTURE_SOURCE,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
