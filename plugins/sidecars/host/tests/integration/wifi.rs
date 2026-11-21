/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Wi-Fi Telemetry Stories: Scan Assembly, Congestion, Bands, Latency Outcomes and Throughput

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

// Third Party Imports

// Local Imports
use plugin_host::collectors::wifi::{
    CONGESTION_MODEL_24_GHZ, CONGESTION_MODEL_SAME_CHANNEL, Counters, ScanReport, band, congestion, latency_row, scan_due, scan_rows,
    throughput,
};
use plugin_host::platform::ScannedNetwork;

/* === Constants === */

const WEIGHT: f64 = 10.0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn network(bssid: &str, channel: u32, associated: bool) -> ScannedNetwork {
    ScannedNetwork {
        adapter: Some("Wireless".into()),
        ssid: Some(format!("Network {bssid}")),
        bssid: Some(bssid.into()),
        channel: Some(channel),
        associated,
        ..ScannedNetwork::default()
    }
}

/* === Tests === */

#[test]
fn congestion_weighs_overlapping_channels() {
    let per_channel = BTreeMap::from([(6, 2), (8, 1), (11, 4), (36, 3)]);
    // Channel 6 Counts Fully, Channel 8 at 0.6, Channel 11 Not at All.
    let weighed = congestion(Some(6), &per_channel, WEIGHT);
    assert_eq!(weighed.model, Some(CONGESTION_MODEL_24_GHZ));
    assert!((weighed.score.unwrap() - 26.0).abs() < 1e-9);
    assert!((congestion(Some(6), &per_channel, 13.0).score.unwrap() - 33.8).abs() < 1e-9);
    let same_channel = congestion(Some(36), &per_channel, WEIGHT);
    assert_eq!(same_channel.model, Some(CONGESTION_MODEL_SAME_CHANNEL));
    assert!((same_channel.score.unwrap() - 30.0).abs() < 1e-9);
    assert_eq!(congestion(Some(6), &BTreeMap::from([(6, 20)]), WEIGHT).score, Some(100.0));
    assert_eq!(congestion(None, &per_channel, WEIGHT).score, None);
}

#[test]
fn channels_map_to_bands() {
    assert_eq!(band(6).as_deref(), Some("2.4GHz"));
    assert_eq!(band(36).as_deref(), Some("5GHz"));
    assert_eq!(band(1_005).as_deref(), Some("6GHz"));
    assert_eq!(band(1_006), None);
    assert_eq!(band(200), None);
}

#[test]
fn a_scan_is_one_row_per_network_sharing_its_collection() {
    let report = ScanReport {
        networks: vec![network("a", 6, true), network("b", 6, false), network("c", 36, false)],
        active: true,
        error: None,
    };
    let rows = scan_rows(3, Ok(report), WEIGHT);
    assert_eq!(rows.len(), 3);
    for row in &rows {
        assert_eq!(row.sample_index, Some(3));
        assert_eq!(row.scan_status.as_deref(), Some("ok"));
        assert_eq!(row.scan_network_count, Some(3));
        assert_eq!(row.active_wifi_channel, Some(6));
        assert_eq!(row.active_wifi_band.as_deref(), Some("2.4GHz"));
        assert_eq!(row.active_wifi_ssid.as_deref(), Some("Network a"));
        assert_eq!(row.wifi_network_congestion_score, Some(20.0));
        assert_eq!(row.congestion_model.as_deref(), Some(CONGESTION_MODEL_24_GHZ));
    }
    assert_eq!(rows.iter().filter(|row| row.associated == Some(true)).count(), 1);
    assert_eq!(rows[0].network_count, Some(2));
    assert_eq!(rows[2].network_count, Some(1));
    assert_eq!(rows[2].wifi_band.as_deref(), Some("5GHz"));
}

#[test]
fn a_cached_scan_keeps_its_rescan_error() {
    let report = ScanReport {
        networks: vec![network("a", 11, false)],
        active: false,
        error: Some("rescan refused".into()),
    };
    let rows = scan_rows(1, Ok(report), WEIGHT);
    assert_eq!(rows[0].scan_status.as_deref(), Some("cached"));
    assert_eq!(rows[0].scan_error.as_deref(), Some("rescan refused"));
    assert_eq!(rows[0].active_wifi_channel, None);
    assert_eq!(rows[0].wifi_network_congestion_score, None);
}

#[test]
fn an_empty_or_failed_scan_is_one_status_row() {
    let empty = scan_rows(1, Ok(ScanReport::default()), WEIGHT);
    assert_eq!(empty.len(), 1);
    assert_eq!(empty[0].scan_status.as_deref(), Some("unavailable"));
    assert_eq!(empty[0].scan_network_count, Some(0));
    let refused = ScanReport {
        error: Some("refused".into()),
        ..ScanReport::default()
    };
    assert_eq!(scan_rows(1, Ok(refused), WEIGHT)[0].scan_status.as_deref(), Some("error"));
    let failed = scan_rows(2, Err("failed".into()), WEIGHT);
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].scan_status.as_deref(), Some("error"));
    assert_eq!(failed[0].scan_error.as_deref(), Some("failed"));
}

#[test]
fn active_scans_honor_the_minimum_interval() {
    let opened = Instant::now();
    let minimum = Duration::from_secs(30);
    assert!(scan_due(None, opened, minimum));
    assert!(!scan_due(Some(opened), opened + Duration::from_secs(29), minimum));
    assert!(scan_due(Some(opened), opened + minimum, minimum));
}

#[test]
fn an_echo_reply_reports_the_measured_latency() {
    let reply = latency_row(1, "host", Ok(Some(12.5)));
    assert_eq!(reply.probe_status.as_deref(), Some("ok"));
    assert_eq!(reply.wifi_latency_ms, Some(12.5));
    assert_eq!(reply.wifi_packet_loss_percent, Some(0.0));
    assert_eq!(reply.probe_error, None);
}

#[test]
fn an_echo_timeout_reports_loss_without_a_latency() {
    let lost = latency_row(2, "host", Ok(None));
    assert_eq!(lost.probe_status.as_deref(), Some("timeout"));
    assert_eq!(lost.wifi_latency_ms, None);
    assert_eq!(lost.wifi_packet_loss_percent, Some(100.0));
    assert_eq!(lost.probe_error, None);
}

#[test]
fn an_echo_failure_has_no_measured_values() {
    let error = String::from("probe-failure");
    let failed = latency_row(3, "host", Err(error.clone()));
    assert_eq!(failed.probe_status.as_deref(), Some("error"));
    assert_eq!(failed.probe_error, Some(error));
    assert_eq!(failed.wifi_latency_ms, None);
    assert_eq!(failed.wifi_packet_loss_percent, None);
    assert_eq!(failed.wifi_latency_host.as_deref(), Some("host"));
}

#[test]
fn throughput_matches_the_adapter_exactly_or_by_containment() {
    let counters = [
        Counters {
            name: "Wi-Fi".into(),
            received: 2_000,
            transmitted: 1_000,
        },
        Counters {
            name: "Wi-Fi 2".into(),
            received: 400,
            transmitted: 200,
        },
    ];
    assert_eq!(throughput(&counters, "wi-fi", 2.0), Some((1_000.0, 500.0)));
    assert_eq!(throughput(&counters, "Intel Wi-Fi 2 Adapter", 2.0), Some((1_200.0, 600.0)));
    assert_eq!(throughput(&counters, "Ethernet", 2.0), None);
}

#[cfg(unix)]
mod unix {
    use plugin_host::platform::unix::echo::parse_ping_latency_ms;
    use plugin_host::platform::unix::wifi::{NmcliWifiRow, WifiLinkReport, adapter_matches, parse_iw_interfaces};

    const HE_160_LINK: &str = "Connected to a4:ce:da:c0:83:f0 (on wlp5s0)
\tSSID: Arcadia
\tfreq: 5540.0
\tsignal: -46 dBm
\trx bitrate: 2401.9 MBit/s 160MHz HE-MCS 11 HE-NSS 2 HE-GI 0 HE-DCM 0
\ttx bitrate: 2401.9 MBit/s 160MHz HE-MCS 11 HE-NSS 2 HE-GI 0 HE-DCM 0
";
    const LEGACY_LINK: &str = "Connected to 00:11:22:33:44:55 (on wlan0)
\tSSID: Legacy
\tsignal: -67 [-69, -70] dBm
\tsignal avg: -66 dBm
\trx bitrate: 54.0 MBit/s
\ttx bitrate: 24.0 MBit/s
";
    const IW_DEVICES: &str = "phy#0
\tUnnamed/non-netdev interface
\t\twdev 0x2
\tInterface wlp5s0
\t\tifindex 3
";
    const PROC_WIRELESS: &str = "Inter-| sta-|   Quality        |   Discarded packets               | Missed | WE
 face | tus | link level noise |  nwid  crypt   frag  retry   misc | beacon | 22
wlp5s0: 0000   70.  -40.  -256        0      0      0      0      0        0
  wlan1: 0000    0.  -256.  -256        0      0      0      0      0        0
";

    #[test]
    fn iw_link_parsing() {
        let link = WifiLinkReport::parse_iw_link(HE_160_LINK).unwrap();
        assert_eq!((link.rssi_dbm, link.rx_phy_mbps, link.tx_phy_mbps), (Some(-46.0), Some(2401.9), Some(2401.9)));
        let legacy = WifiLinkReport::parse_iw_link(LEGACY_LINK).unwrap();
        assert_eq!((legacy.rssi_dbm, legacy.rx_phy_mbps, legacy.tx_phy_mbps), (Some(-67.0), Some(54.0), Some(24.0)));
        assert_eq!(WifiLinkReport::parse_iw_link("Not connected.\n"), None);
        assert_eq!(parse_iw_interfaces(IW_DEVICES), vec!["wlp5s0".to_owned()]);
    }

    #[test]
    fn proc_wireless_level() {
        assert_eq!(WifiLinkReport::parse_proc_wireless(PROC_WIRELESS, "wlp5s0").unwrap().rssi_dbm, Some(-40.0));
        assert_eq!(WifiLinkReport::parse_proc_wireless(PROC_WIRELESS, "wlan1"), None);
        assert_eq!(WifiLinkReport::parse_proc_wireless(PROC_WIRELESS, "wlan2"), None);
    }

    #[test]
    fn nmcli_row_fields() {
        let row = NmcliWifiRow::parse("yes:Arcadia:A4\\:CE\\:DA\\:C0\\:83\\:F0:108:99:WPA2:wlp5s0").unwrap();
        assert!(row.active);
        assert_eq!(row.bssid.as_deref(), Some("A4:CE:DA:C0:83:F0"));
        assert_eq!(row.channel, Some(108));
        assert_eq!(row.signal_percent, Some(99.0));
        assert_eq!(row.security.as_deref(), Some("WPA2"));
        assert_eq!(row.device.as_deref(), Some("wlp5s0"));
        assert!(adapter_matches(Some("WLP5S0"), Some(" wlp5s0 ")));
        assert!(adapter_matches(None, Some("")));
        assert!(!adapter_matches(None, Some("wlp5s0")));
    }

    #[test]
    fn ping_latency_parsing() {
        assert_eq!(parse_ping_latency_ms("64 bytes from 1.1.1.1: icmp_seq=1 ttl=57 time=12.4 ms"), Some(12.4));
        assert_eq!(parse_ping_latency_ms("64 bytes from 127.0.0.1: time<0.1 ms"), Some(0.5));
        assert_eq!(parse_ping_latency_ms("no reply"), None);
    }
}
