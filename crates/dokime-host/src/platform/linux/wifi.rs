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
/// One network of `nmcli -t dev wifi list`, the source of SSID, channel,
/// security and the scan on Linux.
#[cfg(not(windows))]
#[derive(Clone, Debug, PartialEq)]
pub struct NmcliWifiRow {
    pub active: bool,
    pub ssid: Option<String>,
    pub bssid: Option<String>,
    pub channel: Option<u32>,
    pub signal_percent: Option<f64>,
    pub security: Option<String>,
    pub device: Option<String>,
}

/// The current association of one wireless interface as the kernel reports it:
/// the received signal and the PHY rates the link negotiated, not the AP's
/// advertised maximum.
#[cfg(not(windows))]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WifiLinkReport {
    pub rssi_dbm: Option<f64>,
    pub rx_phy_mbps: Option<f64>,
    pub tx_phy_mbps: Option<f64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[cfg(not(windows))]
impl NmcliWifiRow {
    /// Parses one terse line of the `ACTIVE,SSID,BSSID,CHAN,SIGNAL,SECURITY,DEVICE` fields.
    pub fn parse(line: &str) -> Option<Self> {
        let fields = split_nmcli_fields(line);
        if fields.len() < 7 {
            return None;
        }

        Some(Self {
            active: fields[0].eq_ignore_ascii_case("yes"),
            ssid: non_empty(fields[1].clone()),
            bssid: non_empty(fields[2].clone()),
            channel: fields[3].parse().ok(),
            signal_percent: fields[4].parse().ok(),
            security: non_empty(fields[5].clone()),
            device: non_empty(fields[6].clone()),
        })
    }
}

#[cfg(not(windows))]
impl WifiLinkReport {
    /// Parses `iw dev <interface> link`; `None` when the interface is not connected.
    pub fn parse_iw_link(output: &str) -> Option<Self> {
        if !output.trim_start().starts_with("Connected to") {
            return None;
        }

        let mut link = Self::default();
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

    /// Reads the `level` column of `interface` from `/proc/net/wireless` as the
    /// RSSI in dBm; the wireless extensions carry no PHY rates.
    pub fn parse_proc_wireless(output: &str, interface: &str) -> Option<Self> {
        let fields = output.lines().find_map(|line| {
            let (name, fields) = line.trim().split_once(':')?;
            (name.trim() == interface).then_some(fields)
        })?;
        let level = fields.split_whitespace().nth(2)?;
        let level = level.trim_end_matches('.').parse::<f64>().ok()?;
        if !RSSI_RANGE_DBM.contains(&level) {
            return None;
        }

        Some(Self {
            rssi_dbm: Some(level),
            ..Self::default()
        })
    }
}

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
    Ok(stdout.lines().filter_map(NmcliWifiRow::parse).collect())
}

/// Reads the link of `interface` from `iw`, or only its RSSI from
/// `/proc/net/wireless` when `iw` is unavailable. `None` when the interface is
/// not associated or neither source knows it.
#[cfg(not(windows))]
pub(crate) fn wireless_link(interface: &str) -> Option<WifiLinkReport> {
    match iw_links() {
        Ok(mut links) => links.remove(interface),
        Err(_) => {
            let wireless = fs::read_to_string(PROC_NET_WIRELESS).ok()?;
            WifiLinkReport::parse_proc_wireless(&wireless, interface)
        }
    }
}

/// The network interfaces `iw dev` lists, skipping non-netdev devices such as
/// a P2P device.
#[cfg(not(windows))]
pub fn parse_iw_interfaces(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| line.trim().strip_prefix("Interface "))
        .map(|interface| interface.trim().to_owned())
        .filter(|interface| !interface.is_empty())
        .collect()
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
fn iw_links() -> Result<BTreeMap<String, WifiLinkReport>, String> {
    let devices = iw_output(&["dev"])?;

    let mut links = BTreeMap::new();
    for interface in parse_iw_interfaces(&devices) {
        let Ok(link) = iw_output(&["dev", &interface, "link"]) else {
            continue;
        };
        if let Some(link) = WifiLinkReport::parse_iw_link(&link) {
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
