/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

use dokime_host::collectors::wifi::{NmcliWifiRow, WifiLinkReport, parse_iw_interfaces};

/* === Constants === */

const HE_160_LINK: &str = "Connected to a4:ce:da:c0:83:f0 (on wlp5s0)
\tSSID: Arcadia
\tfreq: 5540.0
\tRX: 2942406830 bytes (325203201 packets)
\tTX: 577323037 bytes (99233661 packets)
\tsignal: -46 dBm
\trx bitrate: 2401.9 MBit/s 160MHz HE-MCS 11 HE-NSS 2 HE-GI 0 HE-DCM 0
\ttx bitrate: 2401.9 MBit/s 160MHz HE-MCS 11 HE-NSS 2 HE-GI 0 HE-DCM 0
\tbss flags: short-slot-time
\tdtim period: 3
\tbeacon int: 100
";

const LEGACY_LINK: &str = "Connected to 00:11:22:33:44:55 (on wlan0)
\tSSID: Legacy
\tfreq: 2437
\tsignal: -67 [-69, -70] dBm
\tsignal avg: -66 dBm
\trx bitrate: 54.0 MBit/s
\ttx bitrate: 24.0 MBit/s
";

const IW_DEVICES: &str = "phy#0
\tUnnamed/non-netdev interface
\t\twdev 0x2
\t\ttype P2P-device
\tInterface wlp5s0
\t\tifindex 3
\t\ttype managed
";

const PROC_WIRELESS: &str = "Inter-| sta-|   Quality        |   Discarded packets               | Missed | WE
 face | tus | link level noise |  nwid  crypt   frag  retry   misc | beacon | 22
wlp5s0: 0000   70.  -40.  -256        0      0      0      0      0        0
  wlan1: 0000    0.  -256.  -256        0      0      0      0      0        0
";

/* === Tests === */

#[test]
fn iw_link_parsing() {
    // HE 160 MHz Link Captured From A Live Association
    assert_eq!(
        WifiLinkReport::parse_iw_link(HE_160_LINK),
        Some(WifiLinkReport {
            rssi_dbm: Some(-46.0),
            rx_phy_mbps: Some(2401.9),
            tx_phy_mbps: Some(2401.9)
        })
    );

    // Legacy Rates With Per-Chain Signal And A Signal Average
    assert_eq!(
        WifiLinkReport::parse_iw_link(LEGACY_LINK),
        Some(WifiLinkReport {
            rssi_dbm: Some(-67.0),
            rx_phy_mbps: Some(54.0),
            tx_phy_mbps: Some(24.0)
        })
    );

    // Disconnected Interface And Non-Netdev Devices
    assert_eq!(WifiLinkReport::parse_iw_link("Not connected.\n"), None);
    assert_eq!(parse_iw_interfaces(IW_DEVICES), vec!["wlp5s0".to_owned()]);
}

#[test]
fn proc_wireless_level() {
    let link = WifiLinkReport::parse_proc_wireless(PROC_WIRELESS, "wlp5s0").expect("link");

    assert_eq!(link.rssi_dbm, Some(-40.0));
    assert_eq!(link.rx_phy_mbps, None);
    assert_eq!(WifiLinkReport::parse_proc_wireless(PROC_WIRELESS, "wlan1"), None);
    assert_eq!(WifiLinkReport::parse_proc_wireless(PROC_WIRELESS, "wlan2"), None);
}

#[test]
fn nmcli_row_fields() {
    let row = NmcliWifiRow::parse("yes:Arcadia:A4\\:CE\\:DA\\:C0\\:83\\:F0:108:99:WPA2:wlp5s0").expect("row");

    assert!(row.active);
    assert_eq!(row.bssid.as_deref(), Some("A4:CE:DA:C0:83:F0"));
    assert_eq!(row.channel, Some(108));
    assert_eq!(row.signal_percent, Some(99.0));
    assert_eq!(row.security.as_deref(), Some("WPA2"));
    assert_eq!(row.device.as_deref(), Some("wlp5s0"));
}
