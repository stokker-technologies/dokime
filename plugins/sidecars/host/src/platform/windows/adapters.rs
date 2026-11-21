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
use std::collections::BTreeSet;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ptr::null_mut;
use std::time::{Duration, Instant};

// Third Party Imports
use windows::Win32::Foundation::{ERROR_BUFFER_OVERFLOW, ERROR_SUCCESS};
use windows::Win32::NetworkManagement::IpHelper::{
    FreeMibTable, GAA_FLAG_INCLUDE_ALL_INTERFACES, GAA_FLAG_INCLUDE_GATEWAYS, GAA_FLAG_INCLUDE_PREFIX, GetAdaptersAddresses, GetIfEntry2,
    GetIpForwardTable2, IF_TYPE_ETHERNET_CSMACD, IF_TYPE_IEEE80211, IF_TYPE_SOFTWARE_LOOPBACK, IF_TYPE_TUNNEL, IF_TYPE_WWANPP,
    IF_TYPE_WWANPP2, IP_ADAPTER_ADDRESSES_LH, IP_ADAPTER_DHCP_ENABLED, MIB_IF_ROW2,
};
use windows::Win32::NetworkManagement::Ndis::{
    IF_OPER_STATUS, IfOperStatusDormant, IfOperStatusDown, IfOperStatusLowerLayerDown, IfOperStatusNotPresent, IfOperStatusTesting,
    IfOperStatusUnknown, IfOperStatusUp,
};
use windows::Win32::Networking::WinSock::{AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR, SOCKADDR_IN, SOCKADDR_IN6};

// Local Imports
use crate::adapters::{Adapter, AdapterKind, AdapterStatus, Address, Snapshot};
use dokime::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Every Adapter Through the IP Helper API, Retrying a Growing Buffer Within the Probe Timeout.
pub fn snapshot(timeout: Duration) -> Result<Snapshot> {
    // u64 Storage Preserves the Native Structure's Alignment Across Buffer Resizes.
    let deadline = Instant::now() + timeout;
    let flags = GAA_FLAG_INCLUDE_GATEWAYS | GAA_FLAG_INCLUDE_PREFIX | GAA_FLAG_INCLUDE_ALL_INTERFACES;
    let mut bytes = 0;
    // SAFETY: A Null Buffer Asks Only for the Required Byte Size.
    unsafe { GetAdaptersAddresses(u32::from(AF_UNSPEC.0), flags, None, None, &mut bytes) };
    let mut buffer = vec![0u64; (bytes as usize).div_ceil(8)];
    loop {
        let first = buffer.as_mut_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>();
        // SAFETY: The Buffer Holds at Least `bytes` Bytes.
        let code = unsafe { GetAdaptersAddresses(u32::from(AF_UNSPEC.0), flags, None, Some(first), &mut bytes) };
        if code == ERROR_BUFFER_OVERFLOW.0 {
            if Instant::now() >= deadline {
                return Err(Error::Invalid("Read Adapters: Raise Network Probe Timeout; the Inventory Kept Growing".into()));
            }
            buffer.resize((bytes as usize).div_ceil(8), 0);
            continue;
        }
        if code != ERROR_SUCCESS.0 {
            return Err(probe(format!("GetAdaptersAddresses Returned {code}")));
        }
        let defaults = default_interfaces()?;
        let mut adapters = Vec::new();
        let mut current = first;
        while !current.is_null() {
            // SAFETY: All Linked Entries and Strings Belong to the Live API Buffer.
            let entry = unsafe { &*current };
            let id = unsafe { entry.AdapterName.to_string() }.map_err(probe)?.to_lowercase();
            let name = unsafe { entry.FriendlyName.to_string() }.map_err(probe)?;
            let description = unsafe { entry.Description.to_string() }.map_err(probe)?;

            // Unicast Addresses
            let mut ipv4 = Vec::new();
            let mut ipv6 = Vec::new();
            let mut unicast = entry.FirstUnicastAddress;
            while !unicast.is_null() {
                let address = unsafe { &*unicast };
                if let Some((text, scope)) = unsafe { socket_address(address.Address.lpSockaddr) } {
                    let value = Address {
                        address: text,
                        prefix: u32::from(address.OnLinkPrefixLength),
                        scope,
                    };
                    if scope.is_some() {
                        ipv6.push(value);
                    } else {
                        ipv4.push(value);
                    }
                }
                unicast = address.Next;
            }

            // Gateways and DNS Servers
            let mut gateways = Vec::new();
            let mut gateway = entry.FirstGatewayAddress;
            while !gateway.is_null() {
                let address = unsafe { &*gateway };
                if let Some((text, _)) = unsafe { socket_address(address.Address.lpSockaddr) } {
                    gateways.push(text);
                }
                gateway = address.Next;
            }
            let mut dns_servers = Vec::new();
            let mut dns = entry.FirstDnsServerAddress;
            while !dns.is_null() {
                let address = unsafe { &*dns };
                if let Some((text, _)) = unsafe { socket_address(address.Address.lpSockaddr) } {
                    dns_servers.push(text);
                }
                dns = address.Next;
            }

            // Kind: Ethernet Without a Hardware Interface Is Virtual
            let mut kind = match entry.IfType {
                IF_TYPE_ETHERNET_CSMACD => AdapterKind::Ethernet,
                IF_TYPE_IEEE80211 => AdapterKind::Wifi,
                IF_TYPE_WWANPP | IF_TYPE_WWANPP2 => AdapterKind::Cellular,
                IF_TYPE_SOFTWARE_LOOPBACK => AdapterKind::Loopback,
                IF_TYPE_TUNNEL => AdapterKind::Virtual,
                _ => AdapterKind::Other,
            };
            let mut interface = MIB_IF_ROW2 {
                InterfaceLuid: entry.Luid,
                ..Default::default()
            };
            let read = unsafe { GetIfEntry2(&mut interface) } == ERROR_SUCCESS;
            let hardware = interface.InterfaceAndOperStatusFlags._bitfield & 1 != 0;
            if read && !hardware && kind == AdapterKind::Ethernet {
                kind = AdapterKind::Virtual;
            }

            let length = (entry.PhysicalAddressLength as usize).min(entry.PhysicalAddress.len());
            let mac = (length != 0)
                .then(|| entry.PhysicalAddress[..length].iter().map(|byte| format!("{byte:02x}")).collect::<Vec<_>>().join(":"));
            let dhcp = unsafe { entry.Anonymous2.Flags } & IP_ADAPTER_DHCP_ENABLED != 0
                || unsafe { socket_address(entry.Dhcpv6Server.lpSockaddr) }.is_some();
            let primary = defaults.contains(&unsafe { entry.Anonymous1.Anonymous.IfIndex }) || defaults.contains(&entry.Ipv6IfIndex);
            adapters.push(Adapter {
                id,
                name,
                description,
                kind: kind.as_str().into(),
                mac,
                status: adapter_status(entry.OperStatus)?.as_str().into(),
                mtu: entry.Mtu,
                speed_bps: (entry.TransmitLinkSpeed > 0).then_some(entry.TransmitLinkSpeed),
                ipv4,
                ipv6,
                gateways,
                dns_servers,
                dhcp: Some(dhcp),
                primary,
            });
            current = entry.Next;
        }
        adapters.sort_by(|left, right| left.id.cmp(&right.id));
        return Ok(Snapshot { adapters });
    }
}

/// Text and IPv6 Scope of a Native Socket Address.
///
/// # Safety
///
/// The caller supplies a null pointer or a live native socket address from an IP Helper row.
pub unsafe fn socket_address(address: *const SOCKADDR) -> Option<(String, Option<u32>)> {
    if address.is_null() {
        return None;
    }
    match unsafe { (*address).sa_family } {
        AF_INET => {
            let address = unsafe { &*address.cast::<SOCKADDR_IN>() };
            let bytes = unsafe { address.sin_addr.S_un.S_addr }.to_ne_bytes();
            Some((Ipv4Addr::from(bytes).to_string(), None))
        }
        AF_INET6 => {
            let address = unsafe { &*address.cast::<SOCKADDR_IN6>() };
            let bytes = unsafe { address.sin6_addr.u.Byte };
            Some((Ipv6Addr::from(bytes).to_string(), Some(unsafe { address.Anonymous.sin6_scope_id })))
        }
        _ => None,
    }
}

/* === Private Functions === */

fn adapter_status(value: IF_OPER_STATUS) -> Result<AdapterStatus> {
    if value == IfOperStatusUp {
        return Ok(AdapterStatus::Up);
    }
    let down = [
        IfOperStatusDown,
        IfOperStatusTesting,
        IfOperStatusUnknown,
        IfOperStatusDormant,
        IfOperStatusNotPresent,
        IfOperStatusLowerLayerDown,
    ];
    if down.contains(&value) {
        return Ok(AdapterStatus::Down);
    }
    Err(Error::Invalid(format!("Read Adapters: Report the Unknown Adapter Status {}", value.0)))
}

fn default_interfaces() -> Result<BTreeSet<u32>> {
    let mut table = null_mut();
    // SAFETY: The API Allocates Its Table; It Is Freed After Reading Every Row.
    let code = unsafe { GetIpForwardTable2(AF_UNSPEC, &mut table) };
    if code != ERROR_SUCCESS {
        return Err(probe(format!("GetIpForwardTable2 Returned {}", code.0)));
    }
    let rows = unsafe { std::slice::from_raw_parts((*table).Table.as_ptr(), (*table).NumEntries as usize) };
    let indices = rows.iter().filter(|row| row.DestinationPrefix.PrefixLength == 0).map(|row| row.InterfaceIndex).collect();
    unsafe { FreeMibTable(table.cast()) };
    Ok(indices)
}

fn probe(error: impl std::fmt::Display) -> Error {
    Error::Invalid(format!("Read Adapters: Retry Once the Network Stack Answers: {error}"))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_status_maps_every_operational_state() {
        assert_eq!(adapter_status(IfOperStatusUp).unwrap(), AdapterStatus::Up);
        let down = [
            IfOperStatusDown,
            IfOperStatusTesting,
            IfOperStatusUnknown,
            IfOperStatusDormant,
            IfOperStatusNotPresent,
            IfOperStatusLowerLayerDown,
        ];
        for value in down {
            assert_eq!(adapter_status(value).unwrap(), AdapterStatus::Down);
        }
        assert!(adapter_status(IF_OPER_STATUS(99)).is_err());
    }

    #[test]
    fn native_addresses_print_with_their_scope() {
        let mut ipv4 = SOCKADDR_IN {
            sin_family: AF_INET,
            ..Default::default()
        };
        ipv4.sin_addr.S_un.S_addr = u32::from_ne_bytes([192, 0, 2, 9]);
        assert_eq!(unsafe { socket_address(std::ptr::addr_of!(ipv4).cast()) }, Some(("192.0.2.9".into(), None)));

        let mut ipv6 = SOCKADDR_IN6 {
            sin6_family: AF_INET6,
            ..Default::default()
        };
        ipv6.sin6_addr.u.Byte = Ipv6Addr::LOCALHOST.octets();
        ipv6.Anonymous.sin6_scope_id = 7;
        assert_eq!(unsafe { socket_address(std::ptr::addr_of!(ipv6).cast()) }, Some(("::1".into(), Some(7))));
        assert_eq!(unsafe { socket_address(std::ptr::null()) }, None);
    }
}
