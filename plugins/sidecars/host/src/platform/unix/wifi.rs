/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{collections::BTreeMap, fs, ops::RangeInclusive, time::Duration};

const PROC_NET_WIRELESS: &str = "/proc/net/wireless";
const RSSI_RANGE_DBM: RangeInclusive<f64> = -120.0..=0.0;

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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WifiLinkReport {
    pub rssi_dbm: Option<f64>,
    pub rx_phy_mbps: Option<f64>,
    pub tx_phy_mbps: Option<f64>,
}

impl NmcliWifiRow {
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

impl WifiLinkReport {
    pub fn parse_iw_link(output: &str) -> Option<Self> {
        if !output.trim_start().starts_with("Connected to") {
            return None;
        }

        let mut link = Self::default();
        for line in output.lines() {
            let Some((key, value)) = line.trim().split_once(':') else {
                continue;
            };
            let value = value.split_whitespace().next().and_then(|value| value.parse::<f64>().ok());
            match key {
                "signal" => link.rssi_dbm = value.filter(|rssi| RSSI_RANGE_DBM.contains(rssi)),
                "rx bitrate" => link.rx_phy_mbps = value,
                "tx bitrate" => link.tx_phy_mbps = value,
                _ => {}
            }
        }

        Some(link)
    }

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

        Some(Self { rssi_dbm: Some(level), ..Self::default() })
    }
}

pub(crate) fn nmcli_wifi_rows(rescan: bool, timeout: Duration) -> Result<Vec<NmcliWifiRow>, String> {
    let mut command = dokime_domain::process::command("nmcli");
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
    let output =
        crate::collectors::run_command_bounded(command, timeout).map_err(|error| format!("nmcli could not be started: {error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    let stdout = String::from_utf8(output.stdout).map_err(|error| format!("nmcli output was not UTF-8: {error}"))?;
    Ok(stdout.lines().filter_map(NmcliWifiRow::parse).collect())
}

pub(crate) fn wireless_link(interface: &str) -> Option<WifiLinkReport> {
    match iw_links(crate::host::wifi_link::POLICY.attempt_timeout) {
        Ok(mut links) => links.remove(interface),
        Err(_) => {
            let wireless = fs::read_to_string(PROC_NET_WIRELESS).ok()?;
            WifiLinkReport::parse_proc_wireless(&wireless, interface)
        }
    }
}

pub fn parse_iw_interfaces(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| line.trim().strip_prefix("Interface "))
        .map(|interface| interface.trim().to_owned())
        .filter(|interface| !interface.is_empty())
        .collect()
}

pub(crate) fn adapter_matches(device: Option<&str>, filter: Option<&str>) -> bool {
    let Some(filter) = filter.map(str::trim).filter(|filter| !filter.is_empty()) else {
        return true;
    };
    device.is_some_and(|device| device.eq_ignore_ascii_case(filter))
}

fn iw_links(timeout: Duration) -> Result<BTreeMap<String, WifiLinkReport>, String> {
    let devices = iw_output(&["dev"], timeout)?;

    let mut links = BTreeMap::new();
    for interface in parse_iw_interfaces(&devices) {
        let Ok(link) = iw_output(&["dev", &interface, "link"], timeout) else {
            continue;
        };
        if let Some(link) = WifiLinkReport::parse_iw_link(&link) {
            links.insert(interface, link);
        }
    }

    Ok(links)
}

fn iw_output(args: &[&str], timeout: Duration) -> Result<String, String> {
    let mut command = dokime_domain::process::command("iw");
    command.args(args);
    let output = crate::collectors::run_command_bounded(command, timeout).map_err(|error| format!("iw could not be started: {error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }

    String::from_utf8(output.stdout).map_err(|error| format!("iw output was not UTF-8: {error}"))
}

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

fn non_empty(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}
