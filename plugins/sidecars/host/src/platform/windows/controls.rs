/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Radio, Adapter and Power Plan Reads and Writes

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeSet;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ptr::null_mut;
use std::sync::mpsc;
use std::time::Instant;

// Third Party Imports
use windows::Devices::Enumeration::DeviceInformation;
use windows::Devices::Radios::{RadioAccessStatus, RadioKind};
use windows::Foundation::TypedEventHandler;
use windows::Win32::Devices::DeviceAndDriverInstallation::{
    CM_DEVNODE_STATUS_FLAGS, CM_Get_DevNode_Status, CM_PROB, CM_PROB_DISABLED, CR_SUCCESS, DI_QUIETINSTALL, DICS_DISABLE, DICS_ENABLE,
    DICS_FLAG_GLOBAL, DIF_PROPERTYCHANGE, DIGCF_PRESENT, DIREG_DRV, GUID_DEVCLASS_NET, HDEVINFO, SP_CLASSINSTALL_HEADER, SP_DEVINFO_DATA,
    SP_DEVINSTALL_PARAMS_W, SP_PROPCHANGE_PARAMS, SetupDiCallClassInstaller, SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo,
    SetupDiGetClassDevsW, SetupDiGetDeviceInstallParamsW, SetupDiOpenDevRegKey, SetupDiSetClassInstallParamsW,
    SetupDiSetDeviceInstallParamsW,
};
use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_NO_MORE_ITEMS, ERROR_SUCCESS, HLOCAL, LocalFree};
use windows::Win32::NetworkManagement::IpHelper::{
    ConvertInterfaceGuidToLuid, GetExtendedTcpTable, GetIfEntry2, MIB_IF_ROW2, MIB_TCP6ROW_OWNER_PID, MIB_TCP6TABLE_OWNER_PID,
    MIB_TCPROW_OWNER_PID, MIB_TCPTABLE_OWNER_PID, TCP_TABLE_OWNER_PID_ALL,
};
use windows::Win32::NetworkManagement::Ndis::{NET_LUID_LH, NdisPhysicalMediumBluetooth};
use windows::Win32::Networking::WinSock::{AF_INET, AF_INET6};
use windows::Win32::Security::{CheckTokenMembership, CreateWellKnownSid, PSID, WinBuiltinAdministratorsSid};
use windows::Win32::System::Power::{PowerGetActiveScheme, PowerReadFriendlyName, PowerSetActiveScheme};
use windows::Win32::System::Registry::{KEY_QUERY_VALUE, RegCloseKey, RegQueryValueExW};
use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize};
use windows::core::{BOOL, GUID, HRESULT, IInspectable, w};

// Local Imports
use crate::adapters::Adapter;
use crate::controls::Timing;
use crate::domain::controls::{ControlState, Radio, RadioPower, RadioState};
use dokime::{Error, Result};

/* === Constants === */

/// MIB_TCP_STATE_ESTAB: an Established TCP Connection.
const TCP_ESTABLISHED: u32 = 5;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// A Windows Runtime Apartment Joined for the Current Thread.
struct Apartment;

/// A Device Information Set, Destroyed on Drop.
struct Devices(HDEVINFO);

/// A Radio State Change Registration, Removed on Drop.
struct Watching {
    radio: windows::Devices::Radios::Radio,
    token: i64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Apartment {
    fn new() -> Result<Self> {
        crate::platform::windows::apartment::retain()?;
        // SAFETY: Paired With RoUninitialize in Drop on the Same Thread.
        unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.map_err(probe)?;
        Ok(Self)
    }
}

impl Drop for Apartment {
    fn drop(&mut self) {
        // SAFETY: The Thread Joined the Apartment in Apartment::new.
        unsafe { RoUninitialize() };
    }
}

impl Drop for Devices {
    fn drop(&mut self) {
        // SAFETY: The Set Came From SetupDiGetClassDevsW and Is Not Used Again.
        let _ = unsafe { SetupDiDestroyDeviceInfoList(self.0) };
    }
}

impl Drop for Watching {
    fn drop(&mut self) {
        let _ = self.radio.RemoveStateChanged(self.token);
    }
}

/* === Public Functions === */

/// Whether the Process Runs in the Built-In Administrators Group.
pub fn administrator() -> Result<bool> {
    let mut sid = [0u64; 16];
    let mut bytes = std::mem::size_of_val(&sid) as u32;
    // SAFETY: The SID Buffer Holds `bytes` Bytes.
    unsafe { CreateWellKnownSid(WinBuiltinAdministratorsSid, None, Some(PSID(sid.as_mut_ptr().cast())), &mut bytes) }.map_err(probe)?;
    let mut member = BOOL::default();
    // SAFETY: The SID Was Written Above.
    unsafe { CheckTokenMembership(None, PSID(sid.as_mut_ptr().cast()), &mut member) }.map_err(probe)?;
    Ok(member.as_bool())
}

/// Every Radio of One Kind With Whether It Is On.
pub fn read_radio(kind: Radio, _: &Timing) -> Result<ControlState> {
    let _apartment = Apartment::new()?;
    let selector = windows::Devices::Radios::Radio::GetDeviceSelector().map_err(probe)?;
    let devices = DeviceInformation::FindAllAsyncAqsFilter(&selector).map_err(probe)?.join().map_err(probe)?;
    let mut radios = Vec::new();
    for device in devices {
        let id = device.Id().map_err(probe)?;
        let radio = windows::Devices::Radios::Radio::FromIdAsync(&id).map_err(probe)?.join().map_err(probe)?;
        let actual = match radio.Kind().map_err(probe)? {
            RadioKind::WiFi => Radio::Wifi,
            RadioKind::Bluetooth => Radio::Bluetooth,
            RadioKind::MobileBroadband => Radio::Cellular,
            RadioKind::Other => continue,
            value => return Err(Error::Invalid(format!("Read Radio: Report the Unknown Radio Kind {}", value.0))),
        };
        if actual != kind {
            continue;
        }
        let power = radio_power(radio.State().map_err(probe)?)?;
        radios.push(RadioState {
            id: id.to_string(),
            kind: kind.as_str().into(),
            on: power == RadioPower::On,
        });
    }
    Ok(ControlState { radios, ..Default::default() })
}

/// Whether the Adapter's Device Node Is Enabled.
pub fn read_adapter(id: &str) -> Result<bool> {
    let devices = open()?;
    let device = adapter_device(&devices, id)?;
    let mut status = CM_DEVNODE_STATUS_FLAGS::default();
    let mut problem = CM_PROB::default();
    // SAFETY: The Device Instance Belongs to the Live Information Set.
    let code = unsafe { CM_Get_DevNode_Status(&mut status, &mut problem, device.DevInst, 0) };
    if code != CR_SUCCESS {
        return Err(probe(format!("CM_Get_DevNode_Status Returned {}", code.0)));
    }
    Ok(problem != CM_PROB_DISABLED)
}

/// Write Each Radio, Adapter and Power Plan of the State; Restoring Writes Even Targets That Already Read as Requested.
pub fn apply(state: &ControlState, restoring: bool, timing: &Timing) -> Result<()> {
    if !state.radios.is_empty() {
        let deadline = Instant::now() + timing.radio_settle;
        let _apartment = Apartment::new()?;
        let access = windows::Devices::Radios::Radio::RequestAccessAsync().map_err(probe)?.join().map_err(probe)?;
        if access != RadioAccessStatus::Allowed {
            return Err(Error::Invalid(format!("Set Radio: Allow Radio Access for the Host Plugin; Windows Refused With {}", access.0)));
        }
        for value in &state.radios {
            let radio = windows::Devices::Radios::Radio::FromIdAsync(&value.id.as_str().into()).map_err(probe)?.join().map_err(probe)?;
            let target = if value.on { windows::Devices::Radios::RadioState::On } else { windows::Devices::Radios::RadioState::Off };
            if radio.State().map_err(probe)? == target && !restoring {
                continue;
            }
            set_radio(&radio, target, deadline)?;
        }
    }
    for adapter in &state.adapters {
        if read_adapter(&adapter.id)? == adapter.enabled && !restoring {
            continue;
        }
        if !administrator()? {
            return Err(Error::Invalid("Set Adapter: Run the Host Plugin as Administrator to Change an Adapter".into()));
        }
        set_adapter(&adapter.id, adapter.enabled)?;
    }
    if let Some(plan) = &state.power_plan {
        if crate::power::active_plan()?.id == *plan && !restoring {
            return Ok(());
        }
        let guid = GUID::try_from(plan.as_str()).map_err(probe)?;
        // SAFETY: The GUID Outlives the Call.
        unsafe { PowerSetActiveScheme(None, Some(&guid)) }.ok().map_err(probe)?;
        if crate::power::active_plan()?.id != *plan {
            return Err(Error::Invalid("Set Power Plan: Check the Power Policy; the Plan Did Not Become Active".into()));
        }
    }
    Ok(())
}

/// Whether the Adapter Is Carried Over Bluetooth.
pub fn bluetooth_adapter(adapter: &Adapter) -> Result<bool> {
    let guid = GUID::try_from(adapter.id.trim_matches(['{', '}'])).map_err(probe)?;
    let mut luid = NET_LUID_LH::default();
    // SAFETY: Both Values Outlive the Call.
    unsafe { ConvertInterfaceGuidToLuid(&guid, &mut luid) }.ok().map_err(probe)?;
    let mut row = MIB_IF_ROW2 {
        InterfaceLuid: luid,
        ..Default::default()
    };
    // SAFETY: The Row Names Its Interface by LUID.
    unsafe { GetIfEntry2(&mut row) }.ok().map_err(probe)?;
    Ok(row.PhysicalMediumType == NdisPhysicalMediumBluetooth)
}

/// The Local Addresses of the Process's Established Non-Loopback TCP Connections.
pub fn connected_addresses(pid: u32, timing: &Timing) -> Result<BTreeSet<String>> {
    let mut locals = BTreeSet::new();
    for family in [AF_INET, AF_INET6] {
        let mut size = 0;
        // SAFETY: A Null Table Asks Only for the Required Byte Size.
        unsafe { GetExtendedTcpTable(None, &mut size, false, family.0.into(), TCP_TABLE_OWNER_PID_ALL, 0) };
        let mut buffer = vec![0u64; (size as usize).div_ceil(8)];
        let deadline = Instant::now() + timing.control_probe;
        loop {
            // SAFETY: The Buffer Holds at Least `size` Bytes.
            let code = unsafe {
                GetExtendedTcpTable(Some(buffer.as_mut_ptr().cast()), &mut size, false, family.0.into(), TCP_TABLE_OWNER_PID_ALL, 0)
            };
            if code == ERROR_INSUFFICIENT_BUFFER.0 && Instant::now() < deadline {
                buffer.resize((size as usize).div_ceil(8), 0);
                continue;
            }
            if code != ERROR_SUCCESS.0 {
                return Err(probe(format!("GetExtendedTcpTable Returned {code}")));
            }
            if family == AF_INET {
                let table = buffer.as_ptr().cast::<MIB_TCPTABLE_OWNER_PID>();
                // SAFETY: The API Wrote dwNumEntries Rows Into the Buffer.
                let rows = unsafe {
                    std::slice::from_raw_parts(
                        std::ptr::addr_of!((*table).table).cast::<MIB_TCPROW_OWNER_PID>(),
                        (*table).dwNumEntries as usize,
                    )
                };
                for row in rows {
                    let remote = Ipv4Addr::from(row.dwRemoteAddr.to_ne_bytes());
                    if row.dwOwningPid == pid && row.dwState == TCP_ESTABLISHED && !remote.is_loopback() {
                        locals.insert(Ipv4Addr::from(row.dwLocalAddr.to_ne_bytes()).to_string());
                    }
                }
            } else {
                let table = buffer.as_ptr().cast::<MIB_TCP6TABLE_OWNER_PID>();
                // SAFETY: The API Wrote dwNumEntries Rows Into the Buffer.
                let rows = unsafe {
                    std::slice::from_raw_parts(
                        std::ptr::addr_of!((*table).table).cast::<MIB_TCP6ROW_OWNER_PID>(),
                        (*table).dwNumEntries as usize,
                    )
                };
                for row in rows {
                    let remote = Ipv6Addr::from(row.ucRemoteAddr);
                    if row.dwOwningPid == pid && row.dwState == TCP_ESTABLISHED && !loopback(remote) {
                        locals.insert(ip_text(Ipv6Addr::from(row.ucLocalAddr)));
                    }
                }
            }
            break;
        }
    }
    Ok(locals)
}

/// The Active Plan's GUID and Friendly Name.
pub fn active_scheme() -> Option<(String, Option<String>)> {
    let mut active: *mut GUID = null_mut();
    // SAFETY: The API Writes a LocalAlloc'd GUID Into `active`, Which Is Copied and Freed Once.
    let code = unsafe { PowerGetActiveScheme(None, &mut active) };
    if code != ERROR_SUCCESS || active.is_null() {
        return None;
    }
    // SAFETY: A Successful Call Returned a Valid GUID Allocation.
    let guid = unsafe { *active };
    let allocation = HLOCAL(active.cast());
    // SAFETY: The Allocation Came From LocalAlloc and Is Not Used Again.
    unsafe { LocalFree(Some(allocation)) };
    Some((guid_string(&guid), friendly_name(&guid)))
}

/* === Private Functions === */

fn radio_power(state: windows::Devices::Radios::RadioState) -> Result<RadioPower> {
    match state {
        windows::Devices::Radios::RadioState::On => Ok(RadioPower::On),
        windows::Devices::Radios::RadioState::Off => Ok(RadioPower::Off),
        windows::Devices::Radios::RadioState::Disabled | windows::Devices::Radios::RadioState::Unknown => {
            Err(Error::Invalid("Read Radio: Enable the Radio in Hardware or Policy First".into()))
        }
        value => Err(Error::Invalid(format!("Read Radio: Report the Unknown Radio State {}", value.0))),
    }
}

fn set_radio(radio: &windows::Devices::Radios::Radio, target: windows::Devices::Radios::RadioState, deadline: Instant) -> Result<()> {
    let (changed, changes) = mpsc::sync_channel(1);
    let handler = TypedEventHandler::<windows::Devices::Radios::Radio, IInspectable>::new(move |_, _| {
        let _ = changed.try_send(());
        Ok(())
    });
    let token = radio.StateChanged(&handler).map_err(probe)?;
    let _watching = Watching { radio: radio.clone(), token };
    let access = radio.SetStateAsync(target).map_err(probe)?.join().map_err(probe)?;
    if access != RadioAccessStatus::Allowed {
        return Err(Error::Invalid(format!("Set Radio: Allow Radio Access for the Host Plugin; Windows Refused With {}", access.0)));
    }
    loop {
        if radio.State().map_err(probe)? == target {
            return Ok(());
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        changes
            .recv_timeout(remaining)
            .map_err(|_| Error::Invalid("Set Radio: Raise Radio Settle Timeout; the Radio Did Not Settle".into()))?;
    }
}

fn set_adapter(id: &str, enabled: bool) -> Result<()> {
    let devices = open()?;
    let device = adapter_device(&devices, id)?;
    let mut install = SP_DEVINSTALL_PARAMS_W {
        cbSize: std::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as u32,
        ..Default::default()
    };
    // SAFETY: The Device Belongs to the Live Information Set.
    unsafe { SetupDiGetDeviceInstallParamsW(devices.0, Some(&device), &mut install) }.map_err(probe)?;
    install.Flags |= DI_QUIETINSTALL;
    unsafe { SetupDiSetDeviceInstallParamsW(devices.0, Some(&device), &install) }.map_err(probe)?;
    let change = SP_PROPCHANGE_PARAMS {
        ClassInstallHeader: SP_CLASSINSTALL_HEADER {
            cbSize: std::mem::size_of::<SP_CLASSINSTALL_HEADER>() as u32,
            InstallFunction: DIF_PROPERTYCHANGE,
        },
        StateChange: if enabled { DICS_ENABLE } else { DICS_DISABLE },
        Scope: DICS_FLAG_GLOBAL,
        HwProfile: 0,
    };
    // SAFETY: The Header Begins a Property Change Structure of the Passed Size.
    unsafe {
        SetupDiSetClassInstallParamsW(devices.0, Some(&device), Some(&change.ClassInstallHeader), std::mem::size_of_val(&change) as u32)
    }
    .map_err(probe)?;
    unsafe { SetupDiCallClassInstaller(DIF_PROPERTYCHANGE, devices.0, Some(&device)) }.map_err(probe)?;
    if read_adapter(id)? != enabled {
        return Err(Error::Invalid("Set Adapter: Restart Windows; the Adapter Did Not Settle to the Requested State".into()));
    }
    Ok(())
}

fn open() -> Result<Devices> {
    // SAFETY: The Class GUID Is a Static Constant.
    unsafe { SetupDiGetClassDevsW(Some(&GUID_DEVCLASS_NET), None, None, DIGCF_PRESENT) }.map(Devices).map_err(probe)
}

fn entry(devices: &Devices, index: u32) -> Result<Option<SP_DEVINFO_DATA>> {
    let mut device = SP_DEVINFO_DATA {
        cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
        ..Default::default()
    };
    // SAFETY: The Set Is Live and the Structure Declares Its Size.
    match unsafe { SetupDiEnumDeviceInfo(devices.0, index, &mut device) } {
        Ok(()) => Ok(Some(device)),
        Err(error) if error.code() == HRESULT::from_win32(ERROR_NO_MORE_ITEMS.0) => Ok(None),
        Err(error) => Err(probe(error)),
    }
}

/// The Network Device Whose Driver Key Names the Adapter's Inventory Identity.
fn adapter_device(devices: &Devices, id: &str) -> Result<SP_DEVINFO_DATA> {
    let mut index = 0;
    while let Some(device) = entry(devices, index)? {
        index += 1;
        // SAFETY: The Device Belongs to the Live Information Set; the Key Is Closed Below.
        let Ok(key) = (unsafe { SetupDiOpenDevRegKey(devices.0, &device, DICS_FLAG_GLOBAL.0, 0, DIREG_DRV, KEY_QUERY_VALUE.0) }) else {
            continue;
        };
        let mut size = 0;
        let code = unsafe { RegQueryValueExW(key, w!("NetCfgInstanceId"), None, None, None, Some(&mut size)) };
        if code != ERROR_SUCCESS || size == 0 {
            let _ = unsafe { RegCloseKey(key) };
            continue;
        }
        let mut value = vec![0u16; (size as usize).div_ceil(2)];
        let code = unsafe { RegQueryValueExW(key, w!("NetCfgInstanceId"), None, None, Some(value.as_mut_ptr().cast()), Some(&mut size)) };
        let _ = unsafe { RegCloseKey(key) };
        if code != ERROR_SUCCESS {
            continue;
        }
        let end = value.iter().position(|unit| *unit == 0).unwrap_or(value.len());
        if String::from_utf16_lossy(&value[..end]).eq_ignore_ascii_case(id) {
            return Ok(device);
        }
    }
    Err(Error::Invalid(format!("Set Adapter: Choose an Identity From host.network.adapters; {id} Was Not Found")))
}

fn friendly_name(scheme: &GUID) -> Option<String> {
    let mut size = 0u32;
    // SAFETY: A Null Buffer Asks Only for the Required Byte Size, So the Status Is Not the Answer.
    let _ = unsafe { PowerReadFriendlyName(None, Some(scheme), None, None, None, &mut size) };
    if size == 0 {
        return None;
    }
    let mut buffer = vec![0u16; (size as usize).div_ceil(2)];
    let output = buffer.as_mut_ptr().cast::<u8>();
    // SAFETY: The Buffer Holds at Least `size` Bytes.
    let code = unsafe { PowerReadFriendlyName(None, Some(scheme), None, None, Some(output), &mut size) };
    if code != ERROR_SUCCESS {
        return None;
    }
    let end = buffer.iter().position(|unit| *unit == 0).unwrap_or(buffer.len());
    let name = String::from_utf16_lossy(&buffer[..end]).trim().to_owned();
    (!name.is_empty()).then_some(name)
}

/// Lowercase Hyphenated Form, as powercfg Prints It.
fn guid_string(guid: &GUID) -> String {
    let [a, b, c, d, e, f, g, h] = guid.data4;
    format!("{:08x}-{:04x}-{:04x}-{a:02x}{b:02x}-{c:02x}{d:02x}{e:02x}{f:02x}{g:02x}{h:02x}", guid.data1, guid.data2, guid.data3)
}

fn loopback(address: Ipv6Addr) -> bool {
    address.is_loopback() || address.to_ipv4_mapped().is_some_and(|address| address.is_loopback())
}

fn ip_text(address: Ipv6Addr) -> String {
    address.to_ipv4_mapped().map(|address| address.to_string()).unwrap_or_else(|| address.to_string())
}

fn probe(error: impl std::fmt::Display) -> Error {
    Error::Invalid(format!("Set Control: Retry Once Windows Answers: {error}"))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_radio_power_refuses_disabled_and_unknown_states() {
        assert_eq!(radio_power(windows::Devices::Radios::RadioState::On).unwrap(), RadioPower::On);
        assert_eq!(radio_power(windows::Devices::Radios::RadioState::Off).unwrap(), RadioPower::Off);
        assert!(radio_power(windows::Devices::Radios::RadioState::Disabled).is_err());
        assert!(radio_power(windows::Devices::Radios::RadioState::Unknown).is_err());
        assert!(radio_power(windows::Devices::Radios::RadioState(99)).is_err());
    }

    #[test]
    fn guid_prints_like_powercfg() {
        let guid = GUID::from_u128(0x381b4222_f694_41f0_9685_ff5bb260df2e);
        assert_eq!(guid_string(&guid), "381b4222-f694-41f0-9685-ff5bb260df2e");
    }

    #[test]
    fn mapped_loopback_counts_as_loopback() {
        assert!(loopback(Ipv6Addr::LOCALHOST));
        assert!(loopback(Ipv4Addr::LOCALHOST.to_ipv6_mapped()));
        assert!(!loopback(Ipv4Addr::new(192, 0, 2, 1).to_ipv6_mapped()));
        assert_eq!(ip_text(Ipv4Addr::new(192, 0, 2, 1).to_ipv6_mapped()), "192.0.2.1");
    }
}
