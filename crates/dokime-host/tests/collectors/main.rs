/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
#[cfg(target_os = "linux")]
use dokime_contract::core::Value;
#[cfg(windows)]
use dokime_host::collectors::wifi::WifiLink;
#[cfg(target_os = "linux")]
use dokime_host::collectors::{
    bluetooth::BluetoothCollector,
    wifi::{NmcliWifiRow, WifiLinkReport, parse_iw_interfaces},
};
#[cfg(target_os = "linux")]
use std::{
    os::unix::fs::PermissionsExt,
    process::Command,
    time::{Duration, Instant},
};

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */
#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
const LEGACY_LINK: &str = "Connected to 00:11:22:33:44:55 (on wlan0)
\tSSID: Legacy
\tfreq: 2437
\tsignal: -67 [-69, -70] dBm
\tsignal avg: -66 dBm
\trx bitrate: 54.0 MBit/s
\ttx bitrate: 24.0 MBit/s
";

#[cfg(target_os = "linux")]
const IW_DEVICES: &str = "phy#0
\tUnnamed/non-netdev interface
\t\twdev 0x2
\t\ttype P2P-device
\tInterface wlp5s0
\t\tifindex 3
\t\ttype managed
";

#[cfg(target_os = "linux")]
const PROC_WIRELESS: &str =
    "Inter-| sta-|   Quality        |   Discarded packets               | Missed | WE
 face | tus | link level noise |  nwid  crypt   frag  retry   misc | beacon | 22
wlp5s0: 0000   70.  -40.  -256        0      0      0      0      0        0
  wlan1: 0000    0.  -256.  -256        0      0      0      0      0        0
";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(target_os = "linux")]
pub(crate) fn public_probe_child() {
    let Ok(mode) = std::env::var("DOKIME_PROBE_CASE") else {
        for mode in ["success", "failure", "timeout"] {
            let output = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "domain_contract", "--nocapture"])
                .env("DOKIME_PROBE_CASE", mode)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{mode}: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        return;
    };
    let root = tempfile::tempdir().unwrap();
    let executable = root.path().join("bluetoothctl");
    std::fs::write(
        &executable,
        r#"#!/usr/bin/env python3
import os, sys, time
from pathlib import Path
if sys.argv[1] != "show":
    sys.exit(0)
Path(os.environ["DOKIME_PROBE_PID"]).write_text(str(os.getpid()))
if os.environ["DOKIME_PROBE_CASE"] == "timeout":
    time.sleep(30)
sys.stdout.write("o" * (512 * 1024) + "\nController fixture\nPowered: yes\n")
sys.stdout.flush()
sys.stderr.write("e" * (512 * 1024))
sys.stderr.flush()
sys.exit(7 if os.environ["DOKIME_PROBE_CASE"] == "failure" else 0)
"#,
    )
    .unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
    let path = std::env::join_paths(
        std::iter::once(root.path().to_owned())
            .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    // This exact test runs alone in the child; all environment changes precede
    // the collector's worker thread and do not affect the parent test process.
    unsafe {
        std::env::set_var("PATH", path);
        std::env::set_var("DOKIME_PROBE_PID", root.path().join("pid"));
    }
    let start = Instant::now();
    let sample = BluetoothCollector::default().collect().unwrap();
    match mode.as_str() {
        "success" => {
            assert_eq!(sample.values["probe_status"], Value::from("ok"));
            assert_eq!(sample.values["bluetooth_adapter"], Value::from("fixture"));
        }
        "failure" => {
            assert_eq!(sample.values["probe_status"], Value::from("error"));
            assert_eq!(
                sample.values["probe_error"],
                Value::from("e".repeat(512 * 1024))
            );
        }
        "timeout" => {
            assert_eq!(sample.values["probe_status"], Value::from("error"));
            assert_eq!(
                sample.values["probe_error"],
                Value::from("bluetoothctl could not be started: command timed out")
            );
            assert!(start.elapsed() < Duration::from_secs(10));
        }
        _ => panic!("unknown probe case"),
    }
    let pid = std::fs::read_to_string(root.path().join("pid")).unwrap();
    assert!(
        !std::path::Path::new("/proc").join(pid).exists(),
        "probe must be reaped before collection returns"
    );
}

#[cfg(target_os = "linux")]
pub(crate) fn iw_link_parsing() {
    // HE 160 MHz Link Captured From A Live Association
    assert_eq!(
        WifiLinkReport::parse_iw_link(HE_160_LINK),
        Some(WifiLinkReport {
            rssi_dbm: Some(-46.0),
            rx_phy_mbps: Some(2401.9),
            tx_phy_mbps: Some(2401.9),
        })
    );

    // Legacy Rates With Per-Chain Signal And A Signal Average
    assert_eq!(
        WifiLinkReport::parse_iw_link(LEGACY_LINK),
        Some(WifiLinkReport {
            rssi_dbm: Some(-67.0),
            rx_phy_mbps: Some(54.0),
            tx_phy_mbps: Some(24.0),
        })
    );

    // Disconnected Interface And Non-Netdev Devices
    assert_eq!(WifiLinkReport::parse_iw_link("Not connected.\n"), None);
    assert_eq!(parse_iw_interfaces(IW_DEVICES), vec!["wlp5s0".to_owned()]);
}

#[cfg(target_os = "linux")]
pub(crate) fn proc_wireless_level() {
    let link = WifiLinkReport::parse_proc_wireless(PROC_WIRELESS, "wlp5s0").expect("link");

    assert_eq!(link.rssi_dbm, Some(-40.0));
    assert_eq!(link.rx_phy_mbps, None);
    assert_eq!(
        WifiLinkReport::parse_proc_wireless(PROC_WIRELESS, "wlan1"),
        None
    );
    assert_eq!(
        WifiLinkReport::parse_proc_wireless(PROC_WIRELESS, "wlan2"),
        None
    );
}

#[cfg(target_os = "linux")]
pub(crate) fn nmcli_row_fields() {
    let row = NmcliWifiRow::parse("yes:Arcadia:A4\\:CE\\:DA\\:C0\\:83\\:F0:108:99:WPA2:wlp5s0")
        .expect("row");

    assert!(row.active);
    assert_eq!(row.bssid.as_deref(), Some("A4:CE:DA:C0:83:F0"));
    assert_eq!(row.channel, Some(108));
    assert_eq!(row.signal_percent, Some(99.0));
    assert_eq!(row.security.as_deref(), Some("WPA2"));
    assert_eq!(row.device.as_deref(), Some("wlp5s0"));
}

#[cfg(windows)]
pub(crate) fn rssi_long_conversion() {
    assert_eq!(
        WifiLink::rssi_dbm_from_query(&(-46_i32).to_ne_bytes()),
        Some(-46.0)
    );
    assert_eq!(WifiLink::rssi_dbm_from_query(&[0xd2, 0xff]), None);
    assert_eq!(WifiLink::rssi_dbm_from_query(&12_i32.to_ne_bytes()), None);
    assert_eq!(
        WifiLink::rssi_dbm_from_query(&(-256_i32).to_ne_bytes()),
        None
    );
}

#[test]
fn domain_contract() {
    #[cfg(target_os = "linux")]
    {
        eprintln!("Scenario: public_probe_child");
        public_probe_child();
    }
    #[cfg(target_os = "linux")]
    {
        eprintln!("Scenario: iw_link_parsing");
        iw_link_parsing();
    }
    #[cfg(target_os = "linux")]
    {
        eprintln!("Scenario: proc_wireless_level");
        proc_wireless_level();
    }
    #[cfg(target_os = "linux")]
    {
        eprintln!("Scenario: nmcli_row_fields");
        nmcli_row_fields();
    }
    #[cfg(windows)]
    {
        eprintln!("Scenario: rssi_long_conversion");
        rssi_long_conversion();
    }
}
