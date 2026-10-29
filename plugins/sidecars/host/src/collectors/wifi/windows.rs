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

        match crate::platform::windows::wifi::active_link(options) {
            Ok(Some(link)) => {
                values.insert("probe_status".to_owned(), Value::from("ok"));
                let adapter = link.adapter.clone();
                maybe_insert(&mut values, "wifi_adapter", adapter.clone().map(Value::from));
                maybe_insert(&mut values, "wifi_state", link.state.map(Value::from));
                maybe_insert(&mut values, "wifi_ssid", link.ssid.map(Value::from));
                maybe_insert(&mut values, "wifi_bssid", link.bssid.map(Value::from));
                maybe_insert(&mut values, "wifi_profile", link.profile.map(Value::from));
                maybe_insert(&mut values, "wifi_radio", link.radio.map(Value::from));
                maybe_insert(&mut values, "wifi_auth", link.auth.map(Value::from));
                maybe_insert(&mut values, "wifi_cipher", link.cipher.map(Value::from));
                maybe_insert(&mut values, "wifi_channel", link.channel.map(|channel| Value::Integer(i64::from(channel))));
                maybe_insert(&mut values, "wifi_band", link.band.map(Value::from));
                maybe_insert(&mut values, "wifi_signal_percent", link.signal_percent.map(Value::from));
                maybe_insert(&mut values, "wifi_rssi_dbm", link.rssi_dbm.map(Value::from));
                maybe_insert(&mut values, "wifi_rx_phy_mbps", link.rx_phy_mbps.map(Value::from));
                maybe_insert(&mut values, "wifi_tx_phy_mbps", link.tx_phy_mbps.map(Value::from));
                if let Some((received, sent)) = self.adapter_throughput(adapter.as_deref()) {
                    values.insert("wifi_rx_mbps".to_owned(), Value::from(received * 8.0 / BITS_PER_MEGABIT));
                    values.insert("wifi_tx_mbps".to_owned(), Value::from(sent * 8.0 / BITS_PER_MEGABIT));
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

        Ok(HostStreamSample::new(HostTelemetryStream::Wifi, values))
    }
}

impl WifiCollector {
    pub fn collect_channel_occupancy_with_options(&mut self, options: &HostOptions) -> Result<Vec<HostStreamSample>> {
        self.scan_index += 1;

        match crate::platform::windows::wifi::scan_channel_occupancy(options, crate::config::WIFI_SCAN_WINDOW) {
            Ok(channels) => {
                let scan_timestamp = dokime_domain::model::time::Timestamp::from_system_time(std::time::SystemTime::now());
                let active_channel = channels.iter().find_map(|channel| channel.active_channel);
                let active_band = channels.iter().find_map(|channel| channel.active_band.clone());
                let active_adapter = channels.iter().find_map(|channel| channel.adapter.clone());
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
                        ("sample_index".to_owned(), Value::Integer(self.scan_index as i64)),
                        ("scan_status".to_owned(), Value::from("unavailable")),
                        ("network_count".to_owned(), Value::Integer(0)),
                        ("scan_network_count".to_owned(), Value::Integer(0)),
                    ]);
                    maybe_insert(&mut values, "wifi_channel", active_channel.map(|channel| Value::Integer(i64::from(channel))));
                    maybe_insert(&mut values, "wifi_adapter", active_adapter.map(Value::from));
                    maybe_insert(&mut values, "wifi_band", congestion.active_band.clone().map(Value::from));
                    insert_congestion_fields(&mut values, &congestion, active_channel, 0);
                    return Ok(vec![HostStreamSample::with_timestamp(HostTelemetryStream::WifiChannelOccupancy, scan_timestamp, values)]);
                }

                let scan_network_count = counts.iter().map(|count| count.network_count).sum::<u32>();
                let channels_by_id = channels.into_iter().map(|channel| (channel.channel, channel)).collect::<BTreeMap<_, _>>();
                let samples = dense_wifi_occupancy_channels(&counts, active_channel)
                    .into_iter()
                    .map(|channel| {
                        let row = channels_by_id.get(&channel);
                        let network_count = row.map(|channel| channel.network_count).unwrap_or_default();
                        let mut values = BTreeMap::from([
                            ("sample_index".to_owned(), Value::Integer(self.scan_index as i64)),
                            ("scan_status".to_owned(), Value::from("ok")),
                            ("wifi_channel".to_owned(), Value::Integer(i64::from(channel))),
                            ("network_count".to_owned(), Value::Integer(network_count as i64)),
                            ("scan_network_count".to_owned(), Value::Integer(scan_network_count as i64)),
                        ]);
                        maybe_insert(
                            &mut values,
                            "wifi_adapter",
                            row.and_then(|channel| channel.adapter.clone()).or_else(|| active_adapter.clone()).map(Value::from),
                        );
                        maybe_insert(
                            &mut values,
                            "wifi_band",
                            row.and_then(|channel| channel.band.clone()).or_else(|| wifi_channel_band(channel)).map(Value::from),
                        );
                        maybe_insert(
                            &mut values,
                            "max_signal_percent",
                            row.and_then(|channel| channel.max_signal_percent).map(Value::from),
                        );
                        insert_congestion_fields(&mut values, &congestion, Some(channel), network_count);

                        HostStreamSample::with_timestamp(HostTelemetryStream::WifiChannelOccupancy, scan_timestamp.clone(), values)
                    })
                    .collect::<Vec<_>>();
                Ok(samples)
            }
            Err(error) => {
                let scan_timestamp = dokime_domain::model::time::Timestamp::from_system_time(std::time::SystemTime::now());
                let active = active_wifi_context(options);
                let congestion = wifi_congestion_summary(active.channel, active.band.clone(), &[]);
                self.update_congestion_cache(congestion.clone());
                let mut values = BTreeMap::from([
                    ("sample_index".to_owned(), Value::Integer(self.scan_index as i64)),
                    ("scan_status".to_owned(), Value::from("error")),
                    ("scan_error".to_owned(), Value::from(error.to_string())),
                    ("network_count".to_owned(), Value::Integer(0)),
                    ("scan_network_count".to_owned(), Value::Integer(0)),
                ]);
                maybe_insert(&mut values, "wifi_channel", active.channel.map(|channel| Value::Integer(i64::from(channel))));
                maybe_insert(&mut values, "wifi_adapter", active.adapter.map(Value::from));
                maybe_insert(&mut values, "wifi_band", active.band.map(Value::from));
                insert_congestion_fields(&mut values, &congestion, active.channel, 0);
                Ok(vec![HostStreamSample::with_timestamp(HostTelemetryStream::WifiChannelOccupancy, scan_timestamp, values)])
            }
        }
    }
}

impl WifiCollector {
    pub(super) fn ping_command(host: &str) -> Command {
        let timeout = crate::config::PING_REPLY_TIMEOUT;
        let mut command = Command::new("ping");
        command.args(["-n", "1", "-w", &timeout.as_millis().to_string(), host]);
        command
    }
}

fn active_wifi_context(options: &HostOptions) -> ActiveWifiContext {
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
