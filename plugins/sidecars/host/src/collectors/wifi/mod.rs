/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

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
    process::Command,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

// Third Party Imports
use sysinfo::Networks;

// Local Imports
use crate::{HostOptions, HostStreamSample, HostTelemetryStream, Result, collectors::maybe_insert};
use dokime_domain::model::values::Value;

/* === Constants === */

const WIFI_CONGESTION_MODEL_24_GHZ: &str = "2.4ghz_adjacent_channel_v1";
const WIFI_CONGESTION_MODEL_SAME_CHANNEL: &str = "same_channel_v1";
const BITS_PER_MEGABIT: f64 = 1_000_000.0;
const WIFI_5_GHZ_CHANNELS: &[u32] = &[
    32, 36, 40, 44, 48, 52, 56, 60, 64, 68, 96, 100, 104, 108, 112, 116, 120, 124, 128, 132, 136, 140, 144, 149, 153, 157, 161, 165, 169,
    173, 177,
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct WifiCollector {
    sample_index: u64,
    latency_sample_index: u64,
    scan_index: u64,
    networks: Networks,
    last_network_refresh: Instant,
    congestion_cache: Arc<Mutex<WifiCongestionCache>>,
    options: HostOptions,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct WifiChannelCount {
    channel: u32,
    network_count: u32,
}

#[derive(Clone, Debug, PartialEq)]
struct WifiCongestionSummary {
    active_channel: Option<u32>,
    active_band: Option<String>,
    same_channel_network_count: u32,
    overlapping_network_count: u32,
    overlap_weighted_network_count: f64,
    score: Option<f64>,
    model: Option<&'static str>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct WifiCongestionCache {
    latest: Option<WifiCongestionSummary>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl WifiCollector {
    pub fn standalone(options: HostOptions) -> Self {
        Self::new(options, WifiCongestionCache::shared())
    }

    pub(crate) fn new(options: HostOptions, congestion_cache: Arc<Mutex<WifiCongestionCache>>) -> Self {
        Self {
            sample_index: 0,
            latency_sample_index: 0,
            scan_index: 0,
            networks: Networks::new_with_refreshed_list(),
            last_network_refresh: Instant::now(),
            congestion_cache,
            options,
        }
    }

    pub fn collect_latency_with_options(&mut self, options: &HostOptions) -> Result<HostStreamSample> {
        self.latency_sample_index += 1;
        let host = options
            .latency_host
            .as_deref()
            .or(self.options.latency_host.as_deref())
            .filter(|host| !host.trim().is_empty())
            .unwrap_or(crate::config::LATENCY_HOST)
            .trim()
            .to_owned();
        let mut values = BTreeMap::from([
            ("sample_index".to_owned(), Value::Integer(self.latency_sample_index as i64)),
            ("wifi_latency_host".to_owned(), Value::from(host.clone())),
            ("probe_status".to_owned(), Value::from("unsupported")),
        ]);

        match ping_latency_ms(&host, crate::host::wifi_latency::POLICY.attempt_timeout) {
            Ok(latency_ms) => {
                values.insert("probe_status".to_owned(), Value::from("ok"));
                values.insert("wifi_latency_ms".to_owned(), Value::from(latency_ms));
            }
            Err(error) => {
                values.insert("probe_status".to_owned(), Value::from("error"));
                values.insert("probe_error".to_owned(), Value::from(error));
            }
        }

        Ok(HostStreamSample::new(HostTelemetryStream::WifiLatency, values))
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

    fn update_congestion_cache(&self, congestion: WifiCongestionSummary) {
        self.congestion_cache.lock().expect("wifi congestion cache mutex is not poisoned").latest = Some(congestion);
    }

    fn insert_latest_congestion_fields(&self, values: &mut BTreeMap<String, Value>) {
        let latest = self.congestion_cache.lock().expect("wifi congestion cache mutex is not poisoned").latest.clone();
        let Some(congestion) = latest else {
            return;
        };
        maybe_insert(values, "active_wifi_channel", congestion.active_channel.map(|channel| Value::Integer(i64::from(channel))));
        maybe_insert(values, "active_wifi_band", congestion.active_band.map(Value::from));
        maybe_insert(values, "wifi_network_congestion_score", congestion.score.map(Value::from));
        maybe_insert(values, "congestion_model", congestion.model.map(Value::from));
    }
}

impl WifiCongestionCache {
    pub(crate) fn shared() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::default()))
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn wifi_congestion_summary(
    active_channel: Option<u32>, active_band: Option<String>, channels: &[WifiChannelCount],
) -> WifiCongestionSummary {
    let Some(active_channel) = active_channel else {
        return WifiCongestionSummary {
            active_channel: None,
            active_band,
            same_channel_network_count: 0,
            overlapping_network_count: 0,
            overlap_weighted_network_count: 0.0,
            score: None,
            model: None,
        };
    };
    let model = wifi_congestion_model(active_channel);
    let mut same_channel_network_count = 0;
    let mut overlapping_network_count = 0;
    let mut overlap_weighted_network_count = 0.0;

    for count in channels {
        if count.channel == active_channel {
            same_channel_network_count += count.network_count;
        }
        let weight = wifi_overlap_weight(active_channel, count.channel);
        if weight > 0.0 {
            overlapping_network_count += count.network_count;
            overlap_weighted_network_count += f64::from(count.network_count) * weight;
        }
    }

    WifiCongestionSummary {
        active_channel: Some(active_channel),
        active_band,
        same_channel_network_count,
        overlapping_network_count,
        overlap_weighted_network_count,
        score: Some((overlap_weighted_network_count * 10.0).min(100.0)),
        model: Some(model),
    }
}

fn insert_congestion_fields(
    values: &mut BTreeMap<String, Value>, congestion: &WifiCongestionSummary, row_channel: Option<u32>, row_network_count: u32,
) {
    maybe_insert(values, "active_wifi_channel", congestion.active_channel.map(|channel| Value::Integer(i64::from(channel))));
    maybe_insert(values, "active_wifi_band", congestion.active_band.clone().map(Value::from));
    let active_channel = congestion.active_channel == row_channel;
    values.insert("active_channel".to_owned(), Value::from(active_channel));
    let overlap_weight = congestion.active_channel.zip(row_channel).map_or(0.0, |(active, channel)| wifi_overlap_weight(active, channel));
    values.insert("overlap_weight".to_owned(), Value::from(overlap_weight));
    values.insert("same_channel_network_count".to_owned(), Value::Integer(i64::from(congestion.same_channel_network_count)));
    values.insert("overlapping_network_count".to_owned(), Value::Integer(i64::from(congestion.overlapping_network_count)));
    values.insert("overlap_weighted_network_count".to_owned(), Value::from(congestion.overlap_weighted_network_count));
    maybe_insert(values, "wifi_network_congestion_score", congestion.score.map(Value::from));
    maybe_insert(values, "congestion_model", congestion.model.map(Value::from));
    if active_channel && row_network_count == 0 {
        values.insert("max_signal_percent".to_owned(), Value::Null);
    }
}

fn dense_wifi_occupancy_channels(counts: &[WifiChannelCount], active_channel: Option<u32>) -> Vec<u32> {
    let mut channels = BTreeMap::new();
    for channel in 1..=14 {
        channels.insert(channel, ());
    }
    for channel in WIFI_5_GHZ_CHANNELS {
        channels.insert(*channel, ());
    }
    if active_channel.is_some_and(is_encoded_6ghz_channel) || counts.iter().any(|count| is_encoded_6ghz_channel(count.channel)) {
        for channel in (1..=233).step_by(4) {
            channels.insert(1_000 + channel, ());
        }
    }
    if let Some(active_channel) = active_channel {
        channels.insert(active_channel, ());
    }
    for count in counts {
        channels.insert(count.channel, ());
    }

    channels.into_keys().collect()
}

fn wifi_channel_band(channel: u32) -> Option<String> {
    match channel {
        1..=14 => Some("2.4GHz".to_owned()),
        32..=177 => Some("5GHz".to_owned()),
        _ if is_encoded_6ghz_channel(channel) => Some("6GHz".to_owned()),
        _ => None,
    }
}

fn is_encoded_6ghz_channel(channel: u32) -> bool {
    let display_channel = channel.saturating_sub(1_000);
    channel >= 1_000 && (1..=233).contains(&display_channel) && display_channel % 4 == 1
}

fn ping_latency_ms(host: &str, timeout: Duration) -> Result<f64, String> {
    let output =
        super::run_command_bounded(WifiCollector::ping_command(host), timeout).map_err(|error| format!("ping command failed: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output_text = format!("{stdout}\n{stderr}");

    if !output.status.success() {
        return Err(output_text.trim().to_owned());
    }

    parse_ping_latency_ms(&output_text).ok_or_else(|| format!("ping latency was not found in command output for {host}"))
}

fn parse_ping_latency_ms(output: &str) -> Option<f64> {
    let lower = output.to_ascii_lowercase();
    for marker in ["time=", "time<"] {
        let Some(marker_index) = lower.find(marker) else {
            continue;
        };
        let start = marker_index + marker.len();
        let number = lower[start..]
            .chars()
            .skip_while(|value| value.is_whitespace())
            .take_while(|value| value.is_ascii_digit() || *value == '.')
            .collect::<String>();
        let value = number.parse::<f64>().ok()?;
        return Some(if marker == "time<" { value.max(0.5) } else { value });
    }
    None
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

fn wifi_congestion_model(active_channel: u32) -> &'static str {
    if matches!(active_channel, 1..=14) { WIFI_CONGESTION_MODEL_24_GHZ } else { WIFI_CONGESTION_MODEL_SAME_CHANNEL }
}

/* === Tests === */

#[derive(Clone, Debug, Default, PartialEq)]
struct ActiveWifiContext {
    channel: Option<u32>,
    band: Option<String>,
    adapter: Option<String>,
}
