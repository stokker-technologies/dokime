/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Windows Plugin Resident Memory

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::mem::size_of;

// Third Party Imports
use windows_sys::Win32::{
    Foundation::CloseHandle,
    System::{
        ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS},
        Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    },
};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn resident(pid: u32) -> Option<f64> {
    // The Handle Is Owned Here and Closed After the Single Inspection Call.
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
        if process.is_null() {
            return None;
        }
        let mut counters = PROCESS_MEMORY_COUNTERS {
            cb: size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            ..Default::default()
        };
        let size = counters.cb;
        let success = GetProcessMemoryInfo(process, &mut counters, size);
        CloseHandle(process);
        (success != 0).then_some(counters.WorkingSetSize as f64)
    }
}

/* === Private Functions === */

/* === Tests === */
