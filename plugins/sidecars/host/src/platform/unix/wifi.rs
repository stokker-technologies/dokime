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
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::Duration;

// Third Party Imports
use tokio::io::{AsyncRead, AsyncReadExt};

// Local Imports

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
        if fields.len() < 7 {
            return None;
        }
        Some(Self {
            active: fields[0].eq_ignore_ascii_case("yes"),
            ssid: non_empty(&fields[1]),
            bssid: non_empty(&fields[2]),
            channel: fields[3].parse().ok(),
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

/// NetworkManager's Wi-Fi List; a Rescan Only When Requested.
pub fn nmcli_wifi_rows(rescan: bool, timeout: Duration) -> Result<Vec<NmcliWifiRow>, String> {
    let mut command = command("nmcli");
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
    let output = run_bounded(command, timeout).map_err(|error| format!("Start nmcli: {error}; Install NetworkManager"))?;
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

/// A Child Process Command; Unix Opens No Console Window.
pub fn command(program: &str) -> Command {
    #[allow(clippy::disallowed_methods, reason = "Unix child processes open no console window")]
    Command::new(program)
}

/// Run a Command to Completion Within the Timeout, Killing It When the Timeout Elapses.
pub fn run_bounded(command: Command, timeout: Duration) -> std::io::Result<Output> {
    // Collectors run on the blocking pool; keep the runtime on its own joined thread so block_on never nests runtimes.
    thread::Builder::new()
        .name("host-probe".to_owned())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
            runtime.block_on(async move {
                let mut command = tokio::process::Command::from(command);
                command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
                let mut child = command.spawn()?;
                let stdout = child.stdout.take().ok_or_else(|| std::io::Error::other("Capture Probe Output: stdout is not piped"))?;
                let stderr = child.stderr.take().ok_or_else(|| std::io::Error::other("Capture Probe Output: stderr is not piped"))?;
                let result = capture(stdout, stderr, child.wait(), timeout).await;
                if result.is_err() {
                    // This also waits for the direct child, avoiding zombies; the cancelled pipe readers cannot strand threads.
                    let _ = child.kill().await;
                }
                result
            })
        })?
        .join()
        .map_err(|_| std::io::Error::other("Run Host Probe: the probe worker panicked"))?
}

/* === Private Functions === */

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
    let output = run_bounded(command, timeout).map_err(|error| format!("Start iw: {error}; Install iw"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    String::from_utf8(output.stdout).map_err(|error| format!("Read iw Output: {error}"))
}

async fn capture(
    mut stdout: impl AsyncRead + Unpin, mut stderr: impl AsyncRead + Unpin,
    wait: impl std::future::Future<Output = std::io::Result<std::process::ExitStatus>>, timeout: Duration,
) -> std::io::Result<Output> {
    let capture = async {
        let mut stdout_bytes = Vec::new();
        let mut stderr_bytes = Vec::new();
        let (_, _, status) = tokio::try_join!(stdout.read_to_end(&mut stdout_bytes), stderr.read_to_end(&mut stderr_bytes), wait)?;
        Ok(Output {
            status,
            stdout: stdout_bytes,
            stderr: stderr_bytes,
        })
    };
    tokio::time::timeout(timeout, capture)
        .await
        .unwrap_or_else(|_| Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "Run Host Probe: the command timed out")))
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
