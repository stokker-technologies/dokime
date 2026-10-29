/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

use super::*;

/* === Implementations === */

impl WifiCollector {
    pub fn collect_link_with_options(&mut self, options: &HostOptions) -> Result<HostStreamSample> {
        self.sample_index += 1;
        let mut values = BTreeMap::from([
            ("sample_index".to_owned(), Value::Integer(self.sample_index as i64)),
            ("probe_status".to_owned(), Value::from("unsupported")),
        ]);

        match crate::platform::unix::wifi::nmcli_wifi_rows(false) {
            Ok(rows) => {
                let adapter_filter = options.wifi_adapter_name.as_deref();
                let link = rows
                    .into_iter()
                    .find(|row| row.active && crate::platform::unix::wifi::adapter_matches(row.device.as_deref(), adapter_filter));
                if let Some(link) = link {
                    values.insert("probe_status".to_owned(), Value::from("ok"));
                    values.insert("wifi_state".to_owned(), Value::from("connected"));
                    let adapter = link.device.clone();
                    maybe_insert(&mut values, "wifi_adapter", adapter.clone().map(Value::from));
                    maybe_insert(&mut values, "wifi_ssid", link.ssid.map(Value::from));
                    maybe_insert(&mut values, "wifi_bssid", link.bssid.map(Value::from));
                    maybe_insert(&mut values, "wifi_auth", link.security.map(Value::from));
                    maybe_insert(&mut values, "wifi_channel", link.channel.map(|channel| Value::Integer(i64::from(channel))));
                    maybe_insert(&mut values, "wifi_band", link.channel.and_then(wifi_channel_band).map(Value::from));
                    maybe_insert(&mut values, "wifi_signal_percent", link.signal_percent.map(Value::from));
                    // The Kernel Link: Received Signal And Negotiated PHY Rates
                    let wireless = adapter.as_deref().and_then(crate::platform::unix::wifi::wireless_link).unwrap_or_default();
                    maybe_insert(&mut values, "wifi_rssi_dbm", wireless.rssi_dbm.map(Value::from));
                    maybe_insert(&mut values, "wifi_rx_phy_mbps", wireless.rx_phy_mbps.map(Value::from));
                    maybe_insert(&mut values, "wifi_tx_phy_mbps", wireless.tx_phy_mbps.map(Value::from));
                    if let Some((received, sent)) = self.adapter_throughput(adapter.as_deref()) {
                        values.insert("wifi_rx_mbps".to_owned(), Value::from(received * 8.0 / BITS_PER_MEGABIT));
                        values.insert("wifi_tx_mbps".to_owned(), Value::from(sent * 8.0 / BITS_PER_MEGABIT));
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
}

impl WifiCollector {
    pub fn collect_channel_occupancy_with_options(&mut self, options: &HostOptions) -> Result<Vec<HostStreamSample>> {
        self.scan_index += 1;

        match linux_wifi_scan_rows() {
            Ok((rows, scan_status, scan_error)) => {
                let scan_timestamp = dokime_domain::model::time::Timestamp::from_system_time(std::time::SystemTime::now());
                let adapter_filter = options.wifi_adapter_name.as_deref();
                let rows = rows
                    .into_iter()
                    .filter(|row| crate::platform::unix::wifi::adapter_matches(row.device.as_deref(), adapter_filter))
                    .collect::<Vec<_>>();
                let active_channel = rows.iter().find(|row| row.active).and_then(|row| row.channel);
                let active_band = active_channel.and_then(wifi_channel_band);
                let active_adapter = rows.iter().find(|row| row.active).and_then(|row| row.device.clone());
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
                let scan_network_count = counts.iter().map(|count| count.network_count).sum::<u32>();
                let count_by_channel = counts.iter().map(|count| (count.channel, count.network_count)).collect::<BTreeMap<_, _>>();
                let samples = dense_wifi_occupancy_channels(&counts, active_channel)
                    .into_iter()
                    .map(|channel| {
                        let network_count = count_by_channel.get(&channel).copied().unwrap_or_default();
                        let mut values = BTreeMap::from([
                            ("sample_index".to_owned(), Value::Integer(self.scan_index as i64)),
                            ("scan_status".to_owned(), Value::from(scan_status.clone())),
                            ("wifi_channel".to_owned(), Value::Integer(i64::from(channel))),
                            ("network_count".to_owned(), Value::Integer(i64::from(network_count))),
                            ("scan_network_count".to_owned(), Value::Integer(i64::from(scan_network_count))),
                        ]);
                        maybe_insert(&mut values, "scan_error", scan_error.clone().map(Value::from));
                        maybe_insert(&mut values, "wifi_adapter", active_adapter.clone().map(Value::from));
                        maybe_insert(&mut values, "active_wifi_channel", active_channel.map(|channel| Value::Integer(i64::from(channel))));
                        maybe_insert(
                            &mut values,
                            "wifi_band",
                            wifi_channel_band(channel).or_else(|| congestion.active_band.clone()).map(Value::from),
                        );
                        insert_congestion_fields(&mut values, &congestion, Some(channel), network_count);
                        HostStreamSample::with_timestamp(HostTelemetryStream::WifiChannelOccupancy, scan_timestamp.clone(), values)
                    })
                    .collect();
                Ok(samples)
            }
            Err(error) => {
                let active = active_wifi_context(options);
                let congestion = wifi_congestion_summary(active.channel, active.band.clone(), &[]);
                self.update_congestion_cache(congestion.clone());
                let mut values = BTreeMap::from([
                    ("sample_index".to_owned(), Value::Integer(self.scan_index as i64)),
                    ("scan_status".to_owned(), Value::from("error")),
                    ("scan_error".to_owned(), Value::from(error)),
                    ("network_count".to_owned(), Value::Integer(0)),
                    ("scan_network_count".to_owned(), Value::Integer(0)),
                ]);
                maybe_insert(&mut values, "wifi_channel", active.channel.map(|channel| Value::Integer(i64::from(channel))));
                maybe_insert(&mut values, "wifi_adapter", active.adapter.map(Value::from));
                maybe_insert(&mut values, "wifi_band", active.band.map(Value::from));
                insert_congestion_fields(&mut values, &congestion, active.channel, 0);
                Ok(vec![HostStreamSample::new(HostTelemetryStream::WifiChannelOccupancy, values)])
            }
        }
    }
}

pub(super) fn active_wifi_context(options: &HostOptions) -> ActiveWifiContext {
    let adapter_filter = options.wifi_adapter_name.as_deref();
    crate::platform::unix::wifi::nmcli_wifi_rows(false)
        .ok()
        .and_then(|rows| {
            rows.into_iter().find(|row| row.active && crate::platform::unix::wifi::adapter_matches(row.device.as_deref(), adapter_filter))
        })
        .map(|row| ActiveWifiContext {
            channel: row.channel,
            band: row.channel.and_then(wifi_channel_band),
            adapter: row.device,
        })
        .unwrap_or_default()
}

fn linux_wifi_scan_rows() -> Result<(Vec<crate::platform::unix::wifi::NmcliWifiRow>, String, Option<String>), String> {
    match crate::platform::unix::wifi::nmcli_wifi_rows(true) {
        Ok(rows) => Ok((rows, "ok".to_owned(), None)),
        Err(error) => crate::platform::unix::wifi::nmcli_wifi_rows(false)
            .map(|rows| (rows, "cached".to_owned(), Some(error.clone())))
            .map_err(|_| error),
    }
}

impl WifiCollector {
    pub(super) fn ping_command(host: &str) -> Command {
        let timeout = crate::config::PING_REPLY_TIMEOUT;
        let mut command = Command::new("ping");
        command.args(["-c", "1", "-W", &timeout.as_secs().max(1).to_string(), host]);
        command
    }
}
