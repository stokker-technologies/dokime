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
use crate::platform::unix::wifi::{NmcliWifiRow, adapter_matches, nmcli_wifi_rows, wireless_link};

/* === Implementations === */

impl WifiLinkCollector {
    // The Active NetworkManager Row Plus The Kernel Link; Answers The Adapter For Throughput.
    pub(super) fn read_link(&self, values: &mut BTreeMap<String, Value>) -> Option<String> {
        let rows = match nmcli_wifi_rows(false, crate::host::wifi_link::POLICY.attempt_timeout) {
            Ok(rows) => rows,
            Err(error) => {
                values.insert("probe_status".to_owned(), Value::from("error"));
                values.insert("probe_error".to_owned(), Value::from(error));
                return None;
            }
        };
        let filter = self.options.wifi_adapter_name.as_deref();
        let Some(link) = rows.into_iter().find(|row| row.active && adapter_matches(row.device.as_deref(), filter)) else {
            values.insert("probe_status".to_owned(), Value::from("unavailable"));
            values.insert("associated".to_owned(), Value::from(false));
            return None;
        };
        values.insert("probe_status".to_owned(), Value::from("ok"));
        values.insert("wifi_state".to_owned(), Value::from("connected"));
        values.insert("associated".to_owned(), Value::from(true));
        maybe_insert(values, "wifi_adapter", link.device.clone().map(Value::from));
        maybe_insert(values, "wifi_ssid", link.ssid.map(Value::from));
        maybe_insert(values, "wifi_bssid", link.bssid.map(Value::from));
        maybe_insert(values, "wifi_auth", link.security.map(Value::from));
        maybe_insert(values, "wifi_channel", link.channel.map(|channel| Value::Integer(i64::from(channel))));
        maybe_insert(values, "wifi_band", link.channel.and_then(wifi_channel_band).map(Value::from));
        maybe_insert(values, "wifi_signal_percent", link.signal_percent.map(Value::from));
        // The Kernel Link: Received Signal And Negotiated PHY Rates
        let wireless = link.device.as_deref().and_then(wireless_link).unwrap_or_default();
        maybe_insert(values, "wifi_rssi_dbm", wireless.rssi_dbm.map(Value::from));
        maybe_insert(values, "wifi_rx_phy_mbps", wireless.rx_phy_mbps.map(Value::from));
        maybe_insert(values, "wifi_tx_phy_mbps", wireless.tx_phy_mbps.map(Value::from));
        link.device
    }
}

impl WifiScanCollector {
    // NetworkManager Rescans Only When An Active Scan Is Due; A Refused Rescan Falls Back To Its Cached List.
    pub(super) fn scan(options: &HostOptions, active: bool) -> std::result::Result<ScanReport, String> {
        let timeout = crate::host::wifi_scan::POLICY.attempt_timeout;
        let (rows, active, error) = match nmcli_wifi_rows(active, timeout) {
            Ok(rows) => (rows, active, None),
            Err(error) if active => (nmcli_wifi_rows(false, timeout).map_err(|_| error.clone())?, false, Some(error)),
            Err(error) => return Err(error),
        };
        let filter = options.wifi_adapter_name.as_deref();
        let networks = rows.into_iter().filter(|row| adapter_matches(row.device.as_deref(), filter)).map(scanned).collect();
        Ok(ScanReport { networks, active, error })
    }
}

impl WifiLatencyCollector {
    pub(super) fn round_trip_ms(host: &str, timeout: Duration) -> std::result::Result<Option<f64>, String> {
        crate::platform::unix::echo::round_trip_ms(host, timeout)
    }
}

fn scanned(row: NmcliWifiRow) -> ScannedNetwork {
    ScannedNetwork {
        adapter: row.device,
        ssid: row.ssid,
        bssid: row.bssid,
        signal_percent: row.signal_percent,
        rssi_dbm: None,
        channel: row.channel,
        security: row.security,
        associated: row.active,
    }
}
