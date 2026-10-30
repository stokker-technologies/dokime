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
    domain::controls::{AdapterState, ControlState, Radio, RadioPower, RadioState},
};
use std::{
    collections::BTreeSet,
    net::{Ipv4Addr, Ipv6Addr},
};
use windows::{
    Devices::{
        Enumeration::DeviceInformation,
        Radios::{Radio as NativeRadio, RadioAccessStatus, RadioKind, RadioState as NativeRadioState},
    },
    Win32::{
        Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS},
        NetworkManagement::{
            IpHelper::*,
            Ndis::{NET_LUID_LH, NdisPhysicalMediumBluetooth},
        },
        Networking::WinSock::{AF_INET, AF_INET6},
        Security::{CheckTokenMembership, CreateWellKnownSid, PSID, WinBuiltinAdministratorsSid},
        System::{
            Power::PowerSetActiveScheme,
            WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize},
        },
    },
    core::{BOOL, GUID},
};

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Apartment;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Apartment {
    fn new() -> Result<Self> {
        unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.map_err(probe)?;
        Ok(Self)
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe { RoUninitialize() };
    }
}

/* === Public Functions === */

pub fn administrator() -> Result<bool> {
    let mut sid = [0u64; 16];
    let mut bytes = std::mem::size_of_val(&sid) as u32;
    unsafe { CreateWellKnownSid(WinBuiltinAdministratorsSid, None, Some(PSID(sid.as_mut_ptr().cast())), &mut bytes) }.map_err(probe)?;
    let mut member = BOOL::default();
    unsafe { CheckTokenMembership(None, PSID(sid.as_mut_ptr().cast()), &mut member) }.map_err(probe)?;
    Ok(member.as_bool())
}

pub fn read_radio(kind: Radio) -> Result<ControlState> {
    let _apartment = Apartment::new()?;
    let selector = NativeRadio::GetDeviceSelector().map_err(probe)?;
    let devices = DeviceInformation::FindAllAsyncAqsFilter(&selector).map_err(probe)?.join().map_err(probe)?;
    let mut radios = Vec::new();
    for device in devices {
        let id = device.Id().map_err(probe)?;
        let radio = NativeRadio::FromIdAsync(&id).map_err(probe)?.join().map_err(probe)?;
        let actual = match radio.Kind().map_err(probe)? {
            RadioKind::WiFi => Radio::Wifi,
            RadioKind::Bluetooth => Radio::Bluetooth,
            RadioKind::MobileBroadband => Radio::Cellular,
            RadioKind::Other => continue,
            value => {
                return Err(Error::UnknownValue {
                    kind: "radio kind",
                    value: value.0.to_string(),
                });
            }
        };
        if actual != kind {
            continue;
        }
        let state = radio_power(radio.State().map_err(probe)?)?;
        radios.push(RadioState {
            id: id.to_string(),
            kind,
            on: state == RadioPower::On,
        });
    }
    Ok(ControlState { radios, ..Default::default() })
}

pub fn read_adapter(id: &str) -> Result<ControlState> {
    let enabled = crate::platform::windows::devices::adapter_enabled(id)?;
    Ok(ControlState {
        adapters: vec![AdapterState { id: id.into(), enabled }],
        ..Default::default()
    })
}

pub fn apply(state: &ControlState, restoring: bool) -> Result<()> {
    if !state.radios.is_empty() {
        let deadline = std::time::Instant::now() + super::POLICY.attempt_timeout / 2;
        let _apartment = Apartment::new()?;
        let access = NativeRadio::RequestAccessAsync().map_err(probe)?.join().map_err(probe)?;
        if access != RadioAccessStatus::Allowed {
            return Err(Error::Probe(format!("radio access refused: {}", access.0)));
        }
        for value in &state.radios {
            let radio = NativeRadio::FromIdAsync(&value.id.as_str().into()).map_err(probe)?.join().map_err(probe)?;
            let target = if value.on { NativeRadioState::On } else { NativeRadioState::Off };
            if radio.State().map_err(probe)? == target && !restoring {
                continue;
            }
            set_radio(&radio, target, deadline)?;
        }
    }
    for adapter in &state.adapters {
        if crate::platform::windows::devices::adapter_enabled(&adapter.id)? == adapter.enabled && !restoring {
            continue;
        }
        if !administrator()? {
            return Err(Error::Probe("host.network.adapter.set requires administrator".into()));
        }
        crate::platform::windows::devices::set_adapter(&adapter.id, adapter.enabled)?;
    }
    if let Some(plan) = &state.power_plan {
        if crate::host::power::active_plan()?.id == *plan && !restoring {
            return Ok(());
        }
        let guid = GUID::try_from(plan.as_str()).map_err(probe)?;
        unsafe { PowerSetActiveScheme(None, Some(&guid)) }.ok().map_err(probe)?;
        if crate::host::power::active_plan()?.id != *plan {
            return Err(Error::Probe("power plan did not reach the requested state".into()));
        }
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
    let guid = GUID::try_from(adapter.id.trim_matches(['{', '}'])).map_err(probe)?;
    let mut luid = NET_LUID_LH::default();
    unsafe { ConvertInterfaceGuidToLuid(&guid, &mut luid) }.ok().map_err(probe)?;
    let mut row = MIB_IF_ROW2 {
        InterfaceLuid: luid,
        ..Default::default()
    };
    unsafe { GetIfEntry2(&mut row) }.ok().map_err(probe)?;
    Ok(row.PhysicalMediumType == NdisPhysicalMediumBluetooth)
}

pub fn connected_adapters(pid: u32) -> Result<Vec<String>> {
    let mut locals = BTreeSet::new();
    for family in [AF_INET, AF_INET6] {
        let mut size = 0;
        unsafe { GetExtendedTcpTable(None, &mut size, false, family.0.into(), TCP_TABLE_OWNER_PID_ALL, 0) };
        let mut buffer = vec![0u64; (size as usize).div_ceil(8)];
        let deadline = std::time::Instant::now() + super::POLICY.attempt_timeout;
        loop {
            let code = unsafe {
                GetExtendedTcpTable(Some(buffer.as_mut_ptr().cast()), &mut size, false, family.0.into(), TCP_TABLE_OWNER_PID_ALL, 0)
            };
            if code == ERROR_INSUFFICIENT_BUFFER.0 && std::time::Instant::now() < deadline {
                buffer.resize((size as usize).div_ceil(8), 0);
                continue;
            }
            if code != ERROR_SUCCESS.0 {
                return Err(Error::Probe(format!("GetExtendedTcpTable: {code}")));
            }
            if family == AF_INET {
                let table = buffer.as_ptr().cast::<MIB_TCPTABLE_OWNER_PID>();
                let rows = unsafe {
                    std::slice::from_raw_parts(
                        std::ptr::addr_of!((*table).table).cast::<MIB_TCPROW_OWNER_PID>(),
                        (*table).dwNumEntries as usize,
                    )
                };
                for row in rows {
                    if row.dwOwningPid == pid && row.dwState == 5 && !Ipv4Addr::from(row.dwRemoteAddr.to_ne_bytes()).is_loopback() {
                        locals.insert(Ipv4Addr::from(row.dwLocalAddr.to_ne_bytes()).to_string());
                    }
                }
            } else {
                let table = buffer.as_ptr().cast::<MIB_TCP6TABLE_OWNER_PID>();
                let rows = unsafe {
                    std::slice::from_raw_parts(
                        std::ptr::addr_of!((*table).table).cast::<MIB_TCP6ROW_OWNER_PID>(),
                        (*table).dwNumEntries as usize,
                    )
                };
                for row in rows {
                    if row.dwOwningPid == pid && row.dwState == 5 && !loopback(Ipv6Addr::from(row.ucRemoteAddr)) {
                        locals.insert(ip_text(Ipv6Addr::from(row.ucLocalAddr)));
                    }
                }
            }
            break;
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

fn radio_power(state: NativeRadioState) -> Result<RadioPower> {
    match state {
        NativeRadioState::On => Ok(RadioPower::On),
        NativeRadioState::Off => Ok(RadioPower::Off),
        NativeRadioState::Disabled | NativeRadioState::Unknown => Err(Error::Probe("radio is disabled by hardware or policy".into())),
        value => Err(Error::UnknownValue {
            kind: "radio power",
            value: value.0.to_string(),
        }),
    }
}

fn set_radio(radio: &NativeRadio, target: NativeRadioState, deadline: std::time::Instant) -> Result<()> {
    use std::{sync::mpsc, time::Instant};
    use windows::{Foundation::TypedEventHandler, core::IInspectable};
    let (changed, changes) = mpsc::sync_channel(1);
    let handler = TypedEventHandler::<NativeRadio, IInspectable>::new(move |_, _| {
        let _ = changed.try_send(());
        Ok(())
    });
    let token = radio.StateChanged(&handler).map_err(probe)?;
    struct Watching {
        radio: NativeRadio,
        token: i64,
    }
    impl Drop for Watching {
        fn drop(&mut self) {
            let _ = self.radio.RemoveStateChanged(self.token);
        }
    }
    let _watching = Watching { radio: radio.clone(), token };
    let access = radio.SetStateAsync(target).map_err(probe)?.join().map_err(probe)?;
    if access != RadioAccessStatus::Allowed {
        return Err(Error::Probe(format!("radio access refused: {}", access.0)));
    }
    loop {
        if radio.State().map_err(probe)? == target {
            return Ok(());
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        changes
            .recv_timeout(remaining)
            .map_err(|error| Error::Probe(format!("radio did not settle within its operation policy: {error}")))?;
    }
}

fn loopback(address: Ipv6Addr) -> bool {
    address.is_loopback() || address.to_ipv4_mapped().is_some_and(|address| address.is_loopback())
}
fn ip_text(address: Ipv6Addr) -> String {
    address.to_ipv4_mapped().map(|address| address.to_string()).unwrap_or_else(|| address.to_string())
}

fn probe(error: impl std::fmt::Display) -> Error {
    Error::Probe(error.to_string())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_radio_power() {
        assert_eq!(radio_power(NativeRadioState::On).unwrap(), RadioPower::On);
        assert_eq!(radio_power(NativeRadioState::Off).unwrap(), RadioPower::Off);
        assert!(radio_power(NativeRadioState::Disabled).is_err());
        assert!(radio_power(NativeRadioState::Unknown).is_err());
        assert!(matches!(radio_power(NativeRadioState(99)), Err(Error::UnknownValue { kind: "radio power", .. })));
    }

    #[test]
    fn native_power_write() {
        let prior = crate::host::power::active_plan().unwrap();
        apply(
            &ControlState {
                power_plan: Some(prior.id.clone()),
                ..Default::default()
            },
            true,
        )
        .unwrap();
        assert_eq!(crate::host::power::active_plan().unwrap().id, prior.id);
    }
}
