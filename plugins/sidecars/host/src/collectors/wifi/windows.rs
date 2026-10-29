/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use super::*;

impl WifiLinkCollector {
    // The Current Connection Through WlanQueryInterface; Answers The Adapter For Throughput.
    pub(super) fn read_link(&self, values: &mut BTreeMap<String, Value>) -> Option<String> {
        match crate::platform::windows::wifi::active_link(&self.options) {
            Ok(Some(link)) => {
                values.insert("probe_status".to_owned(), Value::from("ok"));
                values.insert("associated".to_owned(), Value::from(link.bssid.is_some()));
                maybe_insert(values, "wifi_adapter", link.adapter.clone().map(Value::from));
                maybe_insert(values, "wifi_state", link.state.map(Value::from));
                maybe_insert(values, "wifi_ssid", link.ssid.map(Value::from));
                maybe_insert(values, "wifi_bssid", link.bssid.map(Value::from));
                maybe_insert(values, "wifi_profile", link.profile.map(Value::from));
                maybe_insert(values, "wifi_radio", link.radio.map(Value::from));
                maybe_insert(values, "wifi_auth", link.auth.map(Value::from));
                maybe_insert(values, "wifi_cipher", link.cipher.map(Value::from));
                maybe_insert(values, "wifi_channel", link.channel.map(|channel| Value::Integer(i64::from(channel))));
                maybe_insert(values, "wifi_band", link.channel.and_then(wifi_channel_band).map(Value::from));
                maybe_insert(values, "wifi_signal_percent", link.signal_percent.map(Value::from));
                maybe_insert(values, "wifi_rssi_dbm", link.rssi_dbm.map(Value::from));
                maybe_insert(values, "wifi_rx_phy_mbps", link.rx_phy_mbps.map(Value::from));
                maybe_insert(values, "wifi_tx_phy_mbps", link.tx_phy_mbps.map(Value::from));
                link.adapter
            }
            Ok(None) => {
                values.insert("probe_status".to_owned(), Value::from("unavailable"));
                values.insert("associated".to_owned(), Value::from(false));
                None
            }
            Err(error) => {
                values.insert("probe_status".to_owned(), Value::from("error"));
                values.insert("probe_error".to_owned(), Value::from(error.to_string()));
                None
            }
        }
    }
}

impl WifiScanCollector {
    // WlanScan Only When An Active Scan Is Due; Otherwise The Driver's Cached BSS List.
    pub(super) fn scan(options: &HostOptions, active: bool) -> std::result::Result<ScanReport, String> {
        let window = active.then(scan_window);
        let networks = crate::platform::windows::wifi::scan_networks(options, window).map_err(|error| error.to_string())?;
        Ok(ScanReport { networks, active, error: None })
    }
}

impl WifiLatencyCollector {
    pub(super) fn round_trip_ms(host: &str, timeout: Duration) -> std::result::Result<Option<f64>, String> {
        crate::platform::windows::echo::round_trip_ms(host, timeout)
    }
}

// The Scan Completes Within The Collection Bound, Leaving Room To Read The List.
fn scan_window() -> Duration {
    crate::host::wifi_scan::POLICY.attempt_timeout / 2
}
