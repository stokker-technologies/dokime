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
use windows::Devices::Enumeration::DeviceInformation;
use windows::Devices::Radios::{Radio, RadioKind, RadioState};
use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize};

// Local Imports
use crate::domain::controls::RadioPower;
use crate::domain::telemetry::Bluetooth;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// The Windows Runtime Initialized on This Thread for the Radio Queries.
struct Apartment;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Apartment {
    fn new() -> Result<Self, String> {
        crate::platform::windows::apartment::retain().map_err(|error| error.to_string())?;
        // SAFETY: Initializing the runtime has no preconditions; Drop balances it on this thread.
        unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.map_err(|error| format!("Initialize Windows Runtime: {error}"))?;
        Ok(Self)
    }
}

impl Drop for Apartment {
    fn drop(&mut self) {
        // SAFETY: Balances the successful RoInitialize of this thread.
        unsafe { RoUninitialize() };
    }
}

/* === Public Functions === */

/// The Inventory Columns: Present Bluetooth Devices Matching the Configured Adapter and the Bluetooth Radios' Power.
pub fn snapshot(adapter: Option<&str>) -> Result<Bluetooth, String> {
    let names = super::devices::bluetooth_presence().map_err(|error| error.to_string())?;
    let filter = adapter.map(str::to_lowercase);
    let device_names =
        names.into_iter().filter(|name| filter.as_ref().is_none_or(|filter| name.to_lowercase().contains(filter))).collect::<Vec<_>>();
    // An Unreadable Radio Leaves the Radio Columns Empty; Only an Unknown Native Value Fails the Inventory.
    let radios = radios()?;
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

/// Whether Each Bluetooth Radio Is On; None When the Radios Cannot Be Read, an Error for an Unknown Native Value.
fn radios() -> Result<Option<Vec<bool>>, String> {
    let readable = |error: windows::core::Error| error.to_string();
    let read = || -> Result<Result<Vec<bool>, String>, String> {
        let _apartment = Apartment::new()?;
        let selector = Radio::GetDeviceSelector().map_err(readable)?;
        let devices = DeviceInformation::FindAllAsyncAqsFilter(&selector).map_err(readable)?.join().map_err(readable)?;
        let mut radios = Vec::new();
        for device in devices {
            let id = device.Id().map_err(readable)?;
            let radio = Radio::FromIdAsync(&id).map_err(readable)?.join().map_err(readable)?;
            match radio.Kind().map_err(readable)? {
                RadioKind::Bluetooth => {}
                RadioKind::WiFi | RadioKind::MobileBroadband | RadioKind::Other => continue,
                kind => return Ok(Err(format!("Read Bluetooth Radios: Unknown Radio Kind {}", kind.0))),
            }
            match radio.State().map_err(readable)? {
                RadioState::On => radios.push(true),
                RadioState::Off => radios.push(false),
                // A Radio Disabled by Hardware or Policy Makes the Radios Unreadable.
                RadioState::Disabled | RadioState::Unknown => return Err("Read Bluetooth Radios: a Radio Is Disabled".into()),
                state => return Ok(Err(format!("Read Bluetooth Radios: Unknown Radio Power {}", state.0))),
            }
        }
        Ok(Ok(radios))
    };
    match read() {
        Ok(radios) => radios.map(Some),
        Err(_) => Ok(None),
    }
}

/* === Tests === */
