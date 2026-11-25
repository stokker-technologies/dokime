/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bluetooth Adapters and Devices Through BlueZ on the System D-Bus, Radio Byte Counters Through a Raw HCI Socket

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::time::{Duration, Instant};

// Third Party Imports
use zbus::blocking::Connection;
use zbus::fdo::ManagedObjects;
use zbus::zvariant::ObjectPath;

// Local Imports
use crate::collectors::bluetooth::{LinkOptions, Radio, rate};
use crate::domain::readings::Bluetooth;
use crate::domain::telemetry::BluetoothRssi;

/* === Constants === */

const BTPROTO_HCI: libc::c_int = 1;
/// `_IOR('H', 211, int)`: Fill One `hci_dev_info` for the Device Id It Carries.
const HCIGETDEVINFO: u32 = 0x8004_48D3;
const BYTE_RX: usize = 8;
const BYTE_TX: usize = 9;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// The Link Collection's Native State: the BlueZ Connection, the Raw HCI Socket and the Previous Byte Counters per Device Id.
#[derive(Default)]
pub struct Link {
    bus: Option<Connection>,
    hci: Option<OwnedFd>,
    baseline: Option<(Instant, BTreeMap<u16, (u32, u32)>)>,
}

/// The Kernel's `struct hci_dev_info` From `include/net/bluetooth/hci_sock.h`, Byte for Byte.
#[repr(C)]
struct DeviceInfo {
    dev_id: u16,
    // Name, Address, Flags, Type, Features, Packet Type, Link Policy and Mode, MTUs and Packet Counts.
    _identity: [u8; 50],
    // err_rx, err_tx, cmd_tx, evt_rx, acl_tx, acl_rx, sco_tx, sco_rx, byte_rx, byte_tx.
    stat: [u32; 10],
}

const _: () = assert!(size_of::<DeviceInfo>() == 92);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Link {
    /// Each BlueZ Adapter's Connected Devices and Its HCI Byte Rates Since the Previous Read.
    pub fn radios(&mut self, options: &LinkOptions) -> Result<Vec<Radio>, String> {
        let bus = match &self.bus {
            Some(bus) => bus,
            None => self.bus.insert(connect(options.probe_timeout)?),
        };
        let objects = match managed_objects(bus) {
            Ok(objects) => objects,
            Err(error) => {
                self.bus = None;
                return Err(error);
            }
        };
        let mut radios = adapters(&objects);
        let hci = match &self.hci {
            Some(hci) => hci,
            None => self.hci.insert(open_hci()?),
        };
        let now = Instant::now();
        let mut counters = BTreeMap::new();
        for radio in &mut radios {
            let Some(dev_id) = radio.names.first().and_then(|path| device_id(path)) else {
                continue;
            };
            let info = statistics(hci, dev_id)?;
            let (received, transmitted) = (info.stat[BYTE_RX], info.stat[BYTE_TX]);
            counters.insert(info.dev_id, (received, transmitted));
            if let Some((at, previous)) = &self.baseline
                && let Some((previous_received, previous_transmitted)) = previous.get(&info.dev_id)
            {
                let elapsed_s = now.duration_since(*at).max(options.minimum_rate_window).as_secs_f64();
                radio.received = Some(rate(*previous_received, received, elapsed_s));
                radio.transmitted = Some(rate(*previous_transmitted, transmitted, elapsed_s));
            }
        }
        self.baseline = Some((now, counters));
        Ok(radios)
    }

    /// BlueZ Keeps a Device's Last RSSI Between Reports and Republishes It Only on Changes of 8 dBm or More, So a Polled
    /// Value Cannot Tell a Fresh Reading From a Stale One; Linux Records No RSSI.
    pub fn heard(&mut self, _devices: &[String]) -> Result<Vec<BluetoothRssi>, String> {
        Ok(Vec::new())
    }
}

/* === Public Functions === */

/// The Inventory Columns: BlueZ Adapters Matching the Configured Adapter and Their Devices.
pub fn snapshot(adapter: Option<&str>, timeout: Duration) -> Result<Bluetooth, String> {
    inventory(&managed_objects(&connect(timeout)?)?, adapter)
}

/// The Inventory Columns of BlueZ's Managed Objects.
pub fn inventory(objects: &ManagedObjects, filter: Option<&str>) -> Result<Bluetooth, String> {
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
    for (path, interfaces) in objects {
        let Some(properties) = interfaces.get("org.bluez.Device1") else {
            continue;
        };
        let adapter = properties.get("Adapter").and_then(|value| <&ObjectPath<'_>>::try_from(value).ok());
        if !adapter.is_some_and(|path| adapters.iter().any(|(id, _)| *id == path.as_str())) {
            continue;
        }
        let name = properties.get("Alias").or_else(|| properties.get("Name")).and_then(|value| <&str>::try_from(value).ok());
        devices.push(name.unwrap_or(path.as_str()).to_owned());
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
        bluetooth_present_device_count: Some(count(devices.len())),
        bluetooth_radio_state: Some(radio_state.into()),
        bluetooth_device_names: devices,
        ..Bluetooth::default()
    })
}

/// Each BlueZ Adapter as a Radio Named by Its Object Path, Name, Alias and Address, With Its Connected Devices.
pub fn adapters(objects: &ManagedObjects) -> Vec<Radio> {
    let mut radios = BTreeMap::new();
    for (path, interfaces) in objects {
        let Some(properties) = interfaces.get("org.bluez.Adapter1") else {
            continue;
        };
        let text = |key: &str| properties.get(key).and_then(|value| <&str>::try_from(value).ok()).map(str::to_owned);
        let names = std::iter::once(path.as_str().to_owned()).chain(["Name", "Alias", "Address"].into_iter().filter_map(text)).collect();
        radios.insert(
            path.as_str(),
            Radio {
                names,
                connections: Some(0),
                ..Radio::default()
            },
        );
    }
    for interfaces in objects.values() {
        let Some(properties) = interfaces.get("org.bluez.Device1") else {
            continue;
        };
        let connected = properties.get("Connected").and_then(|value| bool::try_from(value).ok()).unwrap_or(false);
        let adapter = properties.get("Adapter").and_then(|value| <&ObjectPath<'_>>::try_from(value).ok());
        if let (true, Some(radio)) = (connected, adapter.and_then(|path| radios.get_mut(path.as_str()))) {
            radio.connections = radio.connections.map(|count| count + 1);
        }
    }
    radios.into_values().collect()
}

/* === Private Functions === */

fn connect(timeout: Duration) -> Result<Connection, String> {
    zbus::blocking::connection::Builder::system().map_err(error)?.method_timeout(timeout).build().map_err(error)
}

fn managed_objects(connection: &Connection) -> Result<ManagedObjects, String> {
    zbus::blocking::fdo::ObjectManagerProxy::builder(connection)
        .destination("org.bluez")
        .map_err(error)?
        .path("/")
        .map_err(error)?
        .build()
        .map_err(error)?
        .get_managed_objects()
        .map_err(error)
}

/// The HCI Device Id of a BlueZ Adapter Path Such as `/org/bluez/hci0`.
fn device_id(path: &str) -> Option<u16> {
    path.rsplit('/').next()?.strip_prefix("hci")?.parse().ok()
}

fn open_hci() -> Result<OwnedFd, String> {
    // SAFETY: socket Takes No Pointers; a Non-Negative Result Is a New Descriptor Nothing Else Owns.
    let descriptor = unsafe { libc::socket(libc::AF_BLUETOOTH, libc::SOCK_RAW | libc::SOCK_CLOEXEC, BTPROTO_HCI) };
    if descriptor < 0 {
        return Err(hci_error(io::Error::last_os_error()));
    }
    // SAFETY: The Descriptor Was Just Opened and Is Owned Only Here.
    Ok(unsafe { OwnedFd::from_raw_fd(descriptor) })
}

fn statistics(socket: &OwnedFd, dev_id: u16) -> Result<DeviceInfo, String> {
    let mut info = DeviceInfo {
        dev_id,
        _identity: [0; 50],
        stat: [0; 10],
    };
    // SAFETY: HCIGETDEVINFO Reads the Device Id and Fills One hci_dev_info, Which DeviceInfo Mirrors.
    if unsafe { libc::ioctl(socket.as_raw_fd(), HCIGETDEVINFO as libc::Ioctl, &raw mut info) } < 0 {
        return Err(hci_error(io::Error::last_os_error()));
    }
    Ok(info)
}

fn error(error: impl std::fmt::Display) -> String {
    format!("Read BlueZ Objects: {error}; Start the Bluetooth Service")
}

fn hci_error(error: io::Error) -> String {
    format!("Read HCI Statistics: {error}; Load the Kernel Bluetooth Module")
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

    #[test]
    fn bluez_adapters_count_connections_and_name_their_hci_device() {
        let radios = adapters(&objects());
        assert_eq!(radios.len(), 1);
        assert_eq!(radios[0].names, vec!["/org/bluez/hci0".to_owned(), "Radio".to_owned()]);
        assert_eq!(radios[0].connections, Some(1));
        assert_eq!(radios[0].received, None);
        assert_eq!(device_id(&radios[0].names[0]), Some(0));
        assert_eq!(device_id("/org/bluez/hci12"), Some(12));
        assert_eq!(device_id("/org/bluez/radio"), None);
    }
}
