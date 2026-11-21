/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Radio, Adapter and Power Profile Reads and Writes

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

// Third Party Imports
use zbus::blocking::connection::Builder;
use zbus::blocking::{Connection, Proxy};

// Local Imports
use crate::adapters::Adapter;
use crate::controls::Timing;
use crate::domain::controls::{ControlState, Radio, RadioState};
use dokime::{Error, Result};

/* === Constants === */

/// CAP_NET_ADMIN in the Effective Capability Mask.
const NET_ADMIN: u64 = 1 << 12;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Whether the Process Runs as Root or Holds CAP_NET_ADMIN.
pub fn administrator() -> Result<bool> {
    // SAFETY: geteuid Has No Preconditions.
    if unsafe { libc::geteuid() } == 0 {
        return Ok(true);
    }
    let status = fs::read_to_string("/proc/self/status").map_err(probe)?;
    let capabilities = status
        .lines()
        .find_map(|line| line.strip_prefix("CapEff:"))
        .and_then(|value| u64::from_str_radix(value.trim(), 16).ok())
        .unwrap_or(0);
    Ok(capabilities & NET_ADMIN != 0)
}

/// Every Radio of One Kind With Whether It Is On: NetworkManager for Wi-Fi and Cellular, BlueZ for Bluetooth.
pub fn read_radio(kind: Radio, timing: &Timing) -> Result<ControlState> {
    let connection = connection(timing)?;
    let mut radios = Vec::new();
    if kind == Radio::Wifi || kind == Radio::Cellular {
        let proxy =
            Proxy::new(&connection, "org.freedesktop.NetworkManager", "/org/freedesktop/NetworkManager", "org.freedesktop.NetworkManager")
                .map_err(probe)?;
        let property = if kind == Radio::Wifi { "WirelessEnabled" } else { "WwanEnabled" };
        let on = proxy.get_property::<bool>(property).map_err(probe)?;
        radios.push(RadioState {
            id: kind.as_str().into(),
            kind: kind.as_str().into(),
            on,
        });
    } else if kind == Radio::Bluetooth {
        let directory = match fs::read_dir("/sys/class/bluetooth") {
            Ok(directory) => Some(directory),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(probe(error)),
        };
        for entry in directory.into_iter().flatten() {
            let entry = entry.map_err(probe)?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let controller = name.starts_with("hci") && name[3..].chars().all(|character| character.is_ascii_digit());
            if !controller {
                continue;
            }
            let path = format!("/org/bluez/{name}");
            let proxy = Proxy::new(&connection, "org.bluez", path.as_str(), "org.bluez.Adapter1").map_err(probe)?;
            let on = proxy.get_property::<bool>("Powered").map_err(probe)?;
            drop(proxy);
            radios.push(RadioState { id: path, kind: kind.as_str().into(), on });
        }
    }
    Ok(ControlState { radios, ..Default::default() })
}

/// Whether the Interface Is Administratively Up.
pub fn read_adapter(id: &str) -> Result<bool> {
    let (socket, mut request) = interface(id)?;
    // SAFETY: The Request Names a Valid Interface Buffer for the Socket.
    if unsafe { libc::ioctl(socket.as_raw_fd(), libc::SIOCGIFFLAGS, &mut request) } != 0 {
        return Err(native_error());
    }
    Ok(unsafe { request.ifr_ifru.ifru_flags } & libc::IFF_UP as i16 != 0)
}

/// Write Each Radio, Adapter and Power Profile of the State, Then Verify Every Target; Restoring Writes Even Targets That Already Read as
/// Requested.
pub fn apply(state: &ControlState, restoring: bool, timing: &Timing) -> Result<()> {
    for radio in &state.radios {
        let kind = radio.radio()?;
        let current = read_radio(kind, timing)?;
        let current = current
            .radios
            .iter()
            .find(|current| current.id == radio.id)
            .ok_or_else(|| Error::Invalid("Set Radio: Reconnect the Radio; It Is No Longer Present".into()))?;
        if current.on == radio.on && !restoring {
            continue;
        }
        let connection = connection(timing)?;
        if kind == Radio::Bluetooth {
            let proxy = Proxy::new(&connection, "org.bluez", radio.id.as_str(), "org.bluez.Adapter1").map_err(probe)?;
            proxy.set_property("Powered", radio.on).map_err(probe)?;
        } else {
            let proxy = Proxy::new(
                &connection,
                "org.freedesktop.NetworkManager",
                "/org/freedesktop/NetworkManager",
                "org.freedesktop.NetworkManager",
            )
            .map_err(probe)?;
            let property = if kind == Radio::Wifi { "WirelessEnabled" } else { "WwanEnabled" };
            proxy.set_property(property, radio.on).map_err(probe)?;
        }
    }
    for adapter in &state.adapters {
        if read_adapter(&adapter.id)? == adapter.enabled && !restoring {
            continue;
        }
        let (socket, mut request) = interface(&adapter.id)?;
        // SAFETY: The Request Names a Valid Interface Buffer for the Socket.
        if unsafe { libc::ioctl(socket.as_raw_fd(), libc::SIOCGIFFLAGS, &mut request) } != 0 {
            return Err(native_error());
        }
        let flags = unsafe { request.ifr_ifru.ifru_flags };
        request.ifr_ifru.ifru_flags = if adapter.enabled { flags | libc::IFF_UP as i16 } else { flags & !(libc::IFF_UP as i16) };
        if unsafe { libc::ioctl(socket.as_raw_fd(), libc::SIOCSIFFLAGS, &request) } != 0 {
            return Err(native_error());
        }
    }
    if let Some(plan) = &state.power_plan
        && (crate::power::active_plan()?.id != *plan || restoring)
    {
        let choices = fs::read_to_string("/sys/firmware/acpi/platform_profile_choices").map_err(probe)?;
        if !choices.split_whitespace().any(|choice| choice == plan) {
            return Err(Error::Invalid(format!("Set Power Plan: Choose a Profile From {}", choices.trim())));
        }
        fs::write("/sys/firmware/acpi/platform_profile", format!("{plan}\n")).map_err(probe)?;
    }

    // Verify Every Target
    for radio in &state.radios {
        let current = read_radio(radio.radio()?, timing)?;
        let settled = current.radios.iter().any(|current| current.id == radio.id && current.on == radio.on);
        if !settled {
            return Err(Error::Invalid("Set Radio: Check the D-Bus Authorization; the Radio Did Not Reach the Requested State".into()));
        }
    }
    for adapter in &state.adapters {
        if read_adapter(&adapter.id)? != adapter.enabled {
            return Err(Error::Invalid("Set Adapter: Check CAP_NET_ADMIN; the Adapter Did Not Reach the Requested State".into()));
        }
    }
    if let Some(plan) = &state.power_plan
        && crate::power::active_plan()?.id != *plan
    {
        return Err(Error::Invalid("Set Power Plan: Check the Firmware; the Profile Did Not Become Active".into()));
    }
    Ok(())
}

/// Whether the Adapter Is Carried Over Bluetooth.
pub fn bluetooth_adapter(adapter: &Adapter) -> Result<bool> {
    let subsystem = fs::canonicalize(format!("/sys/class/net/{}/device/subsystem", adapter.name));
    let bluetooth = subsystem.ok().and_then(|path| path.file_name().map(|name| name == "bluetooth")).unwrap_or(false);
    Ok(adapter.name.starts_with("bnep") || bluetooth)
}

/// The Local Addresses of the Process's Established Non-Loopback TCP Connections.
pub fn connected_addresses(pid: u32, _: &Timing) -> Result<BTreeSet<String>> {
    let mut owned = BTreeSet::new();
    for entry in fs::read_dir(format!("/proc/{pid}/fd")).map_err(probe)? {
        let entry = entry.map_err(probe)?;
        let target = match fs::read_link(entry.path()) {
            Ok(target) => target,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(probe(error)),
        };
        if let Some(inode) = target.to_string_lossy().strip_prefix("socket:[").and_then(|text| text.strip_suffix(']')) {
            owned.insert(inode.to_owned());
        }
    }
    let mut locals = BTreeSet::new();
    for table in ["tcp", "tcp6"] {
        let rows = fs::read_to_string(format!("/proc/{pid}/net/{table}")).map_err(probe)?;
        for line in rows.lines().skip(1) {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            if fields.len() < 10 || fields[3] != "01" || !owned.contains(fields[9]) {
                continue;
            }
            if socket_ip(fields[2])?.is_loopback() {
                continue;
            }
            locals.insert(socket_ip(fields[1])?.to_string());
        }
    }
    Ok(locals)
}

/* === Private Functions === */

fn connection(timing: &Timing) -> Result<Connection> {
    Builder::system().map_err(probe)?.method_timeout(timing.control_probe).build().map_err(probe)
}

fn interface(id: &str) -> Result<(OwnedFd, libc::ifreq)> {
    if id.is_empty() || id.len() >= libc::IFNAMSIZ || id.as_bytes().contains(&0) {
        return Err(Error::Invalid(format!("Set Adapter: Choose an Identity From host.network.adapters Instead of {id:?}")));
    }
    // SAFETY: socket Has No Preconditions; the Descriptor Is Owned Below.
    let descriptor = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM | libc::SOCK_CLOEXEC, 0) };
    if descriptor < 0 {
        return Err(native_error());
    }
    let socket = unsafe { OwnedFd::from_raw_fd(descriptor) };
    let mut request: libc::ifreq = unsafe { std::mem::zeroed() };
    for (target, source) in request.ifr_name.iter_mut().zip(id.bytes()) {
        *target = source as libc::c_char;
    }
    Ok((socket, request))
}

fn socket_ip(socket: &str) -> Result<IpAddr> {
    let hex = socket.split_once(':').ok_or_else(|| probe("Invalid Native Socket Address"))?.0;
    if hex.len() == 8 {
        let value = u32::from_str_radix(hex, 16).map_err(probe)?;
        return Ok(IpAddr::V4(Ipv4Addr::from(value.to_ne_bytes())));
    }
    if hex.len() != 32 {
        return Err(probe("Invalid IPv6 Socket Address"));
    }
    let mut bytes = [0u8; 16];
    for (index, word) in bytes.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        word.copy_from_slice(&u32::from_str_radix(&hex[index * 8..index * 8 + 8], 16).map_err(probe)?.to_ne_bytes());
    }
    let address = Ipv6Addr::from(bytes);
    Ok(address.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(IpAddr::V6(address)))
}

fn native_error() -> Error {
    probe(io::Error::last_os_error())
}

fn probe(error: impl std::fmt::Display) -> Error {
    Error::Invalid(format!("Set Control: Retry Once the System Answers: {error}"))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_socket_addresses_parse_both_families() {
        assert_eq!(socket_ip("0100007F:1234").unwrap(), "127.0.0.1".parse::<IpAddr>().unwrap());
        assert_eq!(socket_ip("00000000000000000000000001000000:1234").unwrap(), "::1".parse::<IpAddr>().unwrap());
        assert!(socket_ip("invalid").is_err());
    }
}
