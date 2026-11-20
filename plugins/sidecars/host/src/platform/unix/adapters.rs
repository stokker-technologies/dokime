/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Adapter Inventory

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::ffi::CStr;
use std::fs;
use std::io;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::ptr::null_mut;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// Third Party Imports

// Local Imports
use crate::adapters::{Adapter, AdapterKind, AdapterStatus, Address, Snapshot};
use dokime::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// The getifaddrs List, Freed Once on Drop.
struct List(*mut libc::ifaddrs);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Drop for List {
    fn drop(&mut self) {
        // SAFETY: The List Came From getifaddrs and Is Not Used Again.
        unsafe { libc::freeifaddrs(self.0) };
    }
}

/* === Public Functions === */

/// Every Adapter Through getifaddrs, sysfs and the Routing Tables, With NetworkManager DNS and DHCP Where It Manages the Link.
pub fn snapshot(timeout: Duration) -> Result<Snapshot> {
    let mut first = null_mut();
    // SAFETY: getifaddrs Owns the Linked List Until freeifaddrs in List::drop.
    if unsafe { libc::getifaddrs(&mut first) } != 0 {
        return Err(probe(io::Error::last_os_error()));
    }
    let _list = List(first);
    let mut adapters = BTreeMap::new();
    let mut current = first;
    while !current.is_null() {
        let entry = unsafe { &*current };
        let name = unsafe { CStr::from_ptr(entry.ifa_name) }.to_string_lossy().into_owned();
        let adapter = adapters.entry(name.clone()).or_insert_with(|| {
            let base = format!("/sys/class/net/{name}");
            let read = |file: &str| fs::read_to_string(format!("{base}/{file}")).ok().map(|text| text.trim().to_owned());
            let kind = if entry.ifa_flags & libc::IFF_LOOPBACK as u32 != 0 {
                AdapterKind::Loopback
            } else if Path::new(&format!("{base}/wireless")).exists() {
                AdapterKind::Wifi
            } else if !Path::new(&format!("{base}/device")).exists() {
                AdapterKind::Virtual
            } else {
                AdapterKind::Ethernet
            };
            let status = if entry.ifa_flags & libc::IFF_RUNNING as u32 != 0 { AdapterStatus::Up } else { AdapterStatus::Down };
            Adapter {
                id: name.clone(),
                name: name.clone(),
                description: name.clone(),
                kind: kind.as_str().into(),
                mac: read("address"),
                status: status.as_str().into(),
                mtu: read("mtu").and_then(|value| value.parse().ok()).unwrap_or(0),
                speed_bps: read("speed").and_then(|value| value.parse::<u64>().ok()).and_then(|speed| speed.checked_mul(1_000_000)),
                ipv4: Vec::new(),
                ipv6: Vec::new(),
                gateways: Vec::new(),
                dns_servers: Vec::new(),
                dhcp: None,
                primary: false,
            }
        });
        if !entry.ifa_addr.is_null() {
            match unsafe { (*entry.ifa_addr).sa_family as i32 } {
                libc::AF_INET => {
                    let ip = unsafe { &*entry.ifa_addr.cast::<libc::sockaddr_in>() };
                    let prefix = if entry.ifa_netmask.is_null() {
                        0
                    } else {
                        unsafe { (*entry.ifa_netmask.cast::<libc::sockaddr_in>()).sin_addr.s_addr }.count_ones()
                    };
                    adapter.ipv4.push(Address {
                        address: Ipv4Addr::from(ip.sin_addr.s_addr.to_ne_bytes()).to_string(),
                        prefix,
                        scope: None,
                    });
                }
                libc::AF_INET6 => {
                    let ip = unsafe { &*entry.ifa_addr.cast::<libc::sockaddr_in6>() };
                    let prefix = if entry.ifa_netmask.is_null() {
                        0
                    } else {
                        unsafe { (*entry.ifa_netmask.cast::<libc::sockaddr_in6>()).sin6_addr.s6_addr }
                            .iter()
                            .map(|byte| byte.count_ones())
                            .sum()
                    };
                    adapter.ipv6.push(Address {
                        address: Ipv6Addr::from(ip.sin6_addr.s6_addr).to_string(),
                        prefix,
                        scope: Some(ip.sin6_scope_id),
                    });
                }
                _ => {}
            }
        }
        current = entry.ifa_next;
    }
    read_routes(&mut adapters)?;
    network_manager(&mut adapters, timeout);
    Ok(Snapshot {
        adapters: adapters.into_values().collect(),
    })
}

/* === Private Functions === */

fn read_routes(adapters: &mut BTreeMap<String, Adapter>) -> Result<()> {
    let ipv4 = fs::read_to_string("/proc/net/route").map_err(probe)?;
    for line in ipv4.lines().skip(1) {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.len() < 8 {
            continue;
        }
        let Some(adapter) = adapters.get_mut(fields[0]) else {
            continue;
        };
        let flags = u32::from_str_radix(fields[3], 16).unwrap_or(0);
        if flags & 1 == 0 {
            continue;
        }
        adapter.primary |= fields[1] == "00000000" && fields[7] == "00000000";
        if flags & 2 != 0
            && let Ok(gateway) = u32::from_str_radix(fields[2], 16)
        {
            let address = Ipv4Addr::from(gateway.to_ne_bytes()).to_string();
            if !adapter.gateways.contains(&address) {
                adapter.gateways.push(address);
            }
        }
    }
    let ipv6 = fs::read_to_string("/proc/net/ipv6_route").map_err(probe)?;
    for line in ipv6.lines() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.len() < 10 {
            continue;
        }
        let Some(adapter) = adapters.get_mut(fields[9]) else {
            continue;
        };
        let flags = u32::from_str_radix(fields[8], 16).unwrap_or(0);
        if flags & 1 == 0 || flags & 0x200 != 0 {
            continue;
        }
        adapter.primary |= fields[0] == "00000000000000000000000000000000" && fields[1] == "00";
        if let Ok(gateway) = u128::from_str_radix(fields[4], 16)
            && gateway != 0
        {
            let address = Ipv6Addr::from(gateway).to_string();
            if !adapter.gateways.contains(&address) {
                adapter.gateways.push(address);
            }
        }
    }
    Ok(())
}

fn network_manager(adapters: &mut BTreeMap<String, Adapter>, timeout: Duration) {
    // NetworkManager Supplies Per-Interface DNS and DHCP Where It Manages the Link.
    let mut command = Command::new("nmcli");
    command.args([
        "--terse",
        "--escape",
        "no",
        "--fields",
        "GENERAL.DEVICE,GENERAL.TYPE,GENERAL.STATE,IP4.DNS,IP6.DNS,DHCP4.OPTION,DHCP6.OPTION",
        "device",
        "show",
    ]);
    let Ok(output) = run_bounded(command, timeout) else {
        return;
    };
    if !output.status.success() {
        return;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut selected = None;
    for line in text.lines() {
        let Some((field, value)) = line.split_once(':') else {
            continue;
        };
        if field == "GENERAL.DEVICE" {
            selected = Some(value.to_owned());
            continue;
        }
        let Some(adapter) = selected.as_ref().and_then(|name| adapters.get_mut(name)) else {
            continue;
        };
        if field == "GENERAL.TYPE" {
            let kind = match value {
                "wifi" => AdapterKind::Wifi,
                "ethernet" => AdapterKind::Ethernet,
                "gsm" | "cdma" => AdapterKind::Cellular,
                "loopback" => AdapterKind::Loopback,
                _ => AdapterKind::Virtual,
            };
            adapter.kind = kind.as_str().into();
        } else if field == "GENERAL.STATE" {
            // State 10 Is Unmanaged; Other States Have Known Active DHCP State.
            adapter.dhcp = if value.split_whitespace().next() == Some("10") { None } else { Some(false) };
        } else if field.starts_with("IP4.DNS[") || field.starts_with("IP6.DNS[") {
            adapter.dns_servers.push(value.to_owned());
        } else if field.starts_with("DHCP4.OPTION[") || field.starts_with("DHCP6.OPTION[") {
            adapter.dhcp = Some(true);
        }
    }
}

/// Run a Command to Completion Within the Timeout, Killing It When the Timeout Passes.
fn run_bounded(mut command: Command, timeout: Duration) -> io::Result<Output> {
    command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let child = command.spawn()?;
    let id = child.id();
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || sender.send(child.wait_with_output()));
    match receiver.recv_timeout(timeout) {
        Ok(output) => output,
        Err(_) => {
            // SAFETY: The Waiting Thread Still Owns the Unreaped Child, So the Identifier Names It.
            unsafe { libc::kill(id as libc::pid_t, libc::SIGKILL) };
            Err(io::Error::new(io::ErrorKind::TimedOut, "command timed out"))
        }
    }
}

fn probe(error: impl std::fmt::Display) -> Error {
    Error::Invalid(format!("Read Adapters: Retry Once the Network Stack Answers: {error}"))
}

/* === Tests === */
