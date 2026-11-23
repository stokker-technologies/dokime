/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Wi-Fi Collector State Shared by Its Observers: the Link Queries the Current Connection, the Scan
// Lists Nearby Networks and the Latency Probe Echoes a Configured Target; None Reads Another's State.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::result;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

// Third Party Imports
use sysinfo::Networks;
use tokio::task;

// Local Imports
use crate::domain::readings::{WifiLatency, WifiLink, WifiScan};
use crate::domain::telemetry::WifiChannels;
use crate::platform::{ScannedNetwork, WifiBand, WifiCapabilities, wifi};
use dokime::{Error, Result};

/* === Constants === */

pub const CONGESTION_MODEL_24_GHZ: &str = "2.4ghz_adjacent_channel_v1";
pub const CONGESTION_MODEL_SAME_CHANNEL: &str = "same_channel_v1";
const BITS_PER_MEGABIT: f64 = 1_000_000.0;
const BITS_PER_BYTE: f64 = 8.0;
const LOST_PERCENT: f64 = 100.0;
// 2.4 GHz Channel Spacing Is 5 MHz; Overlap Falls to Zero Five Channels Away.
const OVERLAP_CHANNEL_SPAN: f64 = 5.0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Wi-Fi Collection State, Released at Each Collection's Stop.
#[derive(Default)]
pub struct Collector {
    link: Arc<Mutex<Link>>,
    scan: Arc<Mutex<Scan>>,
    latency: Arc<Mutex<u64>>,
}

/// What the Link Collection Reads From the Plugin Settings.
pub struct LinkOptions {
    pub adapter: Option<String>,
    pub probe_timeout: Duration,
    pub minimum_rate_window: Duration,
}

/// What the Scan Collection Reads From the Plugin Settings.
pub struct ScanOptions {
    pub adapter: Option<String>,
    pub probe_timeout: Duration,
    pub window: Duration,
    pub minimum_interval: Duration,
    pub congestion_weight: f64,
}

/// What the Latency Collection Reads From the Plugin Settings.
pub struct LatencyOptions {
    pub host: String,
    pub reply_timeout: Duration,
    pub process_timeout: Duration,
    pub payload_bytes: u32,
}

/// What a Platform Scan Returned: Fresh From an Active Scan, or the Cached List.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScanReport {
    pub networks: Vec<ScannedNetwork>,
    pub capabilities: Option<WifiCapabilities>,
    pub active: bool,
    pub error: Option<String>,
}

/// Both Projections of One Scan, Published by the Same Collection Tick.
pub struct ScanReadings {
    pub networks: Vec<WifiScan>,
    pub channels: Vec<WifiChannels>,
}

/// The Channel Plan Remembers Observed 6 GHz Support Until Collection Stop.
#[derive(Default)]
pub struct ChannelPlan {
    saw_6ghz: bool,
}

/// The Congestion Score of the Associated Channel and the Model That Weighed It.
#[derive(Clone, Debug, PartialEq)]
pub struct Congestion {
    pub score: Option<f64>,
    pub model: Option<&'static str>,
}

/// One Adapter's Byte Counters Since the Previous Refresh.
pub struct Counters {
    pub name: String,
    pub received: u64,
    pub transmitted: u64,
}

#[derive(Default)]
struct Link {
    sample_index: u64,
    networks: Option<(Networks, Instant)>,
}

#[derive(Default)]
struct Scan {
    sample_index: u64,
    last_active_scan: Option<Instant>,
    channel_plan: ChannelPlan,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Collector {
    /// Acquire Adapter Counters When Link Collection Demand Starts.
    pub fn start_link(&self) -> Result<()> {
        let networks = Networks::new_with_refreshed_list()
            .map_err(|error| Error::Invalid(format!("Start Wi-Fi Link Collection: {error}; Restart the Plugin")))?;
        self.link.lock().map_err(|_| poisoned())?.networks = Some((networks, Instant::now()));
        Ok(())
    }

    /// One Link Row: the Current Connection Plus the Adapter's Throughput Since the Previous Row.
    pub async fn link(&self, options: LinkOptions) -> Result<WifiLink> {
        let state = Arc::clone(&self.link);
        blocking(move || {
            let mut state = state.lock().map_err(|_| poisoned())?;
            state.sample_index += 1;
            let mut row = WifiLink {
                sample_index: Some(index(state.sample_index)),
                probe_status: Some("unsupported".into()),
                ..WifiLink::default()
            };
            let adapter = wifi::read_link(&options, &mut row);
            let now = Instant::now();
            let (networks, last) =
                state.networks.as_mut().ok_or_else(|| Error::Invalid("Collect Wi-Fi Link: Start the Collection Before Reading".into()))?;
            let elapsed_s = now.duration_since(*last).max(options.minimum_rate_window).as_secs_f64();
            networks.refresh(true);
            *last = now;
            let counters = networks
                .iter()
                .map(|(name, network)| Counters {
                    name: name.to_string_lossy().into_owned(),
                    received: network.received(),
                    transmitted: network.transmitted(),
                })
                .collect::<Vec<_>>();
            if let Some((received, sent)) = adapter.as_deref().and_then(|adapter| throughput(&counters, adapter, elapsed_s)) {
                row.wifi_rx_mbps = Some(received * BITS_PER_BYTE / BITS_PER_MEGABIT);
                row.wifi_tx_mbps = Some(sent * BITS_PER_BYTE / BITS_PER_MEGABIT);
            }
            Ok(row)
        })
        .await
    }

    /// One Row per Observed Network; an Empty or Failed Scan Is One Status Row.
    pub async fn scan(&self, options: ScanOptions) -> Result<ScanReadings> {
        let state = Arc::clone(&self.scan);
        blocking(move || {
            let mut state = state.lock().map_err(|_| poisoned())?;
            state.sample_index += 1;
            let now = Instant::now();
            let active = scan_due(state.last_active_scan, now, options.minimum_interval);
            let report = wifi::scan(&options, active);
            if report.as_ref().is_ok_and(|report| report.active) {
                state.last_active_scan = Some(now);
            }
            let sample_index = state.sample_index;
            Ok(scan_readings(sample_index, report, options.congestion_weight, &mut state.channel_plan))
        })
        .await
    }

    /// One Latency Row; a Missing Reply Is Loss, Only a Probe That Cannot Run Is an Error.
    pub async fn latency(&self, options: LatencyOptions) -> Result<WifiLatency> {
        let state = Arc::clone(&self.latency);
        blocking(move || {
            let mut sample_index = state.lock().map_err(|_| poisoned())?;
            *sample_index += 1;
            Ok(latency_row(*sample_index, &options.host, wifi::round_trip_ms(&options)))
        })
        .await
    }
}

impl Collector {
    /// Drop the Adapter Counters When the Link Collection's Last Listener Leaves.
    pub fn stop_link(&self) {
        *self.link.lock().unwrap_or_else(PoisonError::into_inner) = Link::default();
    }

    /// Forget the Last Active Scan When the Scan Collection's Last Listener Leaves.
    pub fn stop_scan(&self) {
        *self.scan.lock().unwrap_or_else(PoisonError::into_inner) = Scan::default();
    }
}

impl ChannelPlan {
    /// Every Channel Once per Successful Scan, With Primary Counts and Weighted Occupancy.
    pub fn rows(&mut self, sample_index: u64, report: &ScanReport) -> Vec<WifiChannels> {
        self.saw_6ghz |= report.networks.iter().any(|network| network.channel.is_some_and(is_encoded_6ghz_channel));
        let fallback = [WifiBand::Ghz24, WifiBand::Ghz5, WifiBand::Ghz6];
        let fallback = if self.saw_6ghz { &fallback[..] } else { &fallback[..2] };
        let bands = report.capabilities.as_ref().map(|capabilities| capabilities.bands.as_slice()).unwrap_or(fallback);
        let channel_14_allowed = report.capabilities.as_ref().is_some_and(|capabilities| capabilities.channel_14_allowed);
        let mut rows = Vec::new();
        for frequency_band in [WifiBand::Ghz24, WifiBand::Ghz5, WifiBand::Ghz6] {
            if !bands.contains(&frequency_band) {
                continue;
            }
            let channels: Vec<u32> = match frequency_band {
                WifiBand::Ghz24 => (1..=if channel_14_allowed { 14 } else { 13 }).collect(),
                WifiBand::Ghz5 => (36..=64).step_by(4).chain((100..=144).step_by(4)).chain((149..=177).step_by(4)).collect(),
                WifiBand::Ghz6 => (1_001..=1_233).step_by(4).collect(),
            };
            for channel in channels {
                let occupancy = report.networks.iter().filter(|network| occupies(channel, network)).count();
                rows.push(WifiChannels {
                    sample_index: index(sample_index),
                    wifi_band: band(channel).expect("Channel Plan Contains Known Bands"),
                    wifi_channel: i64::from(if frequency_band == WifiBand::Ghz6 { channel - 1_000 } else { channel }),
                    occupancy: i64::try_from(occupancy).unwrap_or(i64::MAX),
                });
            }
        }
        rows
    }
}

/* === Public Functions === */

/// The Band of a Channel; 6 GHz Channels Are Encoded as 1000 Plus Their Display Channel.
pub fn band(channel: u32) -> Option<String> {
    match channel {
        1..=14 => Some("2.4GHz".to_owned()),
        32..=177 => Some("5GHz".to_owned()),
        _ if is_encoded_6ghz_channel(channel) => Some("6GHz".to_owned()),
        _ => None,
    }
}

/// Convert a Platform Frequency in MHz or kHz to the Shared Encoded Channel.
pub fn channel_from_frequency(value: u32) -> Option<u32> {
    let mhz = if value > 10_000 { value / 1_000 } else { value };
    match mhz {
        2_412..=2_472 => Some((mhz - 2_407) / 5),
        2_484 => Some(14),
        5_005..=5_895 => Some((mhz - 5_000) / 5),
        5_955..=7_115 => Some(1_000 + (mhz - 5_950) / 5),
        _ => None,
    }
}

/// Whether an Active Scan Is Due; Collections Between Active Scans Read the Driver's Cache.
pub fn scan_due(last_active_scan: Option<Instant>, now: Instant, minimum_interval: Duration) -> bool {
    last_active_scan.is_none_or(|last| now.duration_since(last) >= minimum_interval)
}

/// Networks on the Associated Channel Plus Overlapping 2.4 GHz Neighbours, Weighted by Overlap, Scaled to 0-100.
pub fn congestion(active_channel: Option<u32>, per_channel: &BTreeMap<u32, u32>, network_weight: f64) -> Congestion {
    let Some(active_channel) = active_channel else {
        return Congestion { score: None, model: None };
    };
    let weighted = per_channel.iter().map(|(channel, count)| f64::from(*count) * overlap_weight(active_channel, *channel)).sum::<f64>();
    let model = if matches!(active_channel, 1..=14) { CONGESTION_MODEL_24_GHZ } else { CONGESTION_MODEL_SAME_CHANNEL };
    Congestion {
        score: Some((weighted * network_weight).min(LOST_PERCENT)),
        model: Some(model),
    }
}

/// The Scan Rows of One Collection: One per Network, or One Status Row When None Was Observed.
pub fn scan_rows(sample_index: u64, report: result::Result<ScanReport, String>, network_weight: f64) -> Vec<WifiScan> {
    let status_row = |status: &str, error: Option<String>| WifiScan {
        sample_index: Some(index(sample_index)),
        scan_status: Some(status.into()),
        scan_error: error,
        scan_network_count: Some(0),
        ..WifiScan::default()
    };
    let report = match report {
        Ok(report) => report,
        Err(error) => return vec![status_row("error", Some(error))],
    };
    if report.networks.is_empty() {
        let status = if report.error.is_some() { "error" } else { "unavailable" };
        return vec![status_row(status, report.error)];
    }
    let status = if report.active { "ok" } else { "cached" };
    let per_channel = channel_counts(&report.networks);
    let associated = report.networks.iter().find(|network| network.associated);
    let active_channel = associated.and_then(|network| network.channel);
    let active_ssid = associated.and_then(|network| network.ssid.clone());
    let congestion = congestion(active_channel, &per_channel, network_weight);
    let total = i64::try_from(report.networks.len()).unwrap_or(i64::MAX);
    report
        .networks
        .into_iter()
        .map(|network| WifiScan {
            sample_index: Some(index(sample_index)),
            scan_status: Some(status.into()),
            scan_error: report.error.clone(),
            wifi_adapter: network.adapter,
            wifi_ssid: network.ssid,
            wifi_bssid: network.bssid,
            wifi_signal_percent: network.signal_percent,
            wifi_rssi_dbm: network.rssi_dbm,
            wifi_channel: network.channel.map(i64::from),
            wifi_band: network.channel.and_then(band),
            wifi_security: network.security,
            associated: Some(network.associated),
            scan_network_count: Some(total),
            active_wifi_channel: active_channel.map(i64::from),
            active_wifi_band: active_channel.and_then(band),
            active_wifi_ssid: active_ssid.clone(),
            wifi_network_congestion_score: congestion.score,
            congestion_model: congestion.model.map(str::to_owned),
        })
        .collect()
}

/// Project One Native Report Into Both Streams Without Scanning Again.
pub fn scan_readings(
    sample_index: u64, report: result::Result<ScanReport, String>, network_weight: f64, plan: &mut ChannelPlan,
) -> ScanReadings {
    let channels = report
        .as_ref()
        .ok()
        .filter(|report| report.error.is_none() || !report.networks.is_empty())
        .map(|report| plan.rows(sample_index, report))
        .unwrap_or_default();
    let networks = scan_rows(sample_index, report, network_weight);
    ScanReadings { networks, channels }
}

/// The Latency Row of One Echo: a Reply, a Loss or a Probe That Could Not Run.
pub fn latency_row(sample_index: u64, host: &str, echo: result::Result<Option<f64>, String>) -> WifiLatency {
    let mut row = WifiLatency {
        sample_index: Some(index(sample_index)),
        wifi_latency_host: Some(host.into()),
        ..WifiLatency::default()
    };
    match echo {
        Ok(Some(latency_ms)) => {
            row.probe_status = Some("ok".into());
            row.wifi_latency_ms = Some(latency_ms);
            row.wifi_packet_loss_percent = Some(0.0);
        }
        Ok(None) => {
            row.probe_status = Some("timeout".into());
            row.wifi_packet_loss_percent = Some(LOST_PERCENT);
        }
        Err(error) => {
            row.probe_status = Some("error".into());
            row.probe_error = Some(error);
        }
    }
    row
}

/// Bytes per Second of the Named Adapter, Exactly or by Containment Either Way, Ignoring Case.
pub fn throughput(counters: &[Counters], adapter: &str, elapsed_s: f64) -> Option<(f64, f64)> {
    if let Some(exact) = counters.iter().find(|counters| counters.name.eq_ignore_ascii_case(adapter)) {
        return Some((exact.received as f64 / elapsed_s, exact.transmitted as f64 / elapsed_s));
    }
    let adapter = adapter.to_lowercase();
    let mut received = 0;
    let mut transmitted = 0;
    let mut matched = false;
    for counters in counters {
        let name = counters.name.to_lowercase();
        if name.contains(&adapter) || adapter.contains(&name) {
            matched = true;
            received += counters.received;
            transmitted += counters.transmitted;
        }
    }
    matched.then_some((received as f64 / elapsed_s, transmitted as f64 / elapsed_s))
}

/* === Private Functions === */

async fn blocking<T: Send + 'static>(collect: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    task::spawn_blocking(collect)
        .await
        .map_err(|error| Error::Invalid(format!("Collect Wi-Fi Telemetry: the collection worker failed: {error}; Restart the Plugin")))?
}

fn poisoned() -> Error {
    Error::Invalid("Collect Wi-Fi Telemetry: a previous collection panicked; Restart the Plugin".into())
}

fn index(sample_index: u64) -> i64 {
    i64::try_from(sample_index).unwrap_or(i64::MAX)
}

fn is_encoded_6ghz_channel(channel: u32) -> bool {
    let display_channel = channel.saturating_sub(1_000);
    channel >= 1_000 && (1..=233).contains(&display_channel) && display_channel % 4 == 1
}

fn overlap_weight(active_channel: u32, scanned_channel: u32) -> f64 {
    if matches!(active_channel, 1..=14) {
        let distance = f64::from(active_channel.abs_diff(scanned_channel));
        (1.0 - distance / OVERLAP_CHANNEL_SPAN).max(0.0)
    } else if active_channel == scanned_channel {
        1.0
    } else {
        0.0
    }
}

fn channel_counts(networks: &[ScannedNetwork]) -> BTreeMap<u32, u32> {
    let mut per_channel = BTreeMap::new();
    for channel in networks.iter().filter_map(|network| network.channel) {
        *per_channel.entry(channel).or_default() += 1;
    }
    per_channel
}

/// Whether a Network Occupies a Channel: Its Primary Channel, or a 20 MHz Cell of Its Bonded 5 or 6 GHz Span.
fn occupies(channel: u32, network: &ScannedNetwork) -> bool {
    let Some(primary) = network.channel else {
        return false;
    };
    if matches!(channel, 1..=14) || band(channel) != band(primary) {
        return channel == primary;
    }
    match (network.channel_width_mhz, network.center_channel) {
        // Channel Numbers Are Spaced 5 MHz; a Bonded Span Covers Its 20 MHz Cell Centers.
        (Some(width), Some(center)) if width >= 20 => channel.abs_diff(center) * 5 < width / 2,
        _ => channel == primary,
    }
}

/* === Tests === */
