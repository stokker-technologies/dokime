/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bluetooth Radio and Adapter Inventory

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::controls::Timing;
use crate::domain::controls::{Radio, RadioPower};
use crate::domain::readings::Bluetooth;
use crate::platform::windows::controls::read_radio;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// The Inventory Columns: Present Bluetooth Devices Matching the Configured Adapter and the Bluetooth Radios' Power.
pub fn snapshot(adapter: Option<&str>, timing: &Timing) -> Result<Bluetooth, String> {
    let names = super::devices::bluetooth_presence().map_err(|error| error.to_string())?;
    let filter = adapter.map(str::to_lowercase);
    let device_names =
        names.into_iter().filter(|name| filter.as_ref().is_none_or(|filter| name.to_lowercase().contains(filter))).collect::<Vec<_>>();
    // An Unreadable Radio Leaves the Radio Columns Empty.
    let radios = read_radio(Radio::Bluetooth, timing).ok().map(|state| state.radios.iter().map(|radio| radio.on).collect::<Vec<_>>());
    let adapter_count = radios.as_ref().map(Vec::len);
    let radio_state = match radios.as_deref() {
        Some(radios) if radios.iter().any(|on| *on) => Some(RadioPower::On),
        Some(radios) if !radios.is_empty() => Some(RadioPower::Off),
        _ => None,
    };
    Ok(Bluetooth {
        bluetooth_adapter: device_names.first().cloned(),
        bluetooth_adapter_count: adapter_count.map(count),
        bluetooth_present_device_count: Some(count(device_names.len())),
        bluetooth_radio_state: Some(radio_state.map_or("unavailable", RadioPower::as_str).into()),
        bluetooth_device_names: device_names,
        ..Bluetooth::default()
    })
}

/* === Private Functions === */

fn count(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/* === Tests === */
