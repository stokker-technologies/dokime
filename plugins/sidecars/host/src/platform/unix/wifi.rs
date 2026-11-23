/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Wi-Fi Link and Scan Through iw and NetworkManager

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::fs;
use std::ops::RangeInclusive;
use std::time::Duration;

// Third Party Imports

// Local Imports
use crate::collectors::wifi::{LatencyOptions, LinkOptions, ScanOptions, ScanReport, band, channel_from_frequency};
use crate::domain::readings::WifiLink;
use crate::platform::command::{bounded_output, command};
use crate::platform::unix::echo;
use crate::platform::{ScannedNetwork, WifiBand, WifiCapabilities};

/* === Constants === */

const PROC_NET_WIRELESS: &str = "/proc/net/wireless";
const RSSI_RANGE_DBM: RangeInclusive<f64> = -120.0..=0.0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One Row of `nmcli -t dev wifi list`.
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

/// The Kernel Link: Received Signal and Negotiated PHY Rates.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WifiLinkReport {
    pub rssi_dbm: Option<f64>,
    pub rx_phy_mbps: Option<f64>,
    pub tx_phy_mbps: Option<f64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl NmcliWifiRow {
    pub fn parse(line: &str) -> Option<Self> {
        let fields = split_nmcli_fields(line);
        if fields.len() != 7 {
            return None;
        }
        Some(Self {
            active: fields[0].eq_ignore_ascii_case("yes"),
            ssid: non_empty(&fields[1]),
            bssid: non_empty(&fields[2]),
            channel: fields[3].split_whitespace().next().and_then(|value| value.parse().ok()).and_then(channel_from_frequency),
            signal_percent: fields[4].parse().ok(),
            security: non_empty(&fields[5]),
            device: non_empty(&fields[6]),
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

/* === Public Functions === */

pub fn read_link(options: &LinkOptions, row: &mut WifiLink) -> Option<String> {
    let rows = match nmcli_wifi_rows(false, options.probe_timeout) {
        Ok(rows) => rows,
        Err(error) => {
            row.probe_status = Some("error".into());
            row.probe_error = Some(error);
            return None;
        }
    };
    let filter = options.adapter.as_deref();
    let Some(link) = rows.into_iter().find(|candidate| candidate.active && adapter_matches(candidate.device.as_deref(), filter)) else {
        row.probe_status = Some("unavailable".into());
        row.associated = Some(false);
        return None;
    };
    row.probe_status = Some("ok".into());
    row.wifi_state = Some("connected".into());
    row.associated = Some(true);
    row.wifi_adapter = link.device.clone();
    row.wifi_ssid = link.ssid;
    row.wifi_bssid = link.bssid;
    row.wifi_auth = link.security;
    row.wifi_channel = link.channel.map(i64::from);
    row.wifi_band = link.channel.and_then(band);
    row.wifi_signal_percent = link.signal_percent;
    let wireless = link.device.as_deref().and_then(|interface| wireless_link(interface, options.probe_timeout)).unwrap_or_default();
    row.wifi_rssi_dbm = wireless.rssi_dbm;
    row.wifi_rx_phy_mbps = wireless.rx_phy_mbps;
    row.wifi_tx_phy_mbps = wireless.tx_phy_mbps;
    link.device
}

pub fn scan(options: &ScanOptions, active: bool) -> Result<ScanReport, String> {
    let (rows, active, error) = match nmcli_wifi_rows(active, options.probe_timeout) {
        Ok(rows) => (rows, active, None),
        Err(error) if active => (nmcli_wifi_rows(false, options.probe_timeout).map_err(|_| error.clone())?, false, Some(error)),
        Err(error) => return Err(error),
    };
    let filter = options.adapter.as_deref();
    let capabilities = wireless_capabilities(filter, options.probe_timeout);
    let networks = rows
        .into_iter()
        .filter(|row| adapter_matches(row.device.as_deref(), filter))
        .map(|row| ScannedNetwork {
            adapter: row.device,
            ssid: row.ssid,
            bssid: row.bssid,
            signal_percent: row.signal_percent,
            rssi_dbm: None,
            channel: row.channel,
            security: row.security,
            associated: row.active,
            channel_width_mhz: None,
            center_channel: None,
        })
        .collect();
    Ok(ScanReport { networks, capabilities, active, error })
}

pub fn round_trip_ms(options: &LatencyOptions) -> Result<Option<f64>, String> {
    echo::round_trip_ms(&options.host, options.reply_timeout, options.payload_bytes, options.process_timeout)
}

/// NetworkManager's Wi-Fi List; a Rescan Only When Requested.
pub fn nmcli_wifi_rows(rescan: bool, timeout: Duration) -> Result<Vec<NmcliWifiRow>, String> {
    let mut command = command("nmcli");
    command.args([
        "-t",
        "-f",
        "ACTIVE,SSID,BSSID,FREQ,SIGNAL,SECURITY,DEVICE",
        "dev",
        "wifi",
        "list",
        "--rescan",
        if rescan { "yes" } else { "no" },
    ]);
    let output = bounded_output(command, timeout).map_err(|error| format!("Start nmcli: {error}; Install NetworkManager"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    let stdout = String::from_utf8(output.stdout).map_err(|error| format!("Read nmcli Output: {error}"))?;
    Ok(stdout.lines().filter_map(NmcliWifiRow::parse).collect())
}

/// The Kernel Link of One Interface Through iw, Falling Back to /proc/net/wireless.
pub fn wireless_link(interface: &str, timeout: Duration) -> Option<WifiLinkReport> {
    match iw_links(timeout) {
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

/// Whether a Device Matches the Configured Adapter; No Filter Matches Every Device.
pub fn adapter_matches(device: Option<&str>, filter: Option<&str>) -> bool {
    let Some(filter) = filter.map(str::trim).filter(|filter| !filter.is_empty()) else {
        return true;
    };
    device.is_some_and(|device| device.eq_ignore_ascii_case(filter))
}

/// Bands From the Kernel PHY Frequency List, Including Bands With No Nearby Networks.
pub fn parse_iw_capabilities(output: &str) -> Option<WifiCapabilities> {
    let mut capabilities = WifiCapabilities::default();
    for line in output.lines() {
        let mut fields = line.split_whitespace();
        if fields.next() != Some("*") {
            continue;
        }
        let Some(frequency) = fields.next().and_then(|value| value.parse::<u32>().ok()) else {
            continue;
        };
        if fields.next() != Some("MHz") {
            continue;
        }
        let Some(channel) = channel_from_frequency(frequency) else {
            continue;
        };
        let frequency_band = match channel {
            1..=14 => WifiBand::Ghz24,
            1_001..=1_233 => WifiBand::Ghz6,
            _ => WifiBand::Ghz5,
        };
        if !capabilities.bands.contains(&frequency_band) {
            capabilities.bands.push(frequency_band);
        }
        capabilities.channel_14_allowed |= channel == 14 && !line.contains("disabled");
    }
    (!capabilities.bands.is_empty()).then_some(capabilities)
}

/* === Private Functions === */

fn wireless_capabilities(filter: Option<&str>, timeout: Duration) -> Option<WifiCapabilities> {
    let devices = iw_output(&["dev"], timeout).ok()?;
    let mut phy = None;
    let mut selected = Vec::new();
    for line in devices.lines().map(str::trim) {
        if let Some(number) = line.strip_prefix("phy#") {
            phy = Some(format!("phy{number}"));
        }
        if let Some(interface) = line.strip_prefix("Interface ")
            && adapter_matches(Some(interface), filter)
        {
            let phy = phy.as_ref()?;
            if !selected.contains(phy) {
                selected.push(phy.clone());
            }
        }
    }
    let mut capabilities = WifiCapabilities::default();
    for phy in selected {
        let output = iw_output(&["phy", &phy, "info"], timeout).ok()?;
        let reading = parse_iw_capabilities(&output)?;
        capabilities.channel_14_allowed |= reading.channel_14_allowed;
        for band in reading.bands {
            if !capabilities.bands.contains(&band) {
                capabilities.bands.push(band);
            }
        }
    }
    Some(capabilities)
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
    let mut command = command("iw");
    command.args(args);
    let output = bounded_output(command, timeout).map_err(|error| format!("Start iw: {error}; Install iw"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    String::from_utf8(output.stdout).map_err(|error| format!("Read iw Output: {error}"))
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

fn non_empty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

/* === Tests === */
