/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Three Wi-Fi Collectors, One Per Stream: The Link Queries The Current
// Connection, The Scan Lists Nearby Networks And The Latency Probe Echoes A
// Configured Target. None Reads Another's State.

/* === Modules === */

#[cfg(all(feature = "runtime", unix))]
mod unix;
#[cfg(all(feature = "runtime", target_os = "windows"))]
mod windows;

/* === Re-Exports === */

#[cfg(all(feature = "runtime", unix))]
pub use crate::platform::unix::wifi::{NmcliWifiRow, WifiLinkReport, parse_iw_interfaces};

#[cfg(all(feature = "runtime", target_os = "windows"))]
pub use crate::platform::windows::wifi::WifiLink;

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

// Third Party Imports
use sysinfo::Networks;

// Local Imports
use crate::platform::ScannedNetwork;
use crate::{HostOptions, HostStreamSample, HostTelemetryStream, Result, collectors::maybe_insert};
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::Value;

/* === Constants === */

const WIFI_CONGESTION_MODEL_24_GHZ: &str = "2.4ghz_adjacent_channel_v1";
const WIFI_CONGESTION_MODEL_SAME_CHANNEL: &str = "same_channel_v1";
const BITS_PER_MEGABIT: f64 = 1_000_000.0;
const LOST_PERCENT: f64 = 100.0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct WifiLinkCollector {
    sample_index: u64,
    networks: Networks,
    last_network_refresh: Instant,
    options: HostOptions,
}

pub struct WifiScanCollector {
    sample_index: u64,
    last_active_scan: Option<Instant>,
    options: HostOptions,
}

pub struct WifiLatencyCollector {
    sample_index: u64,
    options: HostOptions,
}

// What A Platform Scan Returned: Fresh From An Active Scan, Or The Cached List.
pub(crate) struct ScanReport {
    pub(crate) networks: Vec<ScannedNetwork>,
    pub(crate) active: bool,
    pub(crate) error: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct Congestion {
    score: Option<f64>,
    model: Option<&'static str>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl WifiLinkCollector {
    pub fn new(options: HostOptions) -> Self {
        Self {
            sample_index: 0,
            networks: Networks::new_with_refreshed_list(),
            last_network_refresh: Instant::now(),
            options,
        }
    }

    pub fn collect(&mut self) -> Result<HostStreamSample> {
        self.sample_index += 1;
        let mut values = BTreeMap::from([
            ("sample_index".to_owned(), Value::Integer(self.sample_index as i64)),
            ("probe_status".to_owned(), Value::from("unsupported")),
        ]);
        let adapter = self.read_link(&mut values);
        if let Some((received, sent)) = self.adapter_throughput(adapter.as_deref()) {
            values.insert("wifi_rx_mbps".to_owned(), Value::from(received * 8.0 / BITS_PER_MEGABIT));
            values.insert("wifi_tx_mbps".to_owned(), Value::from(sent * 8.0 / BITS_PER_MEGABIT));
        }
        Ok(HostStreamSample::new(HostTelemetryStream::WifiLink, values))
    }

    fn adapter_throughput(&mut self, adapter: Option<&str>) -> Option<(f64, f64)> {
        let now = Instant::now();
        let elapsed_s = now.duration_since(self.last_network_refresh).as_secs_f64().max(0.001);
        self.networks.refresh(true);
        self.last_network_refresh = now;

        let adapter = adapter?;
        let exact = self
            .networks
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(adapter))
            .map(|(_, network)| (network.received(), network.transmitted()));
        if let Some((received, transmitted)) = exact {
            return Some((received as f64 / elapsed_s, transmitted as f64 / elapsed_s));
        }

        let adapter = adapter.to_lowercase();
        let mut received = 0;
        let mut transmitted = 0;
        let mut matched = false;
        for (name, network) in &self.networks {
            let name = name.to_lowercase();
            if name.contains(&adapter) || adapter.contains(&name) {
                matched = true;
                received += network.received();
                transmitted += network.transmitted();
            }
        }

        matched.then_some((received as f64 / elapsed_s, transmitted as f64 / elapsed_s))
    }
}

impl WifiScanCollector {
    pub fn new(options: HostOptions) -> Self {
        Self {
            sample_index: 0,
            last_active_scan: None,
            options,
        }
    }

    // One Row Per Observed Network; An Empty Or Failed Scan Is One Status Row.
    pub fn collect(&mut self) -> Result<Vec<HostStreamSample>> {
        self.sample_index += 1;
        let now = Instant::now();
        let minimum = self.options.minimum_scan_interval;
        let active = self.last_active_scan.is_none_or(|last| now.duration_since(last) >= minimum);
        let report = Self::scan(&self.options, active);
        let timestamp = Timestamp::from_system_time(std::time::SystemTime::now());
        let report = match report {
            Ok(report) => report,
            Err(error) => {
                let values = self.status_row("error", Some(error));
                return Ok(vec![HostStreamSample::with_timestamp(HostTelemetryStream::WifiScan, timestamp, values)]);
            }
        };
        if report.active {
            self.last_active_scan = Some(now);
        }
        let status = if report.active { "ok" } else { "cached" };
        if report.networks.is_empty() {
            let status = if report.error.is_some() { "error" } else { "unavailable" };
            let values = self.status_row(status, report.error);
            return Ok(vec![HostStreamSample::with_timestamp(HostTelemetryStream::WifiScan, timestamp, values)]);
        }

        let mut per_channel = BTreeMap::<u32, u32>::new();
        for channel in report.networks.iter().filter_map(|network| network.channel) {
            *per_channel.entry(channel).or_default() += 1;
        }
        let associated = report.networks.iter().find(|network| network.associated);
        let active_channel = associated.and_then(|network| network.channel);
        let active_ssid = associated.and_then(|network| network.ssid.clone());
        let congestion = congestion(active_channel, &per_channel);
        let total = report.networks.len();
        let samples = report
            .networks
            .into_iter()
            .map(|network| {
                let mut values = BTreeMap::from([
                    ("sample_index".to_owned(), Value::Integer(self.sample_index as i64)),
                    ("scan_status".to_owned(), Value::from(status)),
                    ("associated".to_owned(), Value::from(network.associated)),
                    ("scan_network_count".to_owned(), Value::Integer(total as i64)),
                ]);
                let channel_count = network.channel.and_then(|channel| per_channel.get(&channel)).copied().unwrap_or_default();
                values.insert("network_count".to_owned(), Value::Integer(i64::from(channel_count)));
                maybe_insert(&mut values, "scan_error", report.error.clone().map(Value::from));
                maybe_insert(&mut values, "wifi_adapter", network.adapter.map(Value::from));
                maybe_insert(&mut values, "wifi_ssid", network.ssid.map(Value::from));
                maybe_insert(&mut values, "wifi_bssid", network.bssid.map(Value::from));
                maybe_insert(&mut values, "wifi_signal_percent", network.signal_percent.map(Value::from));
                maybe_insert(&mut values, "wifi_rssi_dbm", network.rssi_dbm.map(Value::from));
                maybe_insert(&mut values, "wifi_channel", network.channel.map(|channel| Value::Integer(i64::from(channel))));
                maybe_insert(&mut values, "wifi_band", network.channel.and_then(wifi_channel_band).map(Value::from));
                maybe_insert(&mut values, "wifi_security", network.security.map(Value::from));
                maybe_insert(&mut values, "active_wifi_channel", active_channel.map(|channel| Value::Integer(i64::from(channel))));
                maybe_insert(&mut values, "active_wifi_band", active_channel.and_then(wifi_channel_band).map(Value::from));
                maybe_insert(&mut values, "active_wifi_ssid", active_ssid.clone().map(Value::from));
                maybe_insert(&mut values, "wifi_network_congestion_score", congestion.score.map(Value::from));
                maybe_insert(&mut values, "congestion_model", congestion.model.map(Value::from));
                HostStreamSample::with_timestamp(HostTelemetryStream::WifiScan, timestamp.clone(), values)
            })
            .collect();
        Ok(samples)
    }

    fn status_row(&self, status: &str, error: Option<String>) -> BTreeMap<String, Value> {
        let mut values = BTreeMap::from([
            ("sample_index".to_owned(), Value::Integer(self.sample_index as i64)),
            ("scan_status".to_owned(), Value::from(status)),
            ("scan_network_count".to_owned(), Value::Integer(0)),
        ]);
        maybe_insert(&mut values, "scan_error", error.map(Value::from));
        values
    }
}

impl WifiLatencyCollector {
    pub fn new(options: HostOptions) -> Self {
        Self { sample_index: 0, options }
    }

    // A Missing Reply Is Loss; Only A Probe That Cannot Run Is An Error.
    pub fn collect(&mut self) -> Result<HostStreamSample> {
        self.sample_index += 1;
        let host =
            self.options.latency_host.as_deref().map(str::trim).filter(|host| !host.is_empty()).unwrap_or(crate::config::LATENCY_HOST);
        let mut values = BTreeMap::from([
            ("sample_index".to_owned(), Value::Integer(self.sample_index as i64)),
            ("wifi_latency_host".to_owned(), Value::from(host)),
        ]);
        match Self::round_trip_ms(host, echo_timeout()) {
            Ok(Some(latency_ms)) => {
                values.insert("probe_status".to_owned(), Value::from("ok"));
                values.insert("wifi_latency_ms".to_owned(), Value::from(latency_ms));
                values.insert("wifi_packet_loss_percent".to_owned(), Value::from(0.0));
            }
            Ok(None) => {
                values.insert("probe_status".to_owned(), Value::from("ok"));
                values.insert("wifi_packet_loss_percent".to_owned(), Value::from(LOST_PERCENT));
            }
            Err(error) => {
                values.insert("probe_status".to_owned(), Value::from("error"));
                values.insert("probe_error".to_owned(), Value::from(error));
            }
        }
        Ok(HostStreamSample::new(HostTelemetryStream::WifiLatency, values))
    }
}

/* === Public Functions === */

pub(crate) fn wifi_channel_band(channel: u32) -> Option<String> {
    match channel {
        1..=14 => Some("2.4GHz".to_owned()),
        32..=177 => Some("5GHz".to_owned()),
        _ if is_encoded_6ghz_channel(channel) => Some("6GHz".to_owned()),
        _ => None,
    }
}

/* === Private Functions === */

// An Echo Waits Half Its Collection's Bound, So A Lost Reply Yields A Loss Sample Rather Than A Timed-Out Collection.
fn echo_timeout() -> Duration {
    crate::host::wifi_latency::POLICY.attempt_timeout / 2
}

// Networks On The Associated Channel Plus Overlapping 2.4 GHz Neighbours, Weighted By Overlap, Scaled To 0-100.
fn congestion(active_channel: Option<u32>, per_channel: &BTreeMap<u32, u32>) -> Congestion {
    let Some(active_channel) = active_channel else {
        return Congestion { score: None, model: None };
    };
    let weighted =
        per_channel.iter().map(|(channel, count)| f64::from(*count) * wifi_overlap_weight(active_channel, *channel)).sum::<f64>();
    let model = if matches!(active_channel, 1..=14) { WIFI_CONGESTION_MODEL_24_GHZ } else { WIFI_CONGESTION_MODEL_SAME_CHANNEL };
    Congestion {
        score: Some((weighted * 10.0).min(100.0)),
        model: Some(model),
    }
}

fn is_encoded_6ghz_channel(channel: u32) -> bool {
    let display_channel = channel.saturating_sub(1_000);
    channel >= 1_000 && (1..=233).contains(&display_channel) && display_channel % 4 == 1
}

fn wifi_overlap_weight(active_channel: u32, scanned_channel: u32) -> f64 {
    if matches!(active_channel, 1..=14) {
        let distance = active_channel.abs_diff(scanned_channel) as f64;
        (1.0 - distance / 5.0).max(0.0)
    } else if active_channel == scanned_channel {
        1.0
    } else {
        0.0
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn congestion_weighs_overlapping_channels() {
        let per_channel = BTreeMap::from([(6, 2), (8, 1), (11, 4), (36, 3)]);
        let congestion = congestion(Some(6), &per_channel);
        assert_eq!(congestion.model, Some(WIFI_CONGESTION_MODEL_24_GHZ));
        // Channel 6 Counts Fully, Channel 8 At 0.6, Channel 11 Not At All.
        assert!((congestion.score.unwrap() - 26.0).abs() < 1e-9);
        assert_eq!(super::congestion(None, &per_channel).score, None);
    }

    #[test]
    fn six_gigahertz_channels_are_encoded() {
        assert_eq!(wifi_channel_band(1_005).as_deref(), Some("6GHz"));
        assert_eq!(wifi_channel_band(1_006), None);
        assert_eq!(wifi_channel_band(36).as_deref(), Some("5GHz"));
    }
}
