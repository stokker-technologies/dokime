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
    ffi::CStr,
    io,
    net::{Ipv4Addr, Ipv6Addr},
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
};

// Third Party Imports

// Local Imports
use super::Change;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct NativeChange {
    kind: &'static str,
    index: u32,
    name: Option<String>,
    address: Option<String>,
}

pub struct Notifications {
    socket: OwnedFd,
    links: std::collections::BTreeMap<String, bool>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Notifications {
    pub fn new() -> crate::Result<Self> {
        let links = crate::host::adapters::snapshot()?
            .adapters
            .into_iter()
            .map(|adapter| (adapter.id, adapter.status == crate::host::adapters::AdapterStatus::Up))
            .collect();
        let descriptor =
            unsafe { libc::socket(libc::AF_NETLINK, libc::SOCK_RAW | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK, libc::NETLINK_ROUTE) };
        if descriptor < 0 {
            return Err(native_error());
        }
        let socket = unsafe { OwnedFd::from_raw_fd(descriptor) };
        let mut address: libc::sockaddr_nl = unsafe { std::mem::zeroed() };
        address.nl_family = libc::AF_NETLINK as u16;
        address.nl_groups =
            (libc::RTMGRP_LINK | libc::RTMGRP_IPV4_IFADDR | libc::RTMGRP_IPV6_IFADDR | libc::RTMGRP_IPV4_ROUTE | libc::RTMGRP_IPV6_ROUTE)
                as u32;
        let code = unsafe {
            libc::bind(socket.as_raw_fd(), (&address as *const libc::sockaddr_nl).cast(), std::mem::size_of_val(&address) as u32)
        };
        if code != 0 {
            return Err(native_error());
        }
        Ok(Self { socket, links })
    }

    pub fn drain(&mut self) -> crate::Result<Vec<Change>> {
        let mut events = Vec::new();
        let capacity = <crate::Network as dokime::StreamSpec>::WINDOW.saturating_sub(1);
        let mut datagrams = 0;
        // Peek The Datagram Size So A Large Kernel Notification Is Never Truncated.
        loop {
            if datagrams == capacity {
                return Err(crate::Error::Probe("netlink notification burst exceeds the stream window".into()));
            }
            let size = unsafe {
                libc::recv(self.socket.as_raw_fd(), std::ptr::null_mut(), 0, libc::MSG_PEEK | libc::MSG_TRUNC | libc::MSG_DONTWAIT)
            };
            if size < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::WouldBlock {
                    return Ok(events);
                }
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(crate::Error::Probe(format!("netlink notification receive: {error}")));
            }
            if size == 0 {
                return Err(crate::Error::Probe("netlink notification socket closed".into()));
            }
            let mut bytes = vec![0u8; size as usize];
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
                    None => interface_name(event.index)?,
                };
                if event.kind == "link_up" || event.kind == "link_down" {
                    let up = event.kind == "link_up";
                    if self.links.insert(name.clone(), up) == Some(up) {
                        continue;
                    }
                }
                events.push(Change::new(event.kind, name, event.address));
            }
            if events.len() > capacity {
                return Err(crate::Error::Probe("netlink notification burst exceeds the stream window".into()));
            }
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn native_error() -> crate::Error {
    crate::Error::Probe(io::Error::last_os_error().to_string())
}

fn interface_name(index: u32) -> crate::Result<String> {
    let mut name = [0 as libc::c_char; libc::IF_NAMESIZE];
    if unsafe { libc::if_indextoname(index, name.as_mut_ptr()) }.is_null() {
        // A Deleted Interface Has No Current Name; Preserve Its Kernel Identity.
        return Ok(index.to_string());
    }
    Ok(unsafe { CStr::from_ptr(name.as_ptr()) }.to_string_lossy().into_owned())
}

fn decode(mut bytes: &[u8], events: &mut Vec<NativeChange>) -> crate::Result<()> {
    while !bytes.is_empty() {
        if bytes.len() < 16 {
            return Err(crate::Error::Probe("truncated netlink header".into()));
        }
        let length = u32::from_ne_bytes(bytes[..4].try_into().unwrap()) as usize;
        if length < 16 || length > bytes.len() {
            return Err(crate::Error::Probe("invalid netlink notification length".into()));
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
                // IFA_LOCAL Wins For Point-To-Point IPv4; IFA_ADDRESS Carries IPv6.
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
                            let index = u32::from_ne_bytes(value.try_into().unwrap());
                            events.push(NativeChange {
                                kind: "default_route_changed",
                                index,
                                name: None,
                                address: None,
                            });
                        }
                    }
                }
            }
            2 | 4 => return Err(crate::Error::Probe("netlink reported an error or notification overrun".into())),
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

fn attributes(mut bytes: &[u8]) -> crate::Result<Vec<(u16, &[u8])>> {
    let mut fields = Vec::new();
    while bytes.len() >= 4 {
        let length = u16::from_ne_bytes(bytes[..2].try_into().unwrap()) as usize;
        let kind = u16::from_ne_bytes(bytes[2..4].try_into().unwrap());
        if length < 4 || length > bytes.len() {
            return Err(crate::Error::Probe("invalid netlink attribute".into()));
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
