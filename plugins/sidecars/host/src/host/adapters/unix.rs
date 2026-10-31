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
use std::{
    collections::BTreeMap,
    ffi::CStr,
    fs,
    net::{Ipv4Addr, Ipv6Addr},
    ptr::null_mut,
};

// Third Party Imports

// Local Imports
use super::{Adapter, AdapterKind, AdapterStatus, Address, Snapshot};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl dokime::harmos::Operation for super::Adapters {
    type Error = dokime::prelude::Refusal;

    async fn run(self, ctx: dokime::harmos::Context<'_, Self>) -> Result<dokime::harmos::Response<super::Snapshot>, Self::Error> {
        let timeout = std::time::Duration::from_millis(ctx.resource::<crate::HostOptions>().timing.network_probe_timeout_ms);
        let task = ctx.resource::<crate::Workers>().0.spawn_blocking(move || snapshot(timeout));
        let snapshot = dokime::plugin::refusal(dokime::plugin::refusal(task.await)?)?;
        Ok(dokime::harmos::Response::new(snapshot))
    }
}

/* === Public Functions === */

pub fn snapshot(timeout: std::time::Duration) -> crate::Result<Snapshot> {
    let mut first = null_mut();
    // SAFETY: getifaddrs Owns The Linked List Until freeifaddrs In List::drop.
    if unsafe { libc::getifaddrs(&mut first) } != 0 {
        return Err(crate::Error::Probe(std::io::Error::last_os_error().to_string()));
    }
    struct List(*mut libc::ifaddrs);
    impl Drop for List {
        fn drop(&mut self) {
            unsafe { libc::freeifaddrs(self.0) };
        }
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
            } else if std::path::Path::new(&format!("{base}/wireless")).exists() {
                AdapterKind::Wifi
            } else if !std::path::Path::new(&format!("{base}/device")).exists() {
                AdapterKind::Virtual
            } else {
                AdapterKind::Ethernet
            };
            Adapter {
                id: name.clone(),
                name: name.clone(),
                description: name.clone(),
                kind,
                mac: read("address"),
                status: if entry.ifa_flags & libc::IFF_RUNNING as u32 != 0 { AdapterStatus::Up } else { AdapterStatus::Down },
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

fn read_routes(adapters: &mut BTreeMap<String, Adapter>) -> crate::Result<()> {
    let ipv4 = fs::read_to_string("/proc/net/route").map_err(|error| crate::Error::Probe(error.to_string()))?;
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
    let ipv6 = fs::read_to_string("/proc/net/ipv6_route").map_err(|error| crate::Error::Probe(error.to_string()))?;
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

fn network_manager(adapters: &mut BTreeMap<String, Adapter>, timeout: std::time::Duration) {
    // NetworkManager Supplies Per-Interface DNS And DHCP Where It Manages The Link.
    let mut command = dokime_domain::process::command("nmcli");
    command.args([
        "--terse",
        "--escape",
        "no",
        "--fields",
        "GENERAL.DEVICE,GENERAL.TYPE,GENERAL.STATE,IP4.DNS,IP6.DNS,DHCP4.OPTION,DHCP6.OPTION",
        "device",
        "show",
    ]);
    let Ok(output) = crate::collectors::run_command_bounded(command, timeout) else {
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
            adapter.kind = match value {
                "wifi" => AdapterKind::Wifi,
                "ethernet" => AdapterKind::Ethernet,
                "gsm" | "cdma" => AdapterKind::Cellular,
                "loopback" => AdapterKind::Loopback,
                _ => AdapterKind::Virtual,
            };
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

/* === Tests === */
