/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Network Connectivity Through the Network List Manager and IP Helper Change Notifications

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::ffi::c_void;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ptr::null_mut;
use std::sync::{Mutex, PoisonError};

// Third Party Imports
use windows::Win32::Foundation::{ERROR_SUCCESS, HANDLE, RPC_E_CHANGED_MODE, WIN32_ERROR};
use windows::Win32::NetworkManagement::IpHelper::{
    CancelMibChangeNotify2, ConvertInterfaceLuidToGuid, FreeMibTable, GetIfEntry2, GetIfTable2, GetIpForwardTable2, MIB_IF_ROW2,
    MIB_IPFORWARD_ROW2, MIB_IPINTERFACE_ROW, MIB_NOTIFICATION_TYPE, MIB_UNICASTIPADDRESS_ROW, MibAddInstance, MibDeleteInstance,
    NotifyIpInterfaceChange, NotifyRouteChange2, NotifyUnicastIpAddressChange,
};
use windows::Win32::NetworkManagement::Ndis::{IfOperStatusUp, NET_LUID_LH};
use windows::Win32::Networking::NetworkListManager::{
    INetworkListManager, NLM_CONNECTIVITY_IPV4_INTERNET, NLM_CONNECTIVITY_IPV6_INTERNET, NetworkListManager,
};
use windows::Win32::Networking::WinSock::{AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR, SOCKADDR_IN, SOCKADDR_IN6};
use windows::Win32::System::Com::{CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize};
use windows::core::GUID;

// Local Imports
use crate::collectors::network::{Change, Pending};
use dokime::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Link, Address and Default-Route Changes Queued by IP Helper Callbacks Until the Next Drain.
pub struct Notifications {
    handles: Vec<usize>,
    sink: Box<Mutex<Pending>>,
}

/// Balances a Successful CoInitializeEx on This Thread.
struct Apartment(bool);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Notifications {
    pub fn new() -> Result<Self> {
        let mut notifications = Self {
            handles: Vec::new(),
            sink: Box::new(Mutex::new(Pending::new(links()?))),
        };
        let context = (&*notifications.sink as *const Mutex<Pending>).cast::<c_void>();
        let mut handle = HANDLE::default();
        // SAFETY: The Box's address stays stable; Drop cancels and joins callbacks before freeing it.
        let code = unsafe { NotifyIpInterfaceChange(AF_UNSPEC, Some(link), Some(context), false, &mut handle) };
        notifications.register(code, handle)?;
        // SAFETY: As above.
        let code = unsafe { NotifyUnicastIpAddressChange(AF_UNSPEC, Some(address), Some(context), false, &mut handle) };
        notifications.register(code, handle)?;
        // SAFETY: As above.
        let code = unsafe { NotifyRouteChange2(AF_UNSPEC, Some(route), context, false, &mut handle) };
        notifications.register(code, handle)?;
        Ok(notifications)
    }

    fn register(&mut self, code: WIN32_ERROR, handle: HANDLE) -> Result<()> {
        if code != ERROR_SUCCESS {
            return Err(Error::Invalid(format!("Watch Network Changes: IP Helper Registration Returned {}; Restart the Plugin", code.0)));
        }
        self.handles.push(handle.0 as usize);
        Ok(())
    }

    pub fn next(&mut self) -> Result<Option<Change>> {
        self.sink.lock().unwrap_or_else(PoisonError::into_inner).pop()
    }
}

impl Drop for Notifications {
    fn drop(&mut self) {
        // Never Hold the Sink Lock While Waiting for a Callback to Finish.
        let mut cancelled = true;
        for handle in self.handles.drain(..) {
            // SAFETY: Each handle came from a successful registration and is cancelled once.
            cancelled &= unsafe { CancelMibChangeNotify2(HANDLE(handle as *mut c_void)) } == ERROR_SUCCESS;
        }
        if !cancelled {
            // Preserve Callback Memory if the OS Could Not Confirm Deregistration.
            let live = std::mem::replace(&mut self.sink, Box::new(Mutex::new(Pending::new(BTreeMap::new()))));
            let _ = Box::leak(live);
        }
    }
}

impl Drop for Apartment {
    fn drop(&mut self) {
        if self.0 {
            // SAFETY: Paired with the successful CoInitializeEx that created this guard on this thread.
            unsafe { CoUninitialize() };
        }
    }
}

/* === Public Functions === */

/// Internet Over IPv4 or IPv6 as the Network List Manager Reports It, Without Own Traffic.
pub fn internet() -> Option<bool> {
    super::apartment::retain().ok()?;
    // SAFETY: Initializing COM has no preconditions; a thread already in another apartment keeps it.
    let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    if initialized.is_err() && initialized != RPC_E_CHANGED_MODE {
        return None;
    }
    let _apartment = Apartment(initialized.is_ok());
    // SAFETY: The class and interface identifiers come from the Network List Manager declarations.
    let manager: INetworkListManager = unsafe { CoCreateInstance(&NetworkListManager, None, CLSCTX_ALL) }.ok()?;
    // SAFETY: The manager is a live COM object on this thread.
    let connectivity = unsafe { manager.GetConnectivity() }.ok()?;
    let internet = NLM_CONNECTIVITY_IPV4_INTERNET.0 | NLM_CONNECTIVITY_IPV6_INTERNET.0;
    Some(connectivity.0 & internet != 0)
}

/// The Adapter Carrying a Default Route, by Its Lowercase Braced GUID.
pub fn primary_interface() -> Result<Option<String>> {
    let mut table = null_mut();
    // SAFETY: The API allocates its table; it is freed after reading every row.
    let code = unsafe { GetIpForwardTable2(AF_UNSPEC, &mut table) };
    if code != ERROR_SUCCESS {
        return Err(Error::Invalid(format!("Read the Default Route: GetIpForwardTable2 Returned {}; Restart the Plugin", code.0)));
    }
    // SAFETY: A successful call returned a table of NumEntries rows.
    let rows: &[MIB_IPFORWARD_ROW2] = unsafe { std::slice::from_raw_parts((*table).Table.as_ptr(), (*table).NumEntries as usize) };
    let primary = rows.iter().filter(|row| row.DestinationPrefix.PrefixLength == 0).find_map(|row| adapter_id(&row.InterfaceLuid));
    // SAFETY: The table came from GetIpForwardTable2 and is not used again.
    unsafe { FreeMibTable(table.cast()) };
    Ok(primary)
}

/* === Private Functions === */

/// Every Interface's Operational State, So Notifications Report Only Real Link Transitions.
fn links() -> Result<BTreeMap<String, bool>> {
    let mut table = null_mut();
    // SAFETY: The API allocates its table; it is freed after reading every row.
    let code = unsafe { GetIfTable2(&mut table) };
    if code != ERROR_SUCCESS {
        return Err(Error::Invalid(format!("Read Network Links: GetIfTable2 Returned {}; Restart the Plugin", code.0)));
    }
    // SAFETY: A successful call returned a table of NumEntries rows.
    let rows: &[MIB_IF_ROW2] = unsafe { std::slice::from_raw_parts((*table).Table.as_ptr(), (*table).NumEntries as usize) };
    let links = rows.iter().filter_map(|row| Some((adapter_id(&row.InterfaceLuid)?, row.OperStatus == IfOperStatusUp))).collect();
    // SAFETY: The table came from GetIfTable2 and is not used again.
    unsafe { FreeMibTable(table.cast()) };
    Ok(links)
}

fn adapter_id(luid: &NET_LUID_LH) -> Option<String> {
    let mut guid = GUID::zeroed();
    // SAFETY: Both pointers are valid for this call.
    if unsafe { ConvertInterfaceLuidToGuid(luid, &mut guid) } != ERROR_SUCCESS {
        return None;
    }
    Some(format!("{{{guid:?}}}").to_lowercase())
}

fn push(context: *const c_void, kind: &str, luid: &NET_LUID_LH, address: Option<String>, up: Option<bool>) {
    // SAFETY: The context is the live sink registered with the notification.
    let sink = unsafe { &*context.cast::<Mutex<Pending>>() };
    let mut pending = sink.lock().unwrap_or_else(PoisonError::into_inner);
    match adapter_id(luid) {
        Some(id) => pending.push(Change::new(kind, id, address), up),
        None => pending.fail(),
    }
}

/// The Textual Form of a Native Socket Address.
///
/// SAFETY: The caller supplies a live native socket address from an IP Helper row.
unsafe fn socket_address(address: *const SOCKADDR) -> Option<String> {
    if address.is_null() {
        return None;
    }
    match unsafe { (*address).sa_family } {
        AF_INET => {
            let address = unsafe { &*address.cast::<SOCKADDR_IN>() };
            Some(Ipv4Addr::from(unsafe { address.sin_addr.S_un.S_addr }.to_ne_bytes()).to_string())
        }
        AF_INET6 => {
            let address = unsafe { &*address.cast::<SOCKADDR_IN6>() };
            Some(Ipv6Addr::from(unsafe { address.sin6_addr.u.Byte }).to_string())
        }
        _ => None,
    }
}

unsafe extern "system" fn link(context: *const c_void, row: *const MIB_IPINTERFACE_ROW, kind: MIB_NOTIFICATION_TYPE) {
    if row.is_null() {
        return;
    }
    // SAFETY: The OS passes a live row for the callback's duration.
    let row = unsafe { &*row };
    let mut interface = MIB_IF_ROW2 {
        InterfaceLuid: row.InterfaceLuid,
        ..Default::default()
    };
    // SAFETY: The row is valid for this call.
    if unsafe { GetIfEntry2(&mut interface) } != ERROR_SUCCESS && kind != MibDeleteInstance {
        // SAFETY: The context is the live sink registered with the notification.
        unsafe { &*context.cast::<Mutex<Pending>>() }.lock().unwrap_or_else(PoisonError::into_inner).fail();
        return;
    }
    let up = kind != MibDeleteInstance && interface.OperStatus == IfOperStatusUp;
    push(context, if up { "link_up" } else { "link_down" }, &row.InterfaceLuid, None, Some(up));
}

unsafe extern "system" fn address(context: *const c_void, row: *const MIB_UNICASTIPADDRESS_ROW, kind: MIB_NOTIFICATION_TYPE) {
    if row.is_null() || (kind != MibAddInstance && kind != MibDeleteInstance) {
        return;
    }
    // SAFETY: The OS passes a live row for the callback's duration.
    let row = unsafe { &*row };
    // SAFETY: The address belongs to the live row.
    let text = unsafe { socket_address(std::ptr::addr_of!(row.Address).cast::<SOCKADDR>()) };
    push(context, if kind == MibDeleteInstance { "address_removed" } else { "address_added" }, &row.InterfaceLuid, text, None);
}

unsafe extern "system" fn route(context: *const c_void, row: *const MIB_IPFORWARD_ROW2, _: MIB_NOTIFICATION_TYPE) {
    if row.is_null() {
        return;
    }
    // SAFETY: The OS passes a live row for the callback's duration.
    let row = unsafe { &*row };
    if row.DestinationPrefix.PrefixLength != 0 {
        return;
    }
    push(context, "default_route_changed", &row.InterfaceLuid, None, None);
}

/* === Tests === */
