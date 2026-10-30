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
use std::time::Duration;

// Third Party Imports

// Local Imports
use crate::{HostOptions, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug)]
pub struct BluetoothSnapshot {
    pub adapter_name: Option<String>,
    pub adapter_count: Option<usize>,
    pub connected_device_count: Option<usize>,
    pub present_device_count: usize,
    pub radio_state: Option<String>,
    pub device_names: Vec<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn snapshot(options: &HostOptions, _: Duration) -> Result<BluetoothSnapshot> {
    let names = super::devices::bluetooth_presence()?;
    let filter = options.bluetooth_adapter_name.as_ref().map(|value| value.to_lowercase());
    let device_names =
        names.into_iter().filter(|name| filter.as_ref().is_none_or(|filter| name.to_lowercase().contains(filter))).collect::<Vec<_>>();
    let radios = crate::host::controls::read_radio("bluetooth").ok();
    let adapter_count = radios.as_ref().map(|state| state.radios.len());
    let radio_state = match radios.as_ref() {
        Some(state) if state.radios.iter().any(|radio| radio.on) => "on",
        Some(state) if !state.radios.is_empty() => "off",
        _ => "unavailable",
    };
    Ok(BluetoothSnapshot {
        adapter_name: device_names.first().cloned(),
        adapter_count,
        connected_device_count: None,
        present_device_count: device_names.len(),
        radio_state: Some(radio_state.into()),
        device_names,
    })
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_presence() {
        let reading = snapshot(&HostOptions::default(), crate::host::bluetooth::POLICY.attempt_timeout).unwrap();
        assert_eq!(reading.present_device_count, reading.device_names.len());
        assert!(matches!(reading.radio_state.as_deref(), Some("on" | "off" | "disabled" | "unavailable")));
    }
}
