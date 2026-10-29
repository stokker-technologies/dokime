/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Internet Connectivity As The Network List Manager Reports It.

use windows::Win32::{
    Foundation::RPC_E_CHANGED_MODE,
    Networking::NetworkListManager::{
        INetworkListManager, NLM_CONNECTIVITY_IPV4_INTERNET, NLM_CONNECTIVITY_IPV6_INTERNET, NetworkListManager,
    },
    System::Com::{CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize},
};

// Balances A Successful CoInitializeEx On This Thread.
struct Apartment(bool);

impl Drop for Apartment {
    fn drop(&mut self) {
        if self.0 {
            // SAFETY: Paired with the successful CoInitializeEx that created this guard on this thread.
            unsafe { CoUninitialize() };
        }
    }
}

pub fn internet() -> Option<bool> {
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

#[cfg(test)]
mod tests {
    #[test]
    fn connectivity_reads_without_a_process() {
        assert!(super::internet().is_some());
    }
}
