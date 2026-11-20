/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bluetooth Adapters and Devices Through BlueZ on the System D-Bus

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use crate::domain::telemetry::Bluetooth;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// The Inventory Columns: BlueZ Adapters Matching the Configured Adapter and Their Devices.
pub fn snapshot(adapter: Option<&str>, timeout: Duration) -> Result<Bluetooth, String> {
    let connection = zbus::blocking::connection::Builder::system().map_err(error)?.method_timeout(timeout).build().map_err(error)?;
    let manager = zbus::blocking::fdo::ObjectManagerProxy::builder(&connection)
        .destination("org.bluez")
        .map_err(error)?
        .path("/")
        .map_err(error)?
        .build()
        .map_err(error)?;
    inventory(&manager.get_managed_objects().map_err(error)?, adapter)
}

/// The Inventory Columns of BlueZ's Managed Objects.
pub fn inventory(objects: &zbus::fdo::ManagedObjects, filter: Option<&str>) -> Result<Bluetooth, String> {
    let mut adapters = Vec::new();
    let mut powered = false;
    for (path, interfaces) in objects {
        let Some(properties) = interfaces.get("org.bluez.Adapter1") else {
            continue;
        };
        let text = |key: &str| properties.get(key).and_then(|value| <&str>::try_from(value).ok());
        if filter.map(str::trim).filter(|filter| !filter.is_empty()).is_some_and(|filter| {
            ![text("Name"), text("Alias"), text("Address"), Some(path.as_str())]
                .into_iter()
                .flatten()
                .any(|value| value.eq_ignore_ascii_case(filter))
        }) {
            continue;
        }
        let on = properties
            .get("Powered")
            .and_then(|value| bool::try_from(value).ok())
            .ok_or_else(|| "Read BlueZ Adapter: No Powered Property; Update BlueZ".to_owned())?;
        powered |= on;
        adapters.push((path.as_str(), text("Alias").or_else(|| text("Name")).unwrap_or(path.as_str())));
    }
    adapters.sort_unstable();
    let mut devices = Vec::new();
    let mut connected = 0;
    for (path, interfaces) in objects {
        let Some(properties) = interfaces.get("org.bluez.Device1") else {
            continue;
        };
        let adapter = properties.get("Adapter").and_then(|value| <&zbus::zvariant::ObjectPath<'_>>::try_from(value).ok());
        if !adapter.is_some_and(|path| adapters.iter().any(|(id, _)| *id == path.as_str())) {
            continue;
        }
        let name = properties.get("Alias").or_else(|| properties.get("Name")).and_then(|value| <&str>::try_from(value).ok());
        devices.push(name.unwrap_or(path.as_str()).to_owned());
        connected += usize::from(properties.get("Connected").and_then(|value| bool::try_from(value).ok()).unwrap_or(false));
    }
    devices.sort();
    let radio_state = if powered {
        "on"
    } else if adapters.is_empty() {
        "unavailable"
    } else {
        "off"
    };
    Ok(Bluetooth {
        bluetooth_adapter: adapters.first().map(|(_, name)| (*name).to_owned()),
        bluetooth_adapter_count: Some(count(adapters.len())),
        bluetooth_connected_device_count: Some(count(connected)),
        bluetooth_present_device_count: Some(count(devices.len())),
        bluetooth_radio_state: Some(radio_state.into()),
        bluetooth_device_names: devices,
        ..Bluetooth::default()
    })
}

/* === Private Functions === */

fn error(error: impl std::fmt::Display) -> String {
    format!("Read BlueZ Inventory: {error}; Start the Bluetooth Service")
}

fn count(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use zbus::zvariant::{ObjectPath, OwnedValue, Str};

    fn objects() -> zbus::fdo::ManagedObjects {
        [
            (
                "/org/bluez/hci0",
                "org.bluez.Adapter1",
                vec![("Alias", OwnedValue::from(Str::from("Radio"))), ("Powered", OwnedValue::from(true))],
            ),
            (
                "/org/bluez/hci0/dev_a",
                "org.bluez.Device1",
                vec![
                    ("Alias", OwnedValue::from(Str::from("Headset"))),
                    ("Adapter", OwnedValue::from(ObjectPath::try_from("/org/bluez/hci0").unwrap())),
                    ("Connected", OwnedValue::from(true)),
                ],
            ),
        ]
        .into_iter()
        .map(|(path, interface, properties)| {
            (
                path.try_into().unwrap(),
                [(interface.try_into().unwrap(), properties.into_iter().map(|(key, value)| (key.into(), value)).collect())]
                    .into_iter()
                    .collect(),
            )
        })
        .collect()
    }

    #[test]
    fn bluez_presence() {
        let row = inventory(&objects(), None).unwrap();
        assert_eq!(row.bluetooth_adapter.as_deref(), Some("Radio"));
        assert_eq!(row.bluetooth_adapter_count, Some(1));
        assert_eq!(row.bluetooth_connected_device_count, Some(1));
        assert_eq!(row.bluetooth_present_device_count, Some(1));
        assert_eq!(row.bluetooth_radio_state.as_deref(), Some("on"));
        assert_eq!(row.bluetooth_device_names, vec!["Headset".to_owned()]);
    }

    #[test]
    fn bluez_selection() {
        let row = inventory(&objects(), Some("Other Radio")).unwrap();
        assert_eq!(row.bluetooth_adapter_count, Some(0));
        assert_eq!(row.bluetooth_present_device_count, Some(0));
        assert_eq!(row.bluetooth_radio_state.as_deref(), Some("unavailable"));
    }

    #[test]
    fn bluez_empty() {
        let row = inventory(&Default::default(), None).unwrap();
        assert!(row.bluetooth_device_names.is_empty());
    }
}
