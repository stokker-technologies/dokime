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

use crate::{
    Error, Result,
    domain::controls::{AdapterState, ControlState, Radio, RadioState},
};
use std::{
    collections::BTreeSet,
    fs,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
};
use zbus::blocking::{Connection, Proxy, connection::Builder};

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn administrator() -> Result<bool> {
    if unsafe { libc::geteuid() } == 0 {
        return Ok(true);
    }
    let status = fs::read_to_string("/proc/self/status").map_err(probe)?;
    let capabilities = status
        .lines()
        .find_map(|line| line.strip_prefix("CapEff:"))
        .and_then(|value| u64::from_str_radix(value.trim(), 16).ok())
        .unwrap_or(0);
    Ok(capabilities & (1 << 12) != 0)
}

pub fn read_radio(kind: Radio) -> Result<ControlState> {
    let connection = connection()?;
    let mut radios = Vec::new();
    if kind == Radio::Wifi || kind == Radio::Cellular {
        let proxy =
            Proxy::new(&connection, "org.freedesktop.NetworkManager", "/org/freedesktop/NetworkManager", "org.freedesktop.NetworkManager")
                .map_err(probe)?;
        let property = if kind == Radio::Wifi { "WirelessEnabled" } else { "WwanEnabled" };
        let on = proxy.get_property::<bool>(property).map_err(probe)?;
        radios.push(RadioState { id: kind.as_str().into(), kind, on });
    } else if kind == Radio::Bluetooth {
        let directory = match fs::read_dir("/sys/class/bluetooth") {
            Ok(directory) => Some(directory),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(probe(error)),
        };
        if let Some(directory) = directory {
            for entry in directory {
                let entry = entry.map_err(probe)?;
                let name = entry.file_name().to_string_lossy().into_owned();
                if !name.starts_with("hci") || !name[3..].chars().all(|character| character.is_ascii_digit()) {
                    continue;
                }
                let path = format!("/org/bluez/{name}");
                let proxy = Proxy::new(&connection, "org.bluez", path.as_str(), "org.bluez.Adapter1").map_err(probe)?;
                let on = proxy.get_property::<bool>("Powered").map_err(probe)?;
                radios.push(RadioState { id: path, kind, on });
            }
        }
    }
    Ok(ControlState { radios, ..Default::default() })
}

pub fn read_adapter(id: &str) -> Result<ControlState> {
    let (socket, mut request) = interface(id)?;
    if unsafe { libc::ioctl(socket.as_raw_fd(), libc::SIOCGIFFLAGS, &mut request) } != 0 {
        return Err(native_error());
    }
    let enabled = unsafe { request.ifr_ifru.ifru_flags } & libc::IFF_UP as i16 != 0;
    Ok(ControlState {
        adapters: vec![AdapterState { id: id.into(), enabled }],
        ..Default::default()
    })
}

pub fn apply(state: &ControlState, restoring: bool) -> Result<()> {
    for radio in &state.radios {
        let current = read_radio(radio.kind)?;
        let current = current
            .radios
            .iter()
            .find(|current| current.id == radio.id)
            .ok_or_else(|| Error::Probe("radio is no longer present".into()))?;
        if current.on == radio.on && !restoring {
            continue;
        }
        let connection = connection()?;
        if radio.kind == Radio::Bluetooth {
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
            proxy.set_property(if radio.kind == Radio::Wifi { "WirelessEnabled" } else { "WwanEnabled" }, radio.on).map_err(probe)?;
        }
    }
    for adapter in &state.adapters {
        if read_adapter(&adapter.id)?.adapters[0].enabled == adapter.enabled && !restoring {
            continue;
        }
        let (socket, mut request) = interface(&adapter.id)?;
        if unsafe { libc::ioctl(socket.as_raw_fd(), libc::SIOCGIFFLAGS, &mut request) } != 0 {
            return Err(native_error());
        }
        let flags = unsafe { request.ifr_ifru.ifru_flags };
        request.ifr_ifru.ifru_flags = if adapter.enabled { flags | libc::IFF_UP as i16 } else { flags & !(libc::IFF_UP as i16) };
        if unsafe { libc::ioctl(socket.as_raw_fd(), libc::SIOCSIFFLAGS, &request) } != 0 {
            return Err(native_error());
        }
    }
    if let Some(plan) = &state.power_plan {
        if crate::host::power::active_plan()?.id == *plan && !restoring {
            return Ok(());
        }
        let choices = fs::read_to_string("/sys/firmware/acpi/platform_profile_choices").map_err(probe)?;
        if !choices.split_whitespace().any(|choice| choice == plan) {
            return Err(Error::Probe("power profile is not available".into()));
        }
        fs::write("/sys/firmware/acpi/platform_profile", format!("{plan}\n")).map_err(probe)?;
    }
    for radio in &state.radios {
        if read_radio(radio.kind)?.radios.iter().find(|current| current.id == radio.id).is_none_or(|current| current.on != radio.on) {
            return Err(Error::Probe("radio did not reach the requested state".into()));
        }
    }
    for adapter in &state.adapters {
        if read_adapter(&adapter.id)?.adapters[0].enabled != adapter.enabled {
            return Err(Error::Probe("adapter did not reach the requested state".into()));
        }
    }
    if let Some(plan) = &state.power_plan
        && crate::host::power::active_plan()?.id != *plan
    {
        return Err(Error::Probe("power profile did not reach the requested state".into()));
    }
    Ok(())
}

pub fn radio_affects(kind: Radio, adapter: &crate::host::adapters::Adapter) -> Result<bool> {
    if kind != Radio::Bluetooth {
        return Ok(adapter.kind
            == if kind == Radio::Cellular {
                crate::host::adapters::AdapterKind::Cellular
            } else {
                crate::host::adapters::AdapterKind::Wifi
            });
    }
    let subsystem = fs::canonicalize(format!("/sys/class/net/{}/device/subsystem", adapter.name));
    Ok(adapter.name.starts_with("bnep")
        || subsystem.ok().and_then(|path| path.file_name().map(|name| name == "bluetooth")).unwrap_or(false))
}

pub fn connected_adapters(pid: u32) -> Result<Vec<String>> {
    let mut owned = BTreeSet::new();
    for entry in fs::read_dir(format!("/proc/{pid}/fd")).map_err(probe)? {
        let entry = entry.map_err(probe)?;
        let target = match fs::read_link(entry.path()) {
            Ok(target) => target,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
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
            let remote = socket_ip(fields[2])?;
            if remote.is_loopback() {
                continue;
            }
            locals.insert(socket_ip(fields[1])?.to_string());
        }
    }
    let inventory = crate::host::adapters::snapshot()?;
    Ok(inventory
        .adapters
        .into_iter()
        .filter(|adapter| adapter.ipv4.iter().chain(&adapter.ipv6).any(|address| locals.contains(&address.address)))
        .map(|adapter| adapter.id)
        .collect())
}

/* === Private Functions === */

fn probe(error: impl std::fmt::Display) -> Error {
    Error::Probe(error.to_string())
}
fn native_error() -> Error {
    probe(std::io::Error::last_os_error())
}
fn connection() -> Result<Connection> {
    Builder::system().map_err(probe)?.method_timeout(super::POLICY.attempt_timeout).build().map_err(probe)
}
fn interface(id: &str) -> Result<(OwnedFd, libc::ifreq)> {
    if id.is_empty() || id.len() >= libc::IFNAMSIZ || id.as_bytes().contains(&0) {
        return Err(Error::Probe("invalid adapter identity".into()));
    }
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
    let hex = socket.split_once(':').ok_or_else(|| Error::Probe("invalid native socket address".into()))?.0;
    if hex.len() == 8 {
        let value = u32::from_str_radix(hex, 16).map_err(probe)?;
        return Ok(IpAddr::V4(Ipv4Addr::from(value.to_ne_bytes())));
    }
    if hex.len() != 32 {
        return Err(Error::Probe("invalid IPv6 socket address".into()));
    }
    let mut bytes = [0u8; 16];
    for (index, word) in bytes.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        word.copy_from_slice(&u32::from_str_radix(&hex[index * 8..index * 8 + 8], 16).map_err(probe)?.to_ne_bytes());
    }
    let address = Ipv6Addr::from(bytes);
    Ok(address.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(IpAddr::V6(address)))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn socket_addresses() {
        assert_eq!(socket_ip("0100007F:1234").unwrap(), "127.0.0.1".parse::<IpAddr>().unwrap());
        assert_eq!(socket_ip("00000000000000000000000001000000:1234").unwrap(), "::1".parse::<IpAddr>().unwrap());
        assert!(socket_ip("invalid").is_err());
    }
}
