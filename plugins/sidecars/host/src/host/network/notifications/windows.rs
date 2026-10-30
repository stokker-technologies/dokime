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
    collections::{BTreeMap, VecDeque},
    ffi::c_void,
    sync::Mutex,
};

// Third Party Imports
use windows::{
    Win32::{
        Foundation::{ERROR_SUCCESS, HANDLE},
        NetworkManagement::{
            IpHelper::*,
            Ndis::{IfOperStatusUp, NET_LUID_LH},
        },
        Networking::WinSock::{AF_UNSPEC, SOCKADDR},
    },
    core::GUID,
};

// Local Imports
use super::Change;
use dokime::StreamSpec;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Notifications {
    handles: Vec<usize>,
    sink: Box<Sink>,
}

struct Sink {
    pending: Mutex<Pending>,
}

struct Pending {
    events: VecDeque<Change>,
    links: BTreeMap<String, bool>,
    failed: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Notifications {
    pub fn new() -> crate::Result<Self> {
        let inventory = crate::host::adapters::snapshot()?;
        let links = inventory
            .adapters
            .into_iter()
            .map(|adapter| {
                let id = adapter.id.to_lowercase();
                let up = adapter.status == crate::host::adapters::AdapterStatus::Up;
                (id, up)
            })
            .collect();
        let mut notifications = Self {
            handles: Vec::new(),
            sink: Box::new(Sink {
                pending: Mutex::new(Pending {
                    events: VecDeque::new(),
                    links,
                    failed: false,
                }),
            }),
        };
        let context = (&*notifications.sink as *const Sink).cast::<c_void>();
        let mut handle = HANDLE::default();
        // SAFETY: The Box's Address Stays Stable; Drop Cancels And Joins Callbacks Before Freeing It.
        let code = unsafe { NotifyIpInterfaceChange(AF_UNSPEC, Some(link), Some(context), false, &mut handle) };
        notifications.register(code, handle)?;
        let code = unsafe { NotifyUnicastIpAddressChange(AF_UNSPEC, Some(address), Some(context), false, &mut handle) };
        notifications.register(code, handle)?;
        let code = unsafe { NotifyRouteChange2(AF_UNSPEC, Some(route), context, false, &mut handle) };
        notifications.register(code, handle)?;
        Ok(notifications)
    }

    fn register(&mut self, code: windows::Win32::Foundation::WIN32_ERROR, handle: HANDLE) -> crate::Result<()> {
        if code != ERROR_SUCCESS {
            return Err(crate::Error::Probe(format!("network notification registration: {}", code.0)));
        }
        self.handles.push(handle.0 as usize);
        Ok(())
    }

    pub fn drain(&mut self) -> crate::Result<Vec<Change>> {
        let mut pending = self.sink.pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if pending.failed {
            return Err(crate::Error::Probe("network notification buffer overflow or native lookup failure".into()));
        }
        Ok(pending.events.drain(..).collect())
    }
}

impl Drop for Notifications {
    fn drop(&mut self) {
        // Never Hold The Sink Lock While Waiting For A Callback To Finish.
        let mut cancelled = true;
        for handle in self.handles.drain(..) {
            let code = unsafe { CancelMibChangeNotify2(HANDLE(handle as *mut c_void)) };
            if code != ERROR_SUCCESS {
                cancelled = false;
            }
        }
        if !cancelled {
            // Preserve Callback Memory If The OS Could Not Confirm Deregistration.
            let empty = Box::new(Sink {
                pending: Mutex::new(Pending {
                    events: VecDeque::new(),
                    links: BTreeMap::new(),
                    failed: true,
                }),
            });
            let live = std::mem::replace(&mut self.sink, empty);
            let _ = Box::leak(live);
            eprintln!("host network notification cancellation failed; callback memory retained");
        }
    }
}

impl Sink {
    fn push(&self, kind: &str, luid: &NET_LUID_LH, address: Option<String>, up: Option<bool>) {
        let mut pending = self.pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut guid = GUID::zeroed();
        if unsafe { ConvertInterfaceLuidToGuid(luid, &mut guid) } != ERROR_SUCCESS {
            pending.failed = true;
            return;
        }
        let id = format!("{{{guid:?}}}").to_lowercase();
        pending.push(Change::new(kind, id, address), up);
    }
}

impl Pending {
    fn push(&mut self, change: Change, up: Option<bool>) {
        if let Some(up) = up
            && self.links.insert(change.adapter_id.clone(), up) == Some(up)
        {
            return;
        }
        if self.events.len() >= <crate::Network as StreamSpec>::WINDOW.saturating_sub(1) {
            self.failed = true;
            return;
        }
        self.events.push_back(change);
    }
}

/* === Public Functions === */

/* === Private Functions === */

unsafe extern "system" fn link(context: *const c_void, row: *const MIB_IPINTERFACE_ROW, kind: MIB_NOTIFICATION_TYPE) {
    if row.is_null() {
        return;
    }
    let row = unsafe { &*row };
    let sink = unsafe { &*context.cast::<Sink>() };
    let mut interface = MIB_IF_ROW2 {
        InterfaceLuid: row.InterfaceLuid,
        ..Default::default()
    };
    let code = unsafe { GetIfEntry2(&mut interface) };
    if code != ERROR_SUCCESS && kind != MibDeleteInstance {
        sink.pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner).failed = true;
        return;
    }
    let up = kind != MibDeleteInstance && interface.OperStatus == IfOperStatusUp;
    sink.push(if up { "link_up" } else { "link_down" }, &row.InterfaceLuid, None, Some(up));
}

unsafe extern "system" fn address(context: *const c_void, row: *const MIB_UNICASTIPADDRESS_ROW, kind: MIB_NOTIFICATION_TYPE) {
    if row.is_null() || (kind != MibAddInstance && kind != MibDeleteInstance) {
        return;
    }
    let row = unsafe { &*row };
    let text =
        unsafe { crate::host::adapters::windows::socket_address(std::ptr::addr_of!(row.Address).cast::<SOCKADDR>()) }.map(|(text, _)| text);
    let sink = unsafe { &*context.cast::<Sink>() };
    sink.push(if kind == MibDeleteInstance { "address_removed" } else { "address_added" }, &row.InterfaceLuid, text, None);
}

unsafe extern "system" fn route(context: *const c_void, row: *const MIB_IPFORWARD_ROW2, _: MIB_NOTIFICATION_TYPE) {
    if row.is_null() {
        return;
    }
    let row = unsafe { &*row };
    if row.DestinationPrefix.PrefixLength != 0 {
        return;
    }
    let sink = unsafe { &*context.cast::<Sink>() };
    sink.push("default_route_changed", &row.InterfaceLuid, None, None);
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn notification_queue() {
        let event = |kind: &str| Change {
            kind: kind.into(),
            adapter_id: "nic".into(),
            address: None,
            timestamp_micros: 7,
        };
        let mut pending = Pending {
            events: VecDeque::new(),
            links: BTreeMap::from([("nic".into(), true)]),
            failed: false,
        };
        pending.push(event("link_up"), Some(true));
        assert!(pending.events.is_empty());
        pending.push(event("address_added"), None);
        pending.push(event("link_down"), Some(false));
        assert_eq!(pending.events.drain(..).map(|change| change.kind).collect::<Vec<_>>(), ["address_added", "link_down"]);
        let capacity = <crate::Network as StreamSpec>::WINDOW - 1;
        for _ in 0..capacity {
            pending.push(event("default_route_changed"), None);
        }
        assert!(!pending.failed);
        pending.push(event("address_removed"), None);
        assert!(pending.failed);
        assert_eq!(pending.events.len(), capacity);
    }
}
