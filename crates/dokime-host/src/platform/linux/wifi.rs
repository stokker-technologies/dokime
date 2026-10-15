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
// Standard Library Imports
use std::{collections::BTreeMap, fs, ops::RangeInclusive, process::Command, time::Duration};
// Third Party Imports

// Local Imports

/* === Constants === */
const PROC_NET_WIRELESS: &str = "/proc/net/wireless";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(5);
const RSSI_RANGE_DBM: RangeInclusive<f64> = -120.0..=0.0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[cfg(not(windows))]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NmcliWifiRow {
    pub(crate) active: bool,
    pub(crate) ssid: Option<String>,
    pub(crate) bssid: Option<String>,
    pub(crate) channel: Option<u32>,
    pub(crate) signal_percent: Option<f64>,
    pub(crate) security: Option<String>,
    pub(crate) device: Option<String>,
}

/// The current association of one wireless interface as the kernel reports it:
/// the received signal and the PHY rates the link negotiated, not the AP's
/// advertised maximum.
#[cfg(not(windows))]
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct WirelessLink {
    pub(crate) rssi_dbm: Option<f64>,
    pub(crate) rx_phy_mbps: Option<f64>,
    pub(crate) tx_phy_mbps: Option<f64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

#[cfg(not(windows))]
pub(crate) fn nmcli_wifi_rows(rescan: bool) -> Result<Vec<NmcliWifiRow>, String> {
    let mut command = Command::new("nmcli");
    command.args([
        "-t",
        "-f",
        "ACTIVE,SSID,BSSID,CHAN,SIGNAL,SECURITY,DEVICE",
        "dev",
        "wifi",
        "list",
        "--rescan",
        if rescan { "yes" } else { "no" },
    ]);
    let output = crate::collectors::run_command_bounded(command, Duration::from_secs(30))
        .map_err(|error| format!("nmcli could not be started: {error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("nmcli output was not UTF-8: {error}"))?;
    Ok(stdout.lines().filter_map(parse_nmcli_wifi_row).collect())
}

/// Reads the link of `interface` from `iw`, or only its RSSI from
/// `/proc/net/wireless` when `iw` is unavailable. `None` when the interface is
/// not associated or neither source knows it.
#[cfg(not(windows))]
pub(crate) fn wireless_link(interface: &str) -> Option<WirelessLink> {
    match iw_links() {
        Ok(mut links) => links.remove(interface),
        Err(_) => {
            let wireless = fs::read_to_string(PROC_NET_WIRELESS).ok()?;
            let rssi_dbm = parse_proc_wireless_level_dbm(&wireless, interface)?;
            Some(WirelessLink {
                rssi_dbm: Some(rssi_dbm),
                ..WirelessLink::default()
            })
        }
    }
}

#[cfg(not(windows))]
pub(crate) fn adapter_matches(device: Option<&str>, filter: Option<&str>) -> bool {
    let Some(filter) = filter.map(str::trim).filter(|filter| !filter.is_empty()) else {
        return true;
    };
    device.is_some_and(|device| device.eq_ignore_ascii_case(filter))
}

/* === Private Functions === */
/// Links of every associated wireless interface `iw dev` lists, by interface.
#[cfg(not(windows))]
fn iw_links() -> Result<BTreeMap<String, WirelessLink>, String> {
    let devices = iw_output(&["dev"])?;

    let mut links = BTreeMap::new();
    for interface in parse_iw_interfaces(&devices) {
        let Ok(link) = iw_output(&["dev", &interface, "link"]) else {
            continue;
        };
        if let Some(link) = parse_iw_link(&link) {
            links.insert(interface, link);
        }
    }

    Ok(links)
}

#[cfg(not(windows))]
fn iw_output(args: &[&str]) -> Result<String, String> {
    let mut command = Command::new("iw");
    command.args(args);
    let output = crate::collectors::run_command_bounded(command, COMMAND_TIMEOUT)
        .map_err(|error| format!("iw could not be started: {error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }

    String::from_utf8(output.stdout).map_err(|error| format!("iw output was not UTF-8: {error}"))
}

#[cfg(not(windows))]
fn parse_iw_interfaces(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| line.trim().strip_prefix("Interface "))
        .map(|interface| interface.trim().to_owned())
        .filter(|interface| !interface.is_empty())
        .collect()
}

/// Parses `iw dev <interface> link`; `None` when the interface is not connected.
#[cfg(not(windows))]
fn parse_iw_link(output: &str) -> Option<WirelessLink> {
    if !output.trim_start().starts_with("Connected to") {
        return None;
    }

    let mut link = WirelessLink::default();
    for line in output.lines() {
        let Some((key, value)) = line.trim().split_once(':') else {
            continue;
        };
        let value = value
            .split_whitespace()
            .next()
            .and_then(|value| value.parse::<f64>().ok());
        match key {
            "signal" => link.rssi_dbm = value.filter(|rssi| RSSI_RANGE_DBM.contains(rssi)),
            "rx bitrate" => link.rx_phy_mbps = value,
            "tx bitrate" => link.tx_phy_mbps = value,
            _ => {}
        }
    }

    Some(link)
}

/// The `level` column of `interface` in `/proc/net/wireless`, in dBm.
#[cfg(not(windows))]
fn parse_proc_wireless_level_dbm(output: &str, interface: &str) -> Option<f64> {
    let fields = output.lines().find_map(|line| {
        let (name, fields) = line.trim().split_once(':')?;
        (name.trim() == interface).then_some(fields)
    })?;
    let level = fields.split_whitespace().nth(2)?;
    let level = level.trim_end_matches('.').parse::<f64>().ok()?;

    RSSI_RANGE_DBM.contains(&level).then_some(level)
}

#[cfg(not(windows))]
fn parse_nmcli_wifi_row(line: &str) -> Option<NmcliWifiRow> {
    let fields = split_nmcli_fields(line);
    if fields.len() < 7 {
        return None;
    }
    Some(NmcliWifiRow {
        active: fields[0].eq_ignore_ascii_case("yes"),
        ssid: non_empty(fields[1].clone()),
        bssid: non_empty(fields[2].clone()),
        channel: fields[3].parse().ok(),
        signal_percent: fields[4].parse().ok(),
        security: non_empty(fields[5].clone()),
        device: non_empty(fields[6].clone()),
    })
}

#[cfg(not(windows))]
fn split_nmcli_fields(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut escaped = false;
    for character in line.chars() {
        if escaped {
            field.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == ':' {
            fields.push(field);
            field = String::new();
        } else {
            field.push(character);
        }
    }
    fields.push(field);
    fields
}

#[cfg(not(windows))]
fn non_empty(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

/* === Tests === */
#[cfg(all(test, not(windows)))]
mod tests {
    use super::*;

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

    const PROC_WIRELESS: &str =
        "Inter-| sta-|   Quality        |   Discarded packets               | Missed | WE
 face | tus | link level noise |  nwid  crypt   frag  retry   misc | beacon | 22
wlp5s0: 0000   70.  -40.  -256        0      0      0      0      0        0
  wlan1: 0000    0.  -256.  -256        0      0      0      0      0        0
";

    #[test]
    fn iw_link_parsing() {
        // HE 160 MHz Link Captured From A Live Association
        assert_eq!(
            parse_iw_link(HE_160_LINK),
            Some(WirelessLink {
                rssi_dbm: Some(-46.0),
                rx_phy_mbps: Some(2401.9),
                tx_phy_mbps: Some(2401.9),
            })
        );

        // Legacy Rates With Per-Chain Signal And A Signal Average
        assert_eq!(
            parse_iw_link(LEGACY_LINK),
            Some(WirelessLink {
                rssi_dbm: Some(-67.0),
                rx_phy_mbps: Some(54.0),
                tx_phy_mbps: Some(24.0),
            })
        );

        // Disconnected Interface And Non-Netdev Devices
        assert_eq!(parse_iw_link("Not connected.\n"), None);
        assert_eq!(parse_iw_interfaces(IW_DEVICES), vec!["wlp5s0".to_owned()]);
    }

    #[test]
    fn proc_wireless_level() {
        assert_eq!(
            parse_proc_wireless_level_dbm(PROC_WIRELESS, "wlp5s0"),
            Some(-40.0)
        );
        assert_eq!(parse_proc_wireless_level_dbm(PROC_WIRELESS, "wlan1"), None);
        assert_eq!(parse_proc_wireless_level_dbm(PROC_WIRELESS, "wlan2"), None);
    }

    #[test]
    fn nmcli_row_fields() {
        let row =
            parse_nmcli_wifi_row("yes:Arcadia:A4\\:CE\\:DA\\:C0\\:83\\:F0:108:99:WPA2:wlp5s0")
                .expect("row");

        assert!(row.active);
        assert_eq!(row.bssid.as_deref(), Some("A4:CE:DA:C0:83:F0"));
        assert_eq!(row.channel, Some(108));
        assert_eq!(row.signal_percent, Some(99.0));
        assert_eq!(row.security.as_deref(), Some("WPA2"));
        assert_eq!(row.device.as_deref(), Some("wlp5s0"));
    }
}
