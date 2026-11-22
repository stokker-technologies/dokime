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
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

// Third Party Imports

// Local Imports
use dokime::encoding::Message;
use plugin_host::collectors::wifi::{
    CONGESTION_MODEL_24_GHZ, CONGESTION_MODEL_SAME_CHANNEL, ChannelPlan, Counters, ScanReport, band, channel_from_frequency, congestion,
    latency_row, scan_due, scan_readings, scan_rows, throughput,
};
use plugin_host::domain::telemetry::{WifiChannels, WifiCongestion};
use plugin_host::platform::{ScannedNetwork, WifiBand, WifiCapabilities};

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
fn wifi_channels_include_the_full_empty_fallback_plan_once_per_scan() {
    let mut plan = ChannelPlan::default();
    for sample_index in [1, 2] {
        let rows = plan.rows(sample_index, &ScanReport::default());
        assert_eq!(rows.len(), 41);
        let cells = rows.iter().map(|row| (&row.wifi_band, row.wifi_channel)).collect::<BTreeSet<_>>();
        assert_eq!(cells.len(), rows.len());
        assert!(rows.iter().all(|row| row.sample_index == sample_index as i64 && row.network_count == 0 && row.occupancy == 0.0));
        let channels = |band| rows.iter().filter(|row| row.wifi_band == band).map(|row| row.wifi_channel).collect::<Vec<_>>();
        assert_eq!(channels("2.4GHz"), (1..=13).collect::<Vec<_>>());
        assert_eq!(
            channels("5GHz"),
            vec![
                36, 40, 44, 48, 52, 56, 60, 64, 100, 104, 108, 112, 116, 120, 124, 128, 132, 136, 140, 144, 149, 153, 157, 161, 165, 169,
                173, 177
            ]
        );
        assert!(channels("6GHz").is_empty());
    }
}

#[test]
fn wifi_channels_use_explicit_adapter_bands_and_regulatory_channel_14() {
    let mut plan = ChannelPlan::default();
    let report = ScanReport {
        capabilities: Some(WifiCapabilities {
            bands: vec![WifiBand::Ghz24, WifiBand::Ghz6, WifiBand::Ghz6],
            channel_14_allowed: true,
        }),
        ..ScanReport::default()
    };
    let rows = plan.rows(3, &report);
    assert_eq!(rows.len(), 73);
    assert!(rows.iter().all(|row| row.wifi_band != "5GHz" && row.occupancy == 0.0));
    assert!(rows.iter().any(|row| row.wifi_band == "2.4GHz" && row.wifi_channel == 14));
    assert_eq!(
        rows.iter().filter(|row| row.wifi_band == "6GHz").map(|row| row.wifi_channel).collect::<Vec<_>>(),
        (1..=233).step_by(4).collect::<Vec<_>>()
    );
    let unsupported = ScanReport {
        capabilities: Some(WifiCapabilities::default()),
        ..ScanReport::default()
    };
    assert!(plan.rows(4, &unsupported).is_empty());
}

#[test]
fn wifi_channels_keep_observed_6ghz_until_collection_stop() {
    let mut plan = ChannelPlan::default();
    let report = ScanReport {
        networks: vec![network("a", 1_005, false)],
        ..ScanReport::default()
    };
    let observed = plan.rows(1, &report);
    assert_eq!(observed.len(), 100);
    let row = observed.iter().find(|row| row.wifi_band == "6GHz" && row.wifi_channel == 5).unwrap();
    assert_eq!((row.network_count, row.occupancy), (1, 1.0));
    assert_eq!(plan.rows(2, &ScanReport::default()).len(), 100);
    assert_eq!(ChannelPlan::default().rows(3, &ScanReport::default()).len(), 41);
    let known = ScanReport {
        capabilities: Some(WifiCapabilities {
            bands: vec![WifiBand::Ghz5],
            channel_14_allowed: false,
        }),
        ..report
    };
    assert_eq!(plan.rows(4, &known).len(), 28);
}

#[test]
fn wifi_channels_share_adjacent_overlap_without_scaling_or_clamping() {
    let report = ScanReport {
        networks: vec![network("a", 6, true), network("b", 6, false), network("c", 8, false), network("d", 36, false)],
        active: true,
        ..ScanReport::default()
    };
    let rows = ChannelPlan::default().rows(7, &report);
    let row = |band, channel| rows.iter().find(|row| row.wifi_band == band && row.wifi_channel == channel).unwrap();
    assert_eq!(row("2.4GHz", 6).network_count, 2);
    assert!((row("2.4GHz", 6).occupancy - 2.6).abs() < 1e-9);
    assert!((row("2.4GHz", 7).occupancy - 2.4).abs() < 1e-9);
    assert_eq!(row("2.4GHz", 1).occupancy, 0.0);
    assert_eq!(row("5GHz", 36).occupancy, 1.0);
    assert_eq!(row("5GHz", 40).occupancy, 0.0);
    let network_rows = scan_rows(7, Ok(report), WEIGHT);
    assert_eq!(network_rows.len(), 4);
    assert_eq!(network_rows[0].sample_index, Some(row("2.4GHz", 6).sample_index));
    assert_eq!(network_rows[0].network_count, Some(row("2.4GHz", 6).network_count));
    assert!((network_rows[0].wifi_network_congestion_score.unwrap() / WEIGHT - row("2.4GHz", 6).occupancy).abs() < 1e-9);
    let encoded = row("2.4GHz", 6).encode_to_vec();
    assert_eq!(WifiChannels::decode(encoded.as_slice()).unwrap(), *row("2.4GHz", 6));
}

#[test]
fn wifi_channels_count_bonded_spans_without_changing_primary_counts() {
    let mut five = network("a", 36, false);
    five.channel_width_mhz = Some(80);
    five.center_channel = Some(42);
    let mut six = network("b", 1_001, false);
    six.channel_width_mhz = Some(160);
    six.center_channel = Some(1_015);
    let report = ScanReport {
        networks: vec![five, six],
        ..ScanReport::default()
    };
    let rows = ChannelPlan::default().rows(8, &report);
    for row in rows {
        let expected = match row.wifi_band.as_str() {
            "5GHz" if (36..=48).contains(&row.wifi_channel) => 1.0,
            "6GHz" if (1..=29).contains(&row.wifi_channel) => 1.0,
            _ => 0.0,
        };
        assert_eq!(row.occupancy, expected, "{} Channel {}", row.wifi_band, row.wifi_channel);
        let primary = (row.wifi_band == "5GHz" && row.wifi_channel == 36) || (row.wifi_band == "6GHz" && row.wifi_channel == 1);
        assert_eq!(row.network_count, i64::from(primary));
    }
}

#[test]
fn platform_frequencies_keep_overlapping_band_channel_numbers_distinct() {
    assert_eq!(channel_from_frequency(2_412), Some(1));
    assert_eq!(channel_from_frequency(2_484_000), Some(14));
    assert_eq!(channel_from_frequency(5_180_000), Some(36));
    assert_eq!(channel_from_frequency(5_955), Some(1_001));
    assert_eq!(channel_from_frequency(7_115), Some(1_233));
    assert_eq!(channel_from_frequency(60_000_000), None);
}

#[test]
fn wifi_channels_and_networks_are_projections_of_one_report_and_failed_scans_have_no_channels() {
    let mut plan = ChannelPlan::default();
    let report = ScanReport {
        networks: vec![network("a", 1_001, false)],
        active: false,
        error: Some("Scan Refused; Enable Wi-Fi Scanning".into()),
        ..ScanReport::default()
    };
    let readings = scan_readings(9, Ok(report), WEIGHT, &mut plan);
    assert_eq!(readings.networks.len(), 1);
    assert_eq!(readings.networks[0].scan_status.as_deref(), Some("cached"));
    assert_eq!(readings.networks[0].sample_index, Some(9));
    assert_eq!(readings.channels.len(), 100);
    assert!(readings.channels.iter().all(|row| row.sample_index == 9));
    let failed = scan_readings(10, Err("Scan Refused; Enable Wi-Fi Scanning".into()), WEIGHT, &mut plan);
    assert!(failed.channels.is_empty());
    assert_eq!(failed.networks[0].scan_status.as_deref(), Some("error"));
    let refused = ScanReport {
        error: Some("Scan Refused; Enable Wi-Fi Scanning".into()),
        ..ScanReport::default()
    };
    assert!(scan_readings(10, Ok(refused), WEIGHT, &mut plan).channels.is_empty());
    assert_eq!(scan_readings(11, Ok(ScanReport::default()), WEIGHT, &mut plan).channels.len(), 100);
}

#[test]
fn scan_congestion_preserves_missing_association_and_hidden_ssid() {
    let mut hidden = network("a", 6, true);
    hidden.ssid = None;
    let scans = [
        Ok(ScanReport {
            networks: vec![hidden, network("b", 6, false)],
            active: true,
            error: None,
            ..ScanReport::default()
        }),
        Ok(ScanReport {
            networks: vec![network("b", 6, false)],
            active: true,
            error: None,
            ..ScanReport::default()
        }),
        Ok(ScanReport::default()),
        Err("scan refused".into()),
    ];
    let congestion = scans
        .into_iter()
        .enumerate()
        .map(|(index, scan)| {
            let readings = scan_rows(index as u64, scan, WEIGHT);
            let row = WifiCongestion::from(readings.into_iter().next().unwrap());
            let bytes = row.encode_to_vec();
            WifiCongestion::decode(bytes.as_slice()).unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(congestion.len(), 4);
    assert_eq!(congestion[0].scan_network_count, Some(2));
    assert_eq!(congestion[0].active_wifi_ssid, None);
    assert_eq!(congestion[0].active_wifi_channel, Some(6));
    assert_eq!(congestion[0].wifi_network_congestion_score, Some(20.0));
    assert_eq!(congestion[1].scan_network_count, Some(1));
    for row in &congestion[1..] {
        assert_eq!(row.active_wifi_channel, None);
        assert_eq!(row.active_wifi_band, None);
        assert_eq!(row.active_wifi_ssid, None);
        assert_eq!(row.wifi_network_congestion_score, None);
        assert_eq!(row.congestion_model, None);
    }
    assert_eq!(congestion[2].scan_network_count, Some(0));
    assert_eq!(congestion[3].scan_network_count, Some(0));
}

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
        ..ScanReport::default()
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
        ..ScanReport::default()
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
    use plugin_host::platform::WifiBand;
    use plugin_host::platform::unix::echo::parse_ping_latency_ms;
    use plugin_host::platform::unix::wifi::{NmcliWifiRow, WifiLinkReport, adapter_matches, parse_iw_capabilities, parse_iw_interfaces};

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
    fn iw_capabilities_keep_unobserved_bands_and_regulatory_channel_14() {
        let report = parse_iw_capabilities("Frequencies:\n * 2412 MHz [1] (20.0 dBm)\n * 2484 MHz [14] (disabled)\n * 5180 MHz [36] (20.0 dBm)\n * 5955 MHz [1] (disabled)\n").unwrap();
        assert_eq!(report.bands, vec![WifiBand::Ghz24, WifiBand::Ghz5, WifiBand::Ghz6]);
        assert!(!report.channel_14_allowed);
        assert!(parse_iw_capabilities("* 2484 MHz [14] (20.0 dBm)").unwrap().channel_14_allowed);
        assert!(parse_iw_capabilities("no frequencies").is_none());
    }

    #[test]
    fn nmcli_frequency_disambiguates_6ghz() {
        let row = NmcliWifiRow::parse("no:Network:A4\\:CE\\:DA\\:C0\\:83\\:F0:5955 MHz:99:WPA3:wlp5s0").unwrap();
        assert_eq!(row.channel, Some(1_001));
    }

    #[test]
    fn nmcli_row_fields() {
        let row = NmcliWifiRow::parse("yes:Arcadia:A4\\:CE\\:DA\\:C0\\:83\\:F0:5540 MHz:99:WPA2:wlp5s0").unwrap();
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
