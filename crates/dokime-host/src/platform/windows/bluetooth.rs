/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{process::Command, time::Duration};

use serde::Deserialize;

use crate::{Error, HostOptions, Result};

#[derive(Clone, Debug)]
pub struct BluetoothSnapshot {
    pub adapter_name: Option<String>,
    pub adapter_count: usize,
    pub connected_device_count: Option<usize>,
    pub present_device_count: usize,
    pub radio_state: Option<String>,
    pub device_names: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct BluetoothPnPEntity {
    #[serde(alias = "Name", alias = "FriendlyName")]
    name: Option<String>,

    #[serde(rename = "Status")]
    status: Option<String>,

    #[serde(rename = "Present")]
    present: Option<bool>,

    #[serde(rename = "ConfigManagerErrorCode")]
    config_manager_error_code: Option<u32>,
}

pub fn snapshot(options: &HostOptions, timeout: Duration) -> Result<BluetoothSnapshot> {
    let output = bluetooth_pnp_json(timeout)?;
    let devices = bluetooth_devices_from_json(&output)?;
    let present = devices.iter().filter(|device| device.present.unwrap_or(false)).collect::<Vec<_>>();
    let adapter_filter = options.bluetooth_adapter_name.as_ref().map(|value| value.to_lowercase());
    let adapter_names = present
        .iter()
        .filter_map(|device| device.name.clone())
        .filter(|name| adapter_filter.as_ref().map(|filter| name.to_lowercase().contains(filter)).unwrap_or(true))
        .collect::<Vec<_>>();
    let ok_count = present
        .iter()
        .filter(|device| {
            device.status.as_deref().map(|status| status.eq_ignore_ascii_case("ok")).unwrap_or(false)
                && device.config_manager_error_code.unwrap_or(0) == 0
        })
        .count();

    Ok(BluetoothSnapshot {
        adapter_name: adapter_names.first().cloned(),
        adapter_count: adapter_names.len(),
        connected_device_count: None,
        present_device_count: present.len(),
        radio_state: Some(if ok_count > 0 { "present" } else { "unavailable" }.to_owned()),
        device_names: adapter_names,
    })
}

fn bluetooth_pnp_json(timeout: Duration) -> Result<String> {
    let script =
        "Get-PnpDevice -Class Bluetooth | Select-Object FriendlyName,Status,Present,ConfigManagerErrorCode | ConvertTo-Json -Compress";
    let mut command = Command::new("powershell");
    command.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script]);
    super::command::run_bounded(command, timeout).map_err(|error| Error::Probe(error.to_string()))
}

fn bluetooth_devices_from_json(output: &str) -> Result<Vec<BluetoothPnPEntity>> {
    if output.trim().is_empty() {
        return Ok(Vec::new());
    }
    let value: serde_json::Value = serde_json::from_str(output).map_err(|error| Error::Probe(error.to_string()))?;
    if value.is_array() {
        return serde_json::from_value(value).map_err(|error| Error::Probe(error.to_string()));
    }
    serde_json::from_value(value).map(|device| vec![device]).map_err(|error| Error::Probe(error.to_string()))
}
