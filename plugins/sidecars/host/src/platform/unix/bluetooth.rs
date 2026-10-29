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
use std::{collections::BTreeMap, time::Duration};

// Third Party Imports

// Local Imports
use crate::HostOptions;
use crate::collectors::maybe_insert;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn linux_bluetooth_snapshot(options: &HostOptions, timeout: Duration) -> Result<BTreeMap<String, Value>, String> {
    let connection = zbus::blocking::connection::Builder::system().map_err(error)?.method_timeout(timeout).build().map_err(error)?;
    let manager = zbus::blocking::fdo::ObjectManagerProxy::builder(&connection)
        .destination("org.bluez")
        .map_err(error)?
        .path("/")
        .map_err(error)?
        .build()
        .map_err(error)?;
    values(&manager.get_managed_objects().map_err(error)?, options.bluetooth_adapter_name.as_deref())
}

/* === Private Functions === */

fn error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn values(objects: &zbus::fdo::ManagedObjects, filter: Option<&str>) -> Result<BTreeMap<String, Value>, String> {
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
            .ok_or_else(|| "BlueZ adapter has no Powered property".to_owned())?;
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
    let mut values = BTreeMap::new();
    maybe_insert(&mut values, "bluetooth_adapter", adapters.first().map(|(_, name)| Value::from(*name)));
    values.insert("bluetooth_adapter_count".into(), Value::Integer(adapters.len() as i64));
    values.insert("bluetooth_connected_device_count".into(), Value::Integer(connected as i64));
    values.insert("bluetooth_present_device_count".into(), Value::Integer(devices.len() as i64));
    values.insert(
        "bluetooth_radio_state".into(),
        Value::from(if powered {
            "on"
        } else if adapters.is_empty() {
            "unavailable"
        } else {
            "off"
        }),
    );
    values.insert("bluetooth_device_names".into(), Value::Array(devices.into_iter().map(Value::from).collect()));
    Ok(values)
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
    fn native_presence() {
        let values = values(&objects(), None).unwrap();
        assert_eq!(values["bluetooth_adapter_count"], Value::Integer(1));
        assert_eq!(values["bluetooth_connected_device_count"], Value::Integer(1));
        assert_eq!(values["bluetooth_present_device_count"], Value::Integer(1));
        assert_eq!(values["bluetooth_radio_state"], Value::from("on"));
    }
    #[test]
    fn native_selection() {
        let values = values(&objects(), Some("Other Radio")).unwrap();
        assert_eq!(values["bluetooth_adapter_count"], Value::Integer(0));
        assert_eq!(values["bluetooth_present_device_count"], Value::Integer(0));
        assert_eq!(values["bluetooth_radio_state"], Value::from("unavailable"));
    }
    #[test]
    fn native_empty() {
        let values = values(&Default::default(), None).unwrap();
        assert_eq!(values["bluetooth_device_names"], Value::Array(Vec::new()));
    }
}
