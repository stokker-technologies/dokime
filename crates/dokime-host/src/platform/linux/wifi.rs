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
use std::{process::Command, time::Duration};
// Third Party Imports

// Local Imports

/* === Constants === */

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
    pub(crate) rate_mbps: Option<f64>,
    pub(crate) security: Option<String>,
    pub(crate) device: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
/* === Functions === */

#[cfg(not(windows))]
pub(crate) fn nmcli_wifi_rows(rescan: bool) -> Result<Vec<NmcliWifiRow>, String> {
    let mut command = Command::new("nmcli");
    command.args([
        "-t",
        "-f",
        "ACTIVE,SSID,BSSID,CHAN,SIGNAL,RATE,SECURITY,DEVICE",
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

#[cfg(not(windows))]
pub(crate) fn adapter_matches(device: Option<&str>, filter: Option<&str>) -> bool {
    let Some(filter) = filter.map(str::trim).filter(|filter| !filter.is_empty()) else {
        return true;
    };
    device.is_some_and(|device| device.eq_ignore_ascii_case(filter))
}

/* === Private Functions === */
#[cfg(not(windows))]
fn parse_nmcli_wifi_row(line: &str) -> Option<NmcliWifiRow> {
    let fields = split_nmcli_fields(line);
    if fields.len() < 8 {
        return None;
    }
    Some(NmcliWifiRow {
        active: fields[0].eq_ignore_ascii_case("yes"),
        ssid: non_empty(fields[1].clone()),
        bssid: non_empty(fields[2].clone()),
        channel: fields[3].parse().ok(),
        signal_percent: fields[4].parse().ok(),
        rate_mbps: parse_nmcli_rate_mbps(&fields[5]),
        security: non_empty(fields[6].clone()),
        device: non_empty(fields[7].clone()),
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
fn parse_nmcli_rate_mbps(value: &str) -> Option<f64> {
    value
        .split_whitespace()
        .next()
        .and_then(|rate| rate.parse().ok())
}

#[cfg(not(windows))]
fn non_empty(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

/* === Tests === */
