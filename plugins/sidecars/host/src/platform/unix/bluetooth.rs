/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{collections::BTreeMap, time::Duration};

use crate::HostOptions;
use crate::collectors::maybe_insert;
use dokime_domain::model::values::Value;

pub(crate) fn linux_bluetooth_snapshot(options: &HostOptions, timeout: Duration) -> Result<BTreeMap<String, Value>, String> {
    let show_output = bluetoothctl_output(&["show"], timeout)?;
    let controller = parse_linux_controller(&show_output);
    if let Some(filter) = options.bluetooth_adapter_name.as_deref().map(str::trim).filter(|filter| !filter.is_empty()) {
        let matches_filter = controller.iter().any(|value| value.eq_ignore_ascii_case(filter));
        if !matches_filter {
            let mut values = BTreeMap::new();
            values.insert("bluetooth_adapter_count".to_owned(), Value::Integer(0));
            values.insert("bluetooth_connected_device_count".to_owned(), Value::Integer(0));
            values.insert("bluetooth_present_device_count".to_owned(), Value::Integer(0));
            values.insert("bluetooth_device_names".to_owned(), Value::Array(vec![]));
            return Ok(values);
        }
    }

    let devices = bluetoothctl_output(&["devices"], timeout).map(|output| parse_linux_device_names(&output)).unwrap_or_default();
    let connected_devices =
        bluetoothctl_output(&["devices", "Connected"], timeout).map(|output| parse_linux_device_names(&output)).unwrap_or_default();
    let mut values = BTreeMap::new();
    maybe_insert(&mut values, "bluetooth_adapter", controller.first().cloned().map(Value::from));
    values.insert("bluetooth_adapter_count".to_owned(), Value::Integer(if controller.is_empty() { 0 } else { 1 }));
    values.insert("bluetooth_connected_device_count".to_owned(), Value::Integer(connected_devices.len() as i64));
    values.insert("bluetooth_present_device_count".to_owned(), Value::Integer(devices.len() as i64));
    maybe_insert(&mut values, "bluetooth_radio_state", parse_linux_radio_state(&show_output).map(Value::from));
    values.insert("bluetooth_device_names".to_owned(), Value::Array(devices.into_iter().map(Value::from).collect()));
    Ok(values)
}

fn bluetoothctl_output(args: &[&str], timeout: Duration) -> Result<String, String> {
    let mut command = dokime_domain::process::command("bluetoothctl");
    command.args(args);
    let output =
        crate::collectors::run_command_bounded(command, timeout).map_err(|error| format!("bluetoothctl could not be started: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(if stderr.is_empty() { format!("bluetoothctl exited with status {}", output.status) } else { stderr });
    }
    String::from_utf8(output.stdout).map_err(|error| format!("bluetoothctl output was not UTF-8: {error}"))
}

fn parse_linux_controller(output: &str) -> Vec<String> {
    let mut values = Vec::new();
    for line in output.lines().map(str::trim) {
        if let Some(rest) = line.strip_prefix("Controller ") {
            values.push(rest.to_owned());
        } else if let Some(rest) = line.strip_prefix("Name: ") {
            values.push(rest.to_owned());
        } else if let Some(rest) = line.strip_prefix("Alias: ") {
            values.push(rest.to_owned());
        }
    }
    values
}

fn parse_linux_radio_state(output: &str) -> Option<String> {
    for line in output.lines().map(str::trim) {
        if let Some(powered) = line.strip_prefix("Powered: ") {
            return Some(if powered.eq_ignore_ascii_case("yes") { "powered".to_owned() } else { "off".to_owned() });
        }
    }
    None
}

fn parse_linux_device_names(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(3, ' ');
            match (parts.next(), parts.next(), parts.next()) {
                (Some("Device"), Some(_address), Some(name)) => Some(name.trim().to_owned()),
                _ => None,
            }
        })
        .filter(|name| !name.is_empty())
        .collect()
}
