/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Network Connectivity Through NetworkManager and Netlink Change Notifications

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::ffi::CStr;
use std::io;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::time::Duration;

// Third Party Imports

// Local Imports
use crate::collectors::network::{Change, PENDING_CHANGES, Pending};
use crate::platform::command::{bounded_output, command};
use dokime::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Link, Address and Default-Route Changes Read From a Nonblocking Netlink Route Socket.
pub struct Notifications {
    socket: OwnedFd,
    pending: Pending,
}

struct NativeChange {
    kind: &'static str,
    index: u32,
    name: Option<String>,
    address: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Notifications {
    /// Watch Changes From the Given Link States On; a Link Change Repeating Its Known State Is No Change.
    pub fn new(links: BTreeMap<String, bool>) -> Result<Self> {
        // SAFETY: Creating a socket has no preconditions; the descriptor is owned immediately below.
        let descriptor =
            unsafe { libc::socket(libc::AF_NETLINK, libc::SOCK_RAW | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK, libc::NETLINK_ROUTE) };
        if descriptor < 0 {
            return Err(native_error());
        }
        // SAFETY: The descriptor is valid and owned by nothing else.
        let socket = unsafe { OwnedFd::from_raw_fd(descriptor) };
        // SAFETY: An all-zero netlink address is valid before its fields are set.
        let mut address: libc::sockaddr_nl = unsafe { std::mem::zeroed() };
        address.nl_family = libc::AF_NETLINK as u16;
        address.nl_groups =
            (libc::RTMGRP_LINK | libc::RTMGRP_IPV4_IFADDR | libc::RTMGRP_IPV6_IFADDR | libc::RTMGRP_IPV4_ROUTE | libc::RTMGRP_IPV6_ROUTE)
                as u32;
        // SAFETY: The address is a live sockaddr_nl of the given size.
        let code = unsafe {
            libc::bind(socket.as_raw_fd(), (&address as *const libc::sockaddr_nl).cast(), std::mem::size_of_val(&address) as u32)
        };
        if code != 0 {
            return Err(native_error());
        }
        Ok(Self { socket, pending: Pending::new(links) })
    }

    /// Seed Unobserved Links From the First Configured Collection Probe.
    pub fn seed(&mut self, links: BTreeMap<String, bool>) {
        self.pending.seed(links);
    }

    /// Read Every Waiting Datagram, Then Take the Oldest Queued Change.
    pub fn take(&mut self) -> Result<Option<Change>> {
        let mut datagrams = 0;
        // Peek the Datagram Size So a Large Kernel Notification Is Never Truncated.
        loop {
            if datagrams == PENDING_CHANGES {
                self.pending.fail();
                return self.pending.pop();
            }
            // SAFETY: A zero-length peek writes nothing.
            let size = unsafe {
                libc::recv(self.socket.as_raw_fd(), std::ptr::null_mut(), 0, libc::MSG_PEEK | libc::MSG_TRUNC | libc::MSG_DONTWAIT)
            };
            if size < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::WouldBlock {
                    return self.pending.pop();
                }
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(Error::Invalid(format!("Watch Network Changes: Netlink Receive Failed ({error}); Restart the Plugin")));
            }
            if size == 0 {
                return Err(Error::Invalid("Watch Network Changes: the Netlink Socket Closed; Restart the Plugin".into()));
            }
            let mut bytes = vec![0u8; size as usize];
            // SAFETY: The buffer holds `bytes.len()` writable bytes.
            let received = unsafe { libc::recv(self.socket.as_raw_fd(), bytes.as_mut_ptr().cast(), bytes.len(), libc::MSG_DONTWAIT) };
            if received < 0 {
                return Err(native_error());
            }
            datagrams += 1;
            let mut decoded = Vec::new();
            decode(&bytes[..received as usize], &mut decoded)?;
            for event in decoded {
                let name = match event.name {
                    Some(name) => name,
                    None => interface_name(event.index),
                };
                let up = matches!(event.kind, "link_up" | "link_down").then_some(event.kind == "link_up");
                self.pending.push(Change::new(event.kind, name, event.address), up);
            }
        }
    }
}

/* === Public Functions === */

/// NetworkManager's Last Connectivity Verdict, Without Own Traffic.
pub fn internet(timeout: Duration) -> Option<bool> {
    let mut command = command("nmcli");
    command.args(["networking", "connectivity"]);
    let output = bounded_output(command, timeout).ok()?;
    if !output.status.success() {
        return None;
    }
    parse_connectivity(&String::from_utf8_lossy(&output.stdout))
}

/* === Private Functions === */

fn native_error() -> Error {
    Error::Invalid(format!("Watch Network Changes: {}; Restart the Plugin", io::Error::last_os_error()))
}

fn parse_connectivity(state: &str) -> Option<bool> {
    match state.trim() {
        "full" => Some(true),
        "none" | "portal" | "limited" => Some(false),
        _ => None,
    }
}

fn interface_name(index: u32) -> String {
    let mut name = [0 as libc::c_char; libc::IF_NAMESIZE];
    // SAFETY: The buffer holds IF_NAMESIZE bytes, as the call requires.
    if unsafe { libc::if_indextoname(index, name.as_mut_ptr()) }.is_null() {
        // A Deleted Interface Has No Current Name; Preserve Its Kernel Identity.
        return index.to_string();
    }
    // SAFETY: A successful call wrote a NUL-terminated name.
    unsafe { CStr::from_ptr(name.as_ptr()) }.to_string_lossy().into_owned()
}

fn decode(mut bytes: &[u8], events: &mut Vec<NativeChange>) -> Result<()> {
    let invalid = |problem: &str| Error::Invalid(format!("Watch Network Changes: {problem}; Restart the Plugin"));
    while !bytes.is_empty() {
        if bytes.len() < 16 {
            return Err(invalid("Truncated Netlink Header"));
        }
        let length = u32::from_ne_bytes(bytes[..4].try_into().unwrap()) as usize;
        if length < 16 || length > bytes.len() {
            return Err(invalid("Invalid Netlink Notification Length"));
        }
        let kind = u16::from_ne_bytes(bytes[4..6].try_into().unwrap());
        let payload = &bytes[16..length];
        match kind {
            libc::RTM_NEWLINK | libc::RTM_DELLINK if payload.len() >= 16 => {
                let index = u32::from_ne_bytes(payload[4..8].try_into().unwrap());
                let flags = u32::from_ne_bytes(payload[8..12].try_into().unwrap());
                let up = kind != libc::RTM_DELLINK && flags & libc::IFF_RUNNING as u32 != 0;
                let name = attributes(&payload[16..])?
                    .into_iter()
                    .find(|(kind, _)| *kind == 3)
                    .map(|(_, value)| String::from_utf8_lossy(value).trim_end_matches('\0').to_owned());
                events.push(NativeChange {
                    kind: if up { "link_up" } else { "link_down" },
                    index,
                    name,
                    address: None,
                });
            }
            libc::RTM_NEWADDR | libc::RTM_DELADDR if payload.len() >= 8 => {
                let index = u32::from_ne_bytes(payload[4..8].try_into().unwrap());
                let fields = attributes(&payload[8..])?;
                // IFA_LOCAL Wins for Point-to-Point IPv4; IFA_ADDRESS Carries IPv6.
                let address = fields.iter().find(|(kind, _)| *kind == 2).or_else(|| fields.iter().find(|(kind, _)| *kind == 1));
                let address = address.and_then(|(_, value)| match (payload[0] as i32, value.len()) {
                    (libc::AF_INET, 4) => Some(Ipv4Addr::from(<[u8; 4]>::try_from(*value).unwrap()).to_string()),
                    (libc::AF_INET6, 16) => Some(Ipv6Addr::from(<[u8; 16]>::try_from(*value).unwrap()).to_string()),
                    _ => None,
                });
                events.push(NativeChange {
                    kind: if kind == libc::RTM_DELADDR { "address_removed" } else { "address_added" },
                    index,
                    name: None,
                    address,
                });
            }
            libc::RTM_NEWROUTE | libc::RTM_DELROUTE if payload.len() >= 12 && payload[1] == 0 => {
                if payload[4] == libc::RT_TABLE_MAIN && payload[7] == libc::RTN_UNICAST {
                    for (kind, value) in attributes(&payload[12..])? {
                        if kind == 4 && value.len() == 4 {
                            events.push(NativeChange {
                                kind: "default_route_changed",
                                index: u32::from_ne_bytes(value.try_into().unwrap()),
                                name: None,
                                address: None,
                            });
                        }
                    }
                }
            }
            2 | 4 => return Err(invalid("Netlink Reported an Error or Notification Overrun")),
            _ => {}
        }
        let aligned = (length + 3) & !3;
        if aligned > bytes.len() {
            break;
        }
        bytes = &bytes[aligned..];
    }
    Ok(())
}

fn attributes(mut bytes: &[u8]) -> Result<Vec<(u16, &[u8])>> {
    let mut fields = Vec::new();
    while bytes.len() >= 4 {
        let length = u16::from_ne_bytes(bytes[..2].try_into().unwrap()) as usize;
        let kind = u16::from_ne_bytes(bytes[2..4].try_into().unwrap());
        if length < 4 || length > bytes.len() {
            return Err(Error::Invalid("Watch Network Changes: Invalid Netlink Attribute; Restart the Plugin".into()));
        }
        fields.push((kind, &bytes[4..length]));
        let aligned = (length + 3) & !3;
        if aligned > bytes.len() {
            break;
        }
        bytes = &bytes[aligned..];
    }
    Ok(fields)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_full_connectivity_is_internet() {
        assert_eq!(parse_connectivity("full\n"), Some(true));
        assert_eq!(parse_connectivity("portal"), Some(false));
        assert_eq!(parse_connectivity("unknown"), None);
    }

    #[test]
    fn netlink_notifications() {
        fn message(kind: u16, payload: Vec<u8>) -> Vec<u8> {
            let mut bytes = ((16 + payload.len()) as u32).to_ne_bytes().to_vec();
            bytes.extend_from_slice(&kind.to_ne_bytes());
            bytes.extend_from_slice(&[0; 10]);
            bytes.extend(payload);
            bytes
        }
        let mut link = vec![0; 16];
        link[4..8].copy_from_slice(&9u32.to_ne_bytes());
        link[8..12].copy_from_slice(&(libc::IFF_RUNNING as u32).to_ne_bytes());
        link[12..16].copy_from_slice(&(libc::IFF_RUNNING as u32).to_ne_bytes());
        link.extend_from_slice(&9u16.to_ne_bytes());
        link.extend_from_slice(&3u16.to_ne_bytes());
        link.extend_from_slice(b"eth0\0\0\0\0");
        let mut address = vec![libc::AF_INET as u8, 24, 0, 0];
        address.extend_from_slice(&9u32.to_ne_bytes());
        address.extend_from_slice(&8u16.to_ne_bytes());
        address.extend_from_slice(&2u16.to_ne_bytes());
        address.extend_from_slice(&[192, 0, 2, 9]);
        let mut route = vec![libc::AF_INET as u8, 0, 0, 0, libc::RT_TABLE_MAIN, 4, 0, libc::RTN_UNICAST, 0, 0, 0, 0];
        route.extend_from_slice(&8u16.to_ne_bytes());
        route.extend_from_slice(&4u16.to_ne_bytes());
        route.extend_from_slice(&9u32.to_ne_bytes());
        let bytes = [message(libc::RTM_NEWLINK, link), message(libc::RTM_DELADDR, address), message(libc::RTM_NEWROUTE, route)].concat();
        let mut changes = Vec::new();
        decode(&bytes, &mut changes).unwrap();
        assert_eq!(
            changes.iter().map(|change| (change.kind, change.index)).collect::<Vec<_>>(),
            [("link_up", 9), ("address_removed", 9), ("default_route_changed", 9)]
        );
        assert_eq!(changes[0].name.as_deref(), Some("eth0"));
        assert_eq!(changes[1].address.as_deref(), Some("192.0.2.9"));
        assert!(decode(&bytes[..10], &mut Vec::new()).is_err());
        assert!(attributes(&[0, 0, 0, 0]).is_err());
    }
}
