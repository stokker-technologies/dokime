/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#![cfg_attr(not(windows), allow(dead_code, unused_imports))]

/* === Modules === */

/* === Re-Exports === */

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
use dokime_contract::core::{Timestamp, Value};

/* === Constants === */
const WIFI_CONGESTION_MODEL_24_GHZ: &str = "2.4ghz_adjacent_channel_v1";
const WIFI_CONGESTION_MODEL_SAME_CHANNEL: &str = "same_channel_v1";
const DEFAULT_WIFI_LATENCY_HOST: &str = "1.1.1.1";
const BITS_PER_MEGABIT: f64 = 1_000_000.0;
const WIFI_5_GHZ_CHANNELS: &[u32] = &[
    32, 36, 40, 44, 48, 52, 56, 60, 64, 68, 96, 100, 104, 108, 112, 116, 120, 124, 128, 132, 136,
    140, 144, 149, 153, 157, 161, 165, 169, 173, 177,
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
struct ActiveWifiContext {
    channel: Option<u32>,
    band: Option<String>,
    adapter: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct WifiCongestionCache {
    latest: Option<WifiCongestionSummary>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl WifiCollector {
    pub(crate) fn new(
        options: HostOptions,
        congestion_cache: Arc<Mutex<WifiCongestionCache>>,
    ) -> Self {
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

    pub fn collect_link_with_options(&mut self, options: &HostOptions) -> Result<HostStreamSample> {
        self.sample_index += 1;
        let mut values = BTreeMap::from([
            (
                "sample_index".to_owned(),
                Value::Integer(self.sample_index as i64),
            ),
            ("probe_status".to_owned(), Value::from("unsupported")),
        ]);

        #[cfg(windows)]
        match crate::platform::windows::wifi::active_link(options) {
            Ok(Some(link)) => {
                values.insert("probe_status".to_owned(), Value::from("ok"));
                let adapter = link.adapter.clone();
                maybe_insert(
                    &mut values,
                    "wifi_adapter",
                    adapter.clone().map(Value::from),
                );
                maybe_insert(&mut values, "wifi_state", link.state.map(Value::from));
                maybe_insert(&mut values, "wifi_ssid", link.ssid.map(Value::from));
                maybe_insert(&mut values, "wifi_bssid", link.bssid.map(Value::from));
                maybe_insert(&mut values, "wifi_profile", link.profile.map(Value::from));
                maybe_insert(&mut values, "wifi_radio", link.radio.map(Value::from));
                maybe_insert(&mut values, "wifi_auth", link.auth.map(Value::from));
                maybe_insert(&mut values, "wifi_cipher", link.cipher.map(Value::from));
                maybe_insert(
                    &mut values,
                    "wifi_channel",
                    link.channel
                        .map(|channel| Value::Integer(i64::from(channel))),
                );
                maybe_insert(&mut values, "wifi_band", link.band.map(Value::from));
                maybe_insert(
                    &mut values,
                    "wifi_signal_percent",
                    link.signal_percent.map(Value::from),
                );
                maybe_insert(
                    &mut values,
                    "wifi_rx_phy_mbps",
                    link.rx_phy_mbps.map(Value::from),
                );
                maybe_insert(
                    &mut values,
                    "wifi_tx_phy_mbps",
                    link.tx_phy_mbps.map(Value::from),
                );
                if let Some((received, sent)) = self.adapter_throughput(adapter.as_deref()) {
                    values.insert(
                        "wifi_rx_mbps".to_owned(),
                        Value::from(received * 8.0 / BITS_PER_MEGABIT),
                    );
                    values.insert(
                        "wifi_tx_mbps".to_owned(),
                        Value::from(sent * 8.0 / BITS_PER_MEGABIT),
                    );
                }
                self.insert_latest_congestion_fields(&mut values);
            }
            Ok(None) => {
                values.insert("probe_status".to_owned(), Value::from("unavailable"));
            }
            Err(error) => {
                values.insert("probe_status".to_owned(), Value::from("error"));
                values.insert("probe_error".to_owned(), Value::from(error.to_string()));
            }
        }

        #[cfg(not(windows))]
        match crate::platform::linux::wifi::nmcli_wifi_rows(false) {
            Ok(rows) => {
                let adapter_filter = options.wifi_adapter_name.as_deref();
                let link = rows.into_iter().find(|row| {
                    row.active
                        && crate::platform::linux::wifi::adapter_matches(
                            row.device.as_deref(),
                            adapter_filter,
                        )
                });
                if let Some(link) = link {
                    values.insert("probe_status".to_owned(), Value::from("ok"));
                    values.insert("wifi_state".to_owned(), Value::from("connected"));
                    let adapter = link.device.clone();
                    maybe_insert(
                        &mut values,
                        "wifi_adapter",
                        adapter.clone().map(Value::from),
                    );
                    maybe_insert(&mut values, "wifi_ssid", link.ssid.map(Value::from));
                    maybe_insert(&mut values, "wifi_bssid", link.bssid.map(Value::from));
                    maybe_insert(&mut values, "wifi_auth", link.security.map(Value::from));
                    maybe_insert(
                        &mut values,
                        "wifi_channel",
                        link.channel
                            .map(|channel| Value::Integer(i64::from(channel))),
                    );
                    maybe_insert(
                        &mut values,
                        "wifi_band",
                        link.channel.and_then(wifi_channel_band).map(Value::from),
                    );
                    maybe_insert(
                        &mut values,
                        "wifi_signal_percent",
                        link.signal_percent.map(Value::from),
                    );
                    maybe_insert(
                        &mut values,
                        "wifi_rx_phy_mbps",
                        link.rate_mbps.map(Value::from),
                    );
                    maybe_insert(
                        &mut values,
                        "wifi_tx_phy_mbps",
                        link.rate_mbps.map(Value::from),
                    );
                    if let Some((received, sent)) = self.adapter_throughput(adapter.as_deref()) {
                        values.insert(
                            "wifi_rx_mbps".to_owned(),
                            Value::from(received * 8.0 / BITS_PER_MEGABIT),
                        );
                        values.insert(
                            "wifi_tx_mbps".to_owned(),
                            Value::from(sent * 8.0 / BITS_PER_MEGABIT),
                        );
                    }
                    self.insert_latest_congestion_fields(&mut values);
                } else {
                    values.insert("probe_status".to_owned(), Value::from("unavailable"));
                }
            }
            Err(error) => {
                values.insert("probe_status".to_owned(), Value::from("error"));
                values.insert("probe_error".to_owned(), Value::from(error));
            }
        }

        Ok(HostStreamSample::new(HostTelemetryStream::Wifi, values))
    }

    pub fn collect_latency_with_options(
        &mut self,
        options: &HostOptions,
    ) -> Result<HostStreamSample> {
        self.latency_sample_index += 1;
        let host = options
            .latency_host
            .as_deref()
            .or(self.options.latency_host.as_deref())
            .filter(|host| !host.trim().is_empty())
            .unwrap_or(DEFAULT_WIFI_LATENCY_HOST)
            .trim()
            .to_owned();
        let mut values = BTreeMap::from([
            (
                "sample_index".to_owned(),
                Value::Integer(self.latency_sample_index as i64),
            ),
            ("wifi_latency_host".to_owned(), Value::from(host.clone())),
            ("probe_status".to_owned(), Value::from("unsupported")),
        ]);

        match ping_latency_ms(&host) {
            Ok(latency_ms) => {
                values.insert("probe_status".to_owned(), Value::from("ok"));
                values.insert("wifi_latency_ms".to_owned(), Value::from(latency_ms));
            }
            Err(error) => {
                values.insert("probe_status".to_owned(), Value::from("error"));
                values.insert("probe_error".to_owned(), Value::from(error));
            }
        }

        Ok(HostStreamSample::new(
            HostTelemetryStream::WifiLatency,
            values,
        ))
    }

    fn adapter_throughput(&mut self, adapter: Option<&str>) -> Option<(f64, f64)> {
        let now = Instant::now();
        let elapsed_s = now
            .duration_since(self.last_network_refresh)
            .as_secs_f64()
            .max(0.001);
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

    pub fn collect_channel_occupancy_with_options(
        &mut self,
        options: &HostOptions,
    ) -> Result<Vec<HostStreamSample>> {
        self.scan_index += 1;

        #[cfg(windows)]
        {
            match crate::platform::windows::wifi::scan_channel_occupancy(options) {
                Ok(channels) => {
                    let scan_timestamp = Timestamp::now_utc();
                    let active_channel = channels.iter().find_map(|channel| channel.active_channel);
                    let active_band = channels
                        .iter()
                        .find_map(|channel| channel.active_band.clone());
                    let active_adapter =
                        channels.iter().find_map(|channel| channel.adapter.clone());
                    let counts = channels
                        .iter()
                        .map(|channel| WifiChannelCount {
                            channel: channel.channel,
                            network_count: channel.network_count,
                        })
                        .collect::<Vec<_>>();
                    let congestion = wifi_congestion_summary(active_channel, active_band, &counts);
                    self.update_congestion_cache(congestion.clone());

                    if channels.is_empty() {
                        let mut values = BTreeMap::from([
                            (
                                "sample_index".to_owned(),
                                Value::Integer(self.scan_index as i64),
                            ),
                            ("scan_status".to_owned(), Value::from("unavailable")),
                            ("network_count".to_owned(), Value::Integer(0)),
                            ("scan_network_count".to_owned(), Value::Integer(0)),
                        ]);
                        maybe_insert(
                            &mut values,
                            "wifi_channel",
                            active_channel.map(|channel| Value::Integer(i64::from(channel))),
                        );
                        maybe_insert(&mut values, "wifi_adapter", active_adapter.map(Value::from));
                        maybe_insert(
                            &mut values,
                            "wifi_band",
                            congestion.active_band.clone().map(Value::from),
                        );
                        insert_congestion_fields(&mut values, &congestion, active_channel, 0);
                        return Ok(vec![HostStreamSample::with_timestamp(
                            HostTelemetryStream::WifiChannelOccupancy,
                            scan_timestamp,
                            values,
                        )]);
                    }

                    let scan_network_count =
                        counts.iter().map(|count| count.network_count).sum::<u32>();
                    let channels_by_id = channels
                        .into_iter()
                        .map(|channel| (channel.channel, channel))
                        .collect::<BTreeMap<_, _>>();
                    let samples = dense_wifi_occupancy_channels(&counts, active_channel)
                        .into_iter()
                        .map(|channel| {
                            let row = channels_by_id.get(&channel);
                            let network_count =
                                row.map(|channel| channel.network_count).unwrap_or_default();
                            let mut values = BTreeMap::from([
                                (
                                    "sample_index".to_owned(),
                                    Value::Integer(self.scan_index as i64),
                                ),
                                ("scan_status".to_owned(), Value::from("ok")),
                                (
                                    "wifi_channel".to_owned(),
                                    Value::Integer(i64::from(channel)),
                                ),
                                (
                                    "network_count".to_owned(),
                                    Value::Integer(network_count as i64),
                                ),
                                (
                                    "scan_network_count".to_owned(),
                                    Value::Integer(scan_network_count as i64),
                                ),
                            ]);
                            maybe_insert(
                                &mut values,
                                "wifi_adapter",
                                row.and_then(|channel| channel.adapter.clone())
                                    .or_else(|| active_adapter.clone())
                                    .map(Value::from),
                            );
                            maybe_insert(
                                &mut values,
                                "wifi_band",
                                row.and_then(|channel| channel.band.clone())
                                    .or_else(|| wifi_channel_band(channel))
                                    .map(Value::from),
                            );
                            maybe_insert(
                                &mut values,
                                "max_signal_percent",
                                row.and_then(|channel| channel.max_signal_percent)
                                    .map(Value::from),
                            );
                            insert_congestion_fields(
                                &mut values,
                                &congestion,
                                Some(channel),
                                network_count,
                            );

                            HostStreamSample::with_timestamp(
                                HostTelemetryStream::WifiChannelOccupancy,
                                scan_timestamp.clone(),
                                values,
                            )
                        })
                        .collect::<Vec<_>>();
                    Ok(samples)
                }
                Err(error) => {
                    let scan_timestamp = Timestamp::now_utc();
                    let active = active_wifi_context(options);
                    let congestion =
                        wifi_congestion_summary(active.channel, active.band.clone(), &[]);
                    self.update_congestion_cache(congestion.clone());
                    let mut values = BTreeMap::from([
                        (
                            "sample_index".to_owned(),
                            Value::Integer(self.scan_index as i64),
                        ),
                        ("scan_status".to_owned(), Value::from("error")),
                        ("scan_error".to_owned(), Value::from(error.to_string())),
                        ("network_count".to_owned(), Value::Integer(0)),
                        ("scan_network_count".to_owned(), Value::Integer(0)),
                    ]);
                    maybe_insert(
                        &mut values,
                        "wifi_channel",
                        active
                            .channel
                            .map(|channel| Value::Integer(i64::from(channel))),
                    );
                    maybe_insert(&mut values, "wifi_adapter", active.adapter.map(Value::from));
                    maybe_insert(&mut values, "wifi_band", active.band.map(Value::from));
                    insert_congestion_fields(&mut values, &congestion, active.channel, 0);
                    Ok(vec![HostStreamSample::with_timestamp(
                        HostTelemetryStream::WifiChannelOccupancy,
                        scan_timestamp,
                        values,
                    )])
                }
            }
        }

        #[cfg(not(windows))]
        {
            match linux_wifi_scan_rows() {
                Ok((rows, scan_status, scan_error)) => {
                    let scan_timestamp = Timestamp::now_utc();
                    let adapter_filter = options.wifi_adapter_name.as_deref();
                    let rows = rows
                        .into_iter()
                        .filter(|row| {
                            crate::platform::linux::wifi::adapter_matches(
                                row.device.as_deref(),
                                adapter_filter,
                            )
                        })
                        .collect::<Vec<_>>();
                    let active_channel = rows
                        .iter()
                        .find(|row| row.active)
                        .and_then(|row| row.channel);
                    let active_band = active_channel.and_then(wifi_channel_band);
                    let active_adapter = rows
                        .iter()
                        .find(|row| row.active)
                        .and_then(|row| row.device.clone());
                    let mut by_channel = BTreeMap::<u32, u32>::new();
                    for row in &rows {
                        if let Some(channel) = row.channel {
                            *by_channel.entry(channel).or_default() += 1;
                        }
                    }
                    let counts = by_channel
                        .iter()
                        .map(|(channel, network_count)| WifiChannelCount {
                            channel: *channel,
                            network_count: *network_count,
                        })
                        .collect::<Vec<_>>();
                    let congestion = wifi_congestion_summary(active_channel, active_band, &counts);
                    self.update_congestion_cache(congestion.clone());
                    let scan_network_count =
                        counts.iter().map(|count| count.network_count).sum::<u32>();
                    let count_by_channel = counts
                        .iter()
                        .map(|count| (count.channel, count.network_count))
                        .collect::<BTreeMap<_, _>>();
                    let samples = dense_wifi_occupancy_channels(&counts, active_channel)
                        .into_iter()
                        .map(|channel| {
                            let network_count =
                                count_by_channel.get(&channel).copied().unwrap_or_default();
                            let mut values = BTreeMap::from([
                                (
                                    "sample_index".to_owned(),
                                    Value::Integer(self.scan_index as i64),
                                ),
                                ("scan_status".to_owned(), Value::from(scan_status.clone())),
                                (
                                    "wifi_channel".to_owned(),
                                    Value::Integer(i64::from(channel)),
                                ),
                                (
                                    "network_count".to_owned(),
                                    Value::Integer(i64::from(network_count)),
                                ),
                                (
                                    "scan_network_count".to_owned(),
                                    Value::Integer(i64::from(scan_network_count)),
                                ),
                            ]);
                            maybe_insert(
                                &mut values,
                                "scan_error",
                                scan_error.clone().map(Value::from),
                            );
                            maybe_insert(
                                &mut values,
                                "wifi_adapter",
                                active_adapter.clone().map(Value::from),
                            );
                            maybe_insert(
                                &mut values,
                                "active_wifi_channel",
                                active_channel.map(|channel| Value::Integer(i64::from(channel))),
                            );
                            maybe_insert(
                                &mut values,
                                "wifi_band",
                                wifi_channel_band(channel)
                                    .or_else(|| congestion.active_band.clone())
                                    .map(Value::from),
                            );
                            insert_congestion_fields(
                                &mut values,
                                &congestion,
                                Some(channel),
                                network_count,
                            );
                            HostStreamSample::with_timestamp(
                                HostTelemetryStream::WifiChannelOccupancy,
                                scan_timestamp.clone(),
                                values,
                            )
                        })
                        .collect();
                    Ok(samples)
                }
                Err(error) => {
                    let active = active_wifi_context(options);
                    let congestion =
                        wifi_congestion_summary(active.channel, active.band.clone(), &[]);
                    self.update_congestion_cache(congestion.clone());
                    let mut values = BTreeMap::from([
                        (
                            "sample_index".to_owned(),
                            Value::Integer(self.scan_index as i64),
                        ),
                        ("scan_status".to_owned(), Value::from("error")),
                        ("scan_error".to_owned(), Value::from(error)),
                        ("network_count".to_owned(), Value::Integer(0)),
                        ("scan_network_count".to_owned(), Value::Integer(0)),
                    ]);
                    maybe_insert(
                        &mut values,
                        "wifi_channel",
                        active
                            .channel
                            .map(|channel| Value::Integer(i64::from(channel))),
                    );
                    maybe_insert(&mut values, "wifi_adapter", active.adapter.map(Value::from));
                    maybe_insert(&mut values, "wifi_band", active.band.map(Value::from));
                    insert_congestion_fields(&mut values, &congestion, active.channel, 0);
                    Ok(vec![HostStreamSample::new(
                        HostTelemetryStream::WifiChannelOccupancy,
                        values,
                    )])
                }
            }
        }
    }

    fn update_congestion_cache(&self, congestion: WifiCongestionSummary) {
        self.congestion_cache
            .lock()
            .expect("wifi congestion cache mutex is not poisoned")
            .latest = Some(congestion);
    }

    fn insert_latest_congestion_fields(&self, values: &mut BTreeMap<String, Value>) {
        let latest = self
            .congestion_cache
            .lock()
            .expect("wifi congestion cache mutex is not poisoned")
            .latest
            .clone();
        let Some(congestion) = latest else {
            return;
        };
        maybe_insert(
            values,
            "active_wifi_channel",
            congestion
                .active_channel
                .map(|channel| Value::Integer(i64::from(channel))),
        );
        maybe_insert(
            values,
            "active_wifi_band",
            congestion.active_band.map(Value::from),
        );
        maybe_insert(
            values,
            "wifi_network_congestion_score",
            congestion.score.map(Value::from),
        );
        maybe_insert(
            values,
            "congestion_model",
            congestion.model.map(Value::from),
        );
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
    active_channel: Option<u32>,
    active_band: Option<String>,
    channels: &[WifiChannelCount],
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

fn active_wifi_context(options: &HostOptions) -> ActiveWifiContext {
    #[cfg(windows)]
    {
        crate::platform::windows::wifi::active_link(options)
            .ok()
            .flatten()
            .map(|link| ActiveWifiContext {
                channel: link.channel,
                band: link.band,
                adapter: link.adapter,
            })
            .unwrap_or_default()
    }

    #[cfg(not(windows))]
    {
        let adapter_filter = options.wifi_adapter_name.as_deref();
        crate::platform::linux::wifi::nmcli_wifi_rows(false)
            .ok()
            .and_then(|rows| {
                rows.into_iter().find(|row| {
                    row.active
                        && crate::platform::linux::wifi::adapter_matches(
                            row.device.as_deref(),
                            adapter_filter,
                        )
                })
            })
            .map(|row| ActiveWifiContext {
                channel: row.channel,
                band: row.channel.and_then(wifi_channel_band),
                adapter: row.device,
            })
            .unwrap_or_default()
    }
}

#[cfg(not(windows))]
fn linux_wifi_scan_rows() -> Result<
    (
        Vec<crate::platform::linux::wifi::NmcliWifiRow>,
        String,
        Option<String>,
    ),
    String,
> {
    match crate::platform::linux::wifi::nmcli_wifi_rows(true) {
        Ok(rows) => Ok((rows, "ok".to_owned(), None)),
        Err(error) => crate::platform::linux::wifi::nmcli_wifi_rows(false)
            .map(|rows| (rows, "cached".to_owned(), Some(error.clone())))
            .map_err(|_| error),
    }
}

fn insert_congestion_fields(
    values: &mut BTreeMap<String, Value>,
    congestion: &WifiCongestionSummary,
    row_channel: Option<u32>,
    row_network_count: u32,
) {
    maybe_insert(
        values,
        "active_wifi_channel",
        congestion
            .active_channel
            .map(|channel| Value::Integer(i64::from(channel))),
    );
    maybe_insert(
        values,
        "active_wifi_band",
        congestion.active_band.clone().map(Value::from),
    );
    let active_channel = congestion.active_channel == row_channel;
    values.insert("active_channel".to_owned(), Value::from(active_channel));
    let overlap_weight = congestion
        .active_channel
        .zip(row_channel)
        .map_or(0.0, |(active, channel)| {
            wifi_overlap_weight(active, channel)
        });
    values.insert("overlap_weight".to_owned(), Value::from(overlap_weight));
    values.insert(
        "same_channel_network_count".to_owned(),
        Value::Integer(i64::from(congestion.same_channel_network_count)),
    );
    values.insert(
        "overlapping_network_count".to_owned(),
        Value::Integer(i64::from(congestion.overlapping_network_count)),
    );
    values.insert(
        "overlap_weighted_network_count".to_owned(),
        Value::from(congestion.overlap_weighted_network_count),
    );
    maybe_insert(
        values,
        "wifi_network_congestion_score",
        congestion.score.map(Value::from),
    );
    maybe_insert(
        values,
        "congestion_model",
        congestion.model.map(Value::from),
    );
    if active_channel && row_network_count == 0 {
        values.insert("max_signal_percent".to_owned(), Value::Null);
    }
}

fn dense_wifi_occupancy_channels(
    counts: &[WifiChannelCount],
    active_channel: Option<u32>,
) -> Vec<u32> {
    let mut channels = BTreeMap::new();
    for channel in 1..=14 {
        channels.insert(channel, ());
    }
    for channel in WIFI_5_GHZ_CHANNELS {
        channels.insert(*channel, ());
    }
    if active_channel.is_some_and(is_encoded_6ghz_channel)
        || counts
            .iter()
            .any(|count| is_encoded_6ghz_channel(count.channel))
    {
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

fn ping_latency_ms(host: &str) -> Result<f64, String> {
    let output = super::run_command_bounded(ping_command(host), Duration::from_secs(5))
        .map_err(|error| format!("ping command failed: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output_text = format!("{stdout}\n{stderr}");

    if !output.status.success() {
        return Err(output_text.trim().to_owned());
    }

    parse_ping_latency_ms(&output_text)
        .ok_or_else(|| format!("ping latency was not found in command output for {host}"))
}

fn ping_command(host: &str) -> Command {
    let mut command = Command::new("ping");
    if cfg!(windows) {
        command.args(["-n", "1", "-w", "2000", host]);
    } else {
        command.args(["-c", "1", "-W", "2", host]);
    }
    command
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
        return Some(if marker == "time<" {
            value.max(0.5)
        } else {
            value
        });
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
    if matches!(active_channel, 1..=14) {
        WIFI_CONGESTION_MODEL_24_GHZ
    } else {
        WIFI_CONGESTION_MODEL_SAME_CHANNEL
    }
}

/* === Tests === */
